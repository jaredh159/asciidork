use std::{cell::RefCell, rc::Rc};

use crate::internal::*;

#[derive(Debug)]
pub struct Parser<'arena> {
  pub(super) bump: &'arena Bump,
  pub(super) lexer: Lexer<'arena>,
  pub(super) document: Document<'arena>,
  pub(super) peeked_lines: Option<ContiguousLines<'arena>>,
  pub(super) peeked_meta: Option<ChunkMeta<'arena>>,
  pub(super) ctx: ParseContext<'arena>,
  pub(super) errors: RefCell<Vec<Diagnostic>>,
  pub(super) strict: bool, // todo: naming...
}

pub struct ParseResult<'arena> {
  pub document: Document<'arena>,
  pub warnings: Vec<Diagnostic>,
}

#[derive(Debug, Default)]
pub(crate) struct ListContext {
  pub(crate) stack: ListStack,
  pub(crate) parsing_continuations: bool,
}

impl<'arena> Parser<'arena> {
  pub fn new(src: BumpVec<'arena, u8>, bump: &'arena Bump) -> Self {
    Parser {
      bump,
      lexer: Lexer::new(src, bump),
      document: Document::new(bump),
      peeked_lines: None,
      peeked_meta: None,
      ctx: ParseContext::new(bump),
      errors: RefCell::new(Vec::new()),
      strict: true,
    }
  }

  pub fn from_str(src: &str, bump: &'arena Bump) -> Self {
    Parser {
      bump,
      lexer: Lexer::from_str(bump, src),
      document: Document::new(bump),
      peeked_lines: None,
      peeked_meta: None,
      ctx: ParseContext::new(bump),
      errors: RefCell::new(Vec::new()),
      strict: true,
    }
  }

  pub fn apply_job_settings(&mut self, settings: JobSettings) {
    self.strict = settings.strict;
    self.document.meta = settings.into();
  }

  pub fn cell_parser(&mut self, src: BumpVec<'arena, u8>, offset: usize) -> Parser<'arena> {
    let mut cell_parser = Parser::new(src, self.bump);
    cell_parser.strict = self.strict;
    cell_parser.lexer.adjust_offset(offset);
    cell_parser.ctx = self.ctx.clone_for_cell();
    cell_parser.document.meta = self.document.meta.clone_for_cell();
    cell_parser.document.anchors = Rc::clone(&self.document.anchors);
    cell_parser
  }

  pub(crate) fn loc(&self) -> SourceLocation {
    self
      .peeked_lines
      .as_ref()
      .and_then(|lines| lines.loc())
      .unwrap_or_else(|| self.lexer.loc())
  }

  pub(crate) fn read_line(&mut self) -> Option<Line<'arena>> {
    debug_assert!(self.peeked_lines.is_none());
    self.lexer.consume_line()
  }

  pub(crate) fn read_lines(&mut self) -> Option<ContiguousLines<'arena>> {
    if let Some(peeked) = self.peeked_lines.take() {
      return Some(peeked);
    }
    self.lexer.consume_empty_lines();
    if self.lexer.is_eof() {
      return None;
    }
    let mut lines = Deq::new(self.bump);
    while let Some(line) = self.lexer.consume_line() {
      lines.push(line);
      if self.lexer.peek_is(b'\n') {
        break;
      }
    }
    debug_assert!(!lines.is_empty());
    Some(ContiguousLines::new(lines))
  }

  pub(crate) fn read_lines_until(
    &mut self,
    delimiter: Delimiter,
  ) -> Option<ContiguousLines<'arena>> {
    let mut lines = self.read_lines()?;
    if lines.any(|l| l.is_delimiter(delimiter)) {
      return Some(lines);
    }

    let mut additional_lines = BumpVec::new_in(self.bump);
    while !self.lexer.is_eof() && !self.at_delimiter(delimiter) {
      additional_lines.push(self.read_line().unwrap());
    }
    lines.extend(additional_lines);
    Some(lines)
  }

  fn at_delimiter(&self, delimiter: Delimiter) -> bool {
    match delimiter {
      Delimiter::BlockQuote => self.lexer.at_delimiter_line() == Some((4, b'_')),
      Delimiter::Example => self.lexer.at_delimiter_line() == Some((4, b'=')),
      Delimiter::Open => self.lexer.at_delimiter_line() == Some((2, b'-')),
      Delimiter::Sidebar => self.lexer.at_delimiter_line() == Some((4, b'*')),
      Delimiter::Listing => self.lexer.at_delimiter_line() == Some((4, b'-')),
      Delimiter::Literal => self.lexer.at_delimiter_line() == Some((4, b'.')),
      Delimiter::Passthrough => self.lexer.at_delimiter_line() == Some((4, b'+')),
      Delimiter::Comment => self.lexer.at_delimiter_line() == Some((4, b'/')),
    }
  }

  pub(crate) fn restore_lines(&mut self, lines: ContiguousLines<'arena>) {
    debug_assert!(self.peeked_lines.is_none());
    if !lines.is_empty() {
      self.peeked_lines = Some(lines);
    }
  }

  pub(crate) fn restore_peeked_meta(&mut self, meta: ChunkMeta<'arena>) {
    debug_assert!(self.peeked_meta.is_none());
    self.peeked_meta = Some(meta);
  }

  pub(crate) fn restore_peeked(&mut self, lines: ContiguousLines<'arena>, meta: ChunkMeta<'arena>) {
    self.restore_lines(lines);
    self.restore_peeked_meta(meta);
  }

  pub fn parse(mut self) -> std::result::Result<ParseResult<'arena>, Vec<Diagnostic>> {
    self.parse_document_header()?;

    // ensure we only read a single "paragraph" for `inline` doc_type
    // https://docs.asciidoctor.org/asciidoc/latest/document/doctype/#inline-doctype-rules
    if self.document.meta.get_doctype() == DocType::Inline {
      if self.peeked_lines.is_none() {
        self.peeked_lines = self.read_lines();
      }
      self.lexer.truncate();
    }

    while let Some(chunk) = self.parse_chunk()? {
      match chunk {
        Chunk::Block(block) => self.document.content.push_block(block, self.bump),
        Chunk::Section(section) => self.document.content.push_section(section, self.bump),
      }
    }

    // clear the doc attrs so the backend can see them replayed in decl order
    self.document.meta.clear_doc_attrs();

    self.diagnose_document()?;

    Ok(ParseResult {
      document: self.document,
      warnings: vec![],
    })
  }

  fn parse_chunk(&mut self) -> Result<Option<Chunk<'arena>>> {
    match self.parse_section()? {
      Some(section) => Ok(Some(Chunk::Section(section))),
      None => Ok(self.parse_block()?.map(Chunk::Block)),
    }
  }

  pub(crate) fn parse_chunk_meta(
    &mut self,
    lines: &mut ContiguousLines<'arena>,
  ) -> Result<ChunkMeta<'arena>> {
    if let Some(meta) = self.peeked_meta.take() {
      return Ok(meta);
    }
    let start = lines.current_token().unwrap().loc.start;
    let mut attrs = None;
    let mut title = None;
    loop {
      match lines.current() {
        Some(line) if line.is_chunk_title() => {
          let mut line = lines.consume_current().unwrap();
          line.discard_assert(TokenKind::Dots);
          title = Some(self.parse_inlines(&mut line.into_lines())?);
        }
        Some(line) if line.is_attr_list() => {
          let mut line = lines.consume_current().unwrap();
          line.discard_assert(TokenKind::OpenBracket);
          attrs = Some(self.parse_attr_list(&mut line)?);
        }
        _ => break,
      }
    }
    Ok(ChunkMeta { attrs, title, start })
  }

  fn diagnose_document(&self) -> Result<()> {
    if !self.ctx.in_asciidoc_table_cell {
      for (ref_id, ref_loc) in self.ctx.xrefs.borrow().iter() {
        if !self.document.anchors.borrow().contains_key(ref_id) {
          self.err_at_loc(
            format!("Invalid cross reference, no anchor found for `{ref_id}`"),
            *ref_loc,
          )?;
        }
      }
    }
    let toc_pos = self.document.toc.as_ref().map(|toc| toc.position);
    match toc_pos {
      Some(TocPosition::Macro) if !self.ctx.saw_toc_macro => {
        self.err_doc_attr(
          ":toc:",
          "Table of Contents set to `macro` but macro (`toc::[]`) not found",
        )?;
      }
      Some(TocPosition::Preamble) => match &self.document.content {
        DocContent::Blocks(_) | DocContent::Sectioned { preamble: None, .. } => {
          self.err_doc_attr(
            ":toc:",
            "Table of Contents set to `preamble` but no preamble found",
          )?;
        }
        _ => {}
      },
      _ => {}
    }
    Ok(())
  }
}

#[derive(Debug)]
pub enum Chunk<'arena> {
  Block(Block<'arena>),
  Section(Section<'arena>),
}

impl From<Diagnostic> for Vec<Diagnostic> {
  fn from(diagnostic: Diagnostic) -> Self {
    vec![diagnostic]
  }
}
