use std::cell::RefCell;

use crate::internal::*;

#[derive(Debug)]
pub struct Parser<'bmp: 'src, 'src> {
  pub(super) bump: &'bmp Bump,
  pub(super) lexer: Lexer<'src>,
  pub(super) document: Document<'bmp>,
  pub(super) peeked_lines: Option<ContiguousLines<'bmp, 'src>>,
  pub(super) peeked_meta: Option<ChunkMeta<'bmp>>,
  pub(super) ctx: ParseContext<'bmp>,
  pub(super) errors: RefCell<Vec<Diagnostic>>,
  pub(super) strict: bool, // todo: naming...
}

pub struct ParseResult<'bmp> {
  pub document: Document<'bmp>,
  pub warnings: Vec<Diagnostic>,
}

#[derive(Debug, Default)]
pub(crate) struct ListContext {
  pub(crate) stack: ListStack,
  pub(crate) parsing_continuations: bool,
}

impl<'bmp, 'src> Parser<'bmp, 'src> {
  pub fn new(bump: &'bmp Bump, src: &'src str) -> Parser<'bmp, 'src> {
    Parser {
      bump,
      lexer: Lexer::new(src),
      document: Document::new(bump),
      peeked_lines: None,
      peeked_meta: None,
      ctx: ParseContext::new(bump),
      errors: RefCell::new(Vec::new()),
      strict: true,
    }
  }

  pub fn new_opts(bump: &'bmp Bump, src: &'src str, opts: opts::Opts) -> Parser<'bmp, 'src> {
    let mut p = Parser::new(bump, src);
    p.strict = opts.strict;
    p
  }

  pub(crate) fn debug_loc(&self, loc: SourceLocation) {
    println!("{:?}, {}", loc, self.lexer.loc_src(loc));
  }

  pub(crate) fn loc(&self) -> SourceLocation {
    self
      .peeked_lines
      .as_ref()
      .and_then(|lines| lines.loc())
      .unwrap_or_else(|| self.lexer.loc())
  }

  pub(crate) fn line_from(
    &self,
    tokens: BumpVec<'bmp, Token<'src>>,
    loc: SourceLocation,
  ) -> Line<'bmp, 'src> {
    Line::new(tokens, self.lexer.loc_src(loc))
  }

  pub(crate) fn read_line(&mut self) -> Option<Line<'bmp, 'src>> {
    self.lexer.consume_line(self.bump)
  }

  pub(crate) fn read_lines(&mut self) -> Option<ContiguousLines<'bmp, 'src>> {
    if let Some(peeked) = self.peeked_lines.take() {
      return Some(peeked);
    }
    self.lexer.consume_empty_lines();
    if self.lexer.is_eof() {
      return None;
    }
    let mut lines = BumpVec::new_in(self.bump);
    while let Some(line) = self.lexer.consume_line(self.bump) {
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
  ) -> Option<ContiguousLines<'bmp, 'src>> {
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
      // Delimiter::Table(ch) => self.lexer.at_delimiter_line() == Some((4, ch)),
    }
  }

  pub(crate) fn restore_lines(&mut self, lines: ContiguousLines<'bmp, 'src>) {
    debug_assert!(self.peeked_lines.is_none());
    if !lines.is_empty() {
      self.peeked_lines = Some(lines);
    }
  }

  pub(crate) fn restore_peeked_meta(&mut self, meta: ChunkMeta<'bmp>) {
    debug_assert!(self.peeked_meta.is_none());
    self.peeked_meta = Some(meta);
  }

  pub(crate) fn restore_peeked(
    &mut self,
    lines: ContiguousLines<'bmp, 'src>,
    meta: ChunkMeta<'bmp>,
  ) {
    self.restore_lines(lines);
    self.restore_peeked_meta(meta);
  }

  pub fn parse(mut self) -> std::result::Result<ParseResult<'bmp>, Vec<Diagnostic>> {
    self.document.header = self.parse_document_header()?;
    if let Some(DocHeader { ref attrs, .. }) = self.document.header {
      self.ctx.attrs = attrs.clone();
    }

    while let Some(chunk) = self.parse_chunk()? {
      match chunk {
        Chunk::Block(block) => self.document.content.push_block(block, self.bump),
        Chunk::Section(section) => self.document.content.push_section(section, self.bump),
      }
    }

    self.diagnose_document()?;

    Ok(ParseResult {
      document: self.document,
      warnings: vec![],
    })
  }

  fn parse_chunk(&mut self) -> Result<Option<Chunk<'bmp>>> {
    match self.parse_section()? {
      Some(section) => Ok(Some(Chunk::Section(section))),
      None => Ok(self.parse_block()?.map(Chunk::Block)),
    }
  }

  pub(crate) fn parse_chunk_meta(
    &mut self,
    lines: &mut ContiguousLines<'bmp, 'src>,
  ) -> Result<ChunkMeta<'bmp>> {
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
          title = Some(self.parse_inlines(&mut line.into_lines_in(self.bump))?);
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
    for (ref_id, ref_loc) in &self.ctx.xrefs {
      if !self.document.anchors.contains_key(ref_id) {
        self.err_at_loc(
          format!("Invalid cross reference, no anchor found for `{ref_id}`"),
          *ref_loc,
        )?;
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
pub enum Chunk<'bmp> {
  Block(Block<'bmp>),
  Section(Section<'bmp>),
}

impl From<Diagnostic> for Vec<Diagnostic> {
  fn from(diagnostic: Diagnostic) -> Self {
    vec![diagnostic]
  }
}
