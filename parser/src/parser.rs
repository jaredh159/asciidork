use std::{cell::RefCell, rc::Rc};

use crate::{include_resolver::IncludeResolver, internal::*};

// #[derive(Debug)]
pub struct Parser<'bmp: 'src, 'src> {
  pub(super) bump: &'bmp Bump,
  pub(super) lexers: BumpVec<'bmp, Lexer<'src>>,
  pub(super) lexer_idx: usize,
  pub(super) document: Document<'bmp>,
  pub(super) peeked_lines: Option<ContiguousLines<'bmp, 'src>>,
  pub(super) peeked_meta: Option<ChunkMeta<'bmp>>,
  pub(super) ctx: ParseContext<'bmp>,
  pub(super) errors: RefCell<Vec<Diagnostic>>,
  pub(super) strict: bool, // todo: naming...
  pub(super) include_resolver: Option<Box<dyn IncludeResolver>>,
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
  pub fn new(bump: &'bmp Bump, src: impl Into<AsciidocSource<'src>>) -> Parser<'bmp, 'src> {
    Parser {
      bump,
      lexers: bvec![in bump; Lexer::new(src)],
      lexer_idx: 0,
      document: Document::new(bump),
      peeked_lines: None,
      peeked_meta: None,
      ctx: ParseContext::new(bump),
      errors: RefCell::new(Vec::new()),
      strict: true,
      include_resolver: None,
    }
  }

  pub fn new_settings(
    bump: &'bmp Bump,
    src: impl Into<AsciidocSource<'src>>,
    settings: JobSettings,
  ) -> Parser<'bmp, 'src> {
    let mut p = Parser::new(bump, src);
    p.strict = settings.strict;
    p.document.meta = settings.into();
    p
  }

  pub fn set_resolver(&mut self, resolver: Box<dyn IncludeResolver>) {
    self.include_resolver = Some(resolver);
  }

  pub fn cell_parser(&mut self, src: &'src str, offset: usize) -> Parser<'bmp, 'src> {
    let mut cell_parser = Parser::new(self.bump, src);
    cell_parser.strict = self.strict;
    lexer!(cell_parser).adjust_offset(offset);
    cell_parser.ctx = self.ctx.clone_for_cell();
    cell_parser.document.meta = self.document.meta.clone_for_cell();
    cell_parser.document.anchors = Rc::clone(&self.document.anchors);
    cell_parser
  }

  pub(crate) fn debug_loc(&self, loc: SourceLocation) {
    println!("{:?}, {}", loc, lexer!(self).loc_src(loc));
  }

  pub(crate) fn loc(&self) -> SourceLocation {
    self
      .peeked_lines
      .as_ref()
      .and_then(|lines| lines.loc())
      .unwrap_or_else(|| lexer!(self).loc())
  }

  pub(crate) fn line_from(
    &self,
    tokens: BumpVec<'bmp, Token<'src>>,
    loc: impl Into<SourceLocation>,
  ) -> Line<'bmp, 'src> {
    Line::new(tokens, lexer!(self).loc_src(loc))
  }

  pub(crate) fn read_line(&mut self) -> Option<Line<'bmp, 'src>> {
    debug_assert!(self.peeked_lines.is_none());
    lexer!(self).consume_line(self.bump)
  }

  pub(crate) fn read_lines(&mut self) -> Option<ContiguousLines<'bmp, 'src>> {
    if let Some(peeked) = self.peeked_lines.take() {
      return Some(peeked);
    }
    lexer!(self).consume_empty_lines();
    if lexer!(self).is_eof() {
      return None;
    }
    let mut lines = BumpVec::new_in(self.bump);
    while let Some(line) = lexer!(self).consume_line(self.bump) {
      lines.push(line);
      if lexer!(self).peek_is(b'\n') {
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
    while !lexer!(self).is_eof() && !self.at_delimiter(delimiter) {
      additional_lines.push(self.read_line().unwrap());
    }
    lines.extend(additional_lines);
    Some(lines)
  }

  fn at_delimiter(&self, delimiter: Delimiter) -> bool {
    match delimiter {
      Delimiter::BlockQuote => lexer!(self).at_delimiter_line() == Some((4, b'_')),
      Delimiter::Example => lexer!(self).at_delimiter_line() == Some((4, b'=')),
      Delimiter::Open => lexer!(self).at_delimiter_line() == Some((2, b'-')),
      Delimiter::Sidebar => lexer!(self).at_delimiter_line() == Some((4, b'*')),
      Delimiter::Listing => lexer!(self).at_delimiter_line() == Some((4, b'-')),
      Delimiter::Literal => lexer!(self).at_delimiter_line() == Some((4, b'.')),
      Delimiter::Passthrough => lexer!(self).at_delimiter_line() == Some((4, b'+')),
      Delimiter::Comment => lexer!(self).at_delimiter_line() == Some((4, b'/')),
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
    self.parse_document_header()?;

    // ensure we only read a single "paragraph" for `inline` doc_type
    // https://docs.asciidoctor.org/asciidoc/latest/document/doctype/#inline-doctype-rules
    if self.document.meta.get_doctype() == DocType::Inline {
      if self.peeked_lines.is_none() {
        self.peeked_lines = self.read_lines();
      }
      lexer!(self).truncate();
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
pub enum Chunk<'bmp> {
  Block(Block<'bmp>),
  Section(Section<'bmp>),
}

impl From<Diagnostic> for Vec<Diagnostic> {
  fn from(diagnostic: Diagnostic) -> Self {
    vec![diagnostic]
  }
}
