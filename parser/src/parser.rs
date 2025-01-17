use std::{cell::RefCell, rc::Rc};

use crate::internal::*;

pub struct Parser<'arena> {
  pub(super) bump: &'arena Bump,
  pub(super) lexer: Lexer<'arena>,
  pub(super) document: Document<'arena>,
  pub(super) peeked_lines: Option<ContiguousLines<'arena>>,
  pub(super) peeked_meta: Option<ChunkMeta<'arena>>,
  pub(super) ctx: ParseContext<'arena>,
  pub(super) errors: RefCell<Vec<Diagnostic>>,
  pub(super) strict: bool, // todo: naming...
  pub(super) include_resolver: Option<Box<dyn IncludeResolver>>,
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
  pub fn new(src: BumpVec<'arena, u8>, file: SourceFile, bump: &'arena Bump) -> Self {
    Parser::from_lexer(Lexer::new(src, file, bump))
  }

  pub fn from_str(src: &str, file: SourceFile, bump: &'arena Bump) -> Self {
    Parser::from_lexer(Lexer::from_str(bump, file, src))
  }

  fn from_lexer(lexer: Lexer<'arena>) -> Self {
    let mut parser = Parser {
      bump: lexer.bump,
      document: Document::new(lexer.bump),
      peeked_lines: None,
      peeked_meta: None,
      ctx: ParseContext::new(lexer.bump),
      errors: RefCell::new(Vec::new()),
      strict: true,
      include_resolver: None,
      lexer,
    };
    parser.set_source_file_attrs();
    parser
  }

  pub fn apply_job_settings(&mut self, settings: JobSettings) {
    if let Some(leveloffset) = settings.job_attrs.get("leveloffset") {
      Parser::adjust_leveloffset(&mut self.ctx.leveloffset, &leveloffset.value);
    }
    self.strict = settings.strict;
    self.ctx.max_include_depth = settings.job_attrs.u16("max-include-depth").unwrap_or(64);
    self.document.meta = settings.into();
    self.set_source_file_attrs();
  }

  pub fn provide_timestamps(
    &mut self,
    now: u64,
    input_modified_time: Option<u64>,
    reproducible_override: Option<u64>,
  ) {
    self.set_datetime_attrs(now, input_modified_time, reproducible_override);
  }

  pub fn set_resolver(&mut self, resolver: Box<dyn IncludeResolver>) {
    self.include_resolver = Some(resolver);
  }

  pub fn cell_parser(&mut self, src: BumpVec<'arena, u8>, offset: u32) -> Parser<'arena> {
    let mut cell_parser = Parser::new(src, self.lexer.source_file().clone(), self.bump);
    cell_parser.strict = self.strict;
    cell_parser.lexer.adjust_offset(offset);
    cell_parser.ctx = self.ctx.clone_for_cell(self.bump);
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

  pub(crate) fn read_line(&mut self) -> Result<Option<Line<'arena>>> {
    assert!(self.peeked_lines.is_none());
    if self.lexer.is_eof() {
      return Ok(None);
    }

    let mut drop_line = false;
    let mut line = Line::empty(self.bump);
    while !self.lexer.peek_is(b'\n') && !self.lexer.is_eof() {
      let token = self.lexer.next_token();
      self.push_token_replacing_attr_ref(token, &mut line, &mut drop_line)?;
    }
    if self.lexer.peek_is(b'\n') {
      self.lexer.skip_byte();
    }
    if drop_line {
      return self.read_line();
    }
    if line.starts(TokenKind::Directive) {
      match self.try_process_directive(&mut line)? {
        DirectiveAction::Passthrough => Ok(Some(line)),
        DirectiveAction::SubstituteLine(line) => Ok(Some(line)),
        DirectiveAction::ReadNextLine => self.read_line(),
        DirectiveAction::SkipLinesUntilEndIf => self.skip_lines_until_endif(&line),
      }
    } else {
      Ok(Some(line))
    }
  }

  pub(crate) fn read_lines(&mut self) -> Result<Option<ContiguousLines<'arena>>> {
    if let Some(peeked) = self.peeked_lines.take() {
      return Ok(Some(peeked));
    }
    self.lexer.consume_empty_lines();
    if self.lexer.is_eof() {
      return Ok(None);
    }
    let mut lines = Deq::new(self.bump);
    while let Some(line) = self.read_line()? {
      if line.is_empty() {
        if lines.is_empty() {
          // this case can happen if our first non-empty line was an include directive
          // that then resolved to an initial empty line, otherwise consume_empty_lines
          // would have skipped over it, so we keep going
          continue;
        } else {
          // this case happens only when we DROP a line
          break;
        }
      }
      lines.push(line);
      if self.lexer.peek_is(b'\n') {
        break;
      }
    }
    if lines.is_empty() {
      Ok(None)
    } else {
      Ok(Some(ContiguousLines::new(lines)))
    }
  }

  pub(crate) fn read_lines_until(
    &mut self,
    delimiter: Delimiter,
  ) -> Result<Option<ContiguousLines<'arena>>> {
    let Some(mut lines) = self.read_lines()? else {
      return Ok(None);
    };
    if lines.any(|l| l.is_delimiter(delimiter)) {
      return Ok(Some(lines));
    }

    let mut additional_lines = BumpVec::new_in(self.bump);
    while !self.lexer.is_eof() && !self.at_delimiter(delimiter) {
      additional_lines.push(self.read_line()?.unwrap());
    }
    lines.extend(additional_lines);
    Ok(Some(lines))
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
    if !meta.is_empty() {
      debug_assert!(self.peeked_meta.is_none());
      self.peeked_meta = Some(meta);
    }
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
        // tmp:
        self.peeked_lines = self.read_lines().expect("tmep");
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
    assert!(!lines.is_empty());
    let start = lines.current_token().unwrap().loc.start;
    let mut attrs = MultiAttrList::new_in(self.bump);
    let mut title = None;
    if !lines.current().unwrap().is_fully_unconsumed() {
      return Ok(ChunkMeta { attrs, title, start });
    }
    loop {
      match lines.current() {
        Some(line) if line.is_chunk_title() => {
          let mut line = lines.consume_current().unwrap();
          line.discard_assert(TokenKind::Dots);
          title = Some(self.parse_inlines(&mut line.into_lines())?);
        }
        Some(line) if line.is_block_attr_list() => {
          let mut line = lines.consume_current().unwrap();
          line.discard_assert(TokenKind::OpenBracket);
          attrs.push(self.parse_block_attr_list(&mut line)?);
        }
        Some(line) if line.is_block_anchor() => {
          let mut line = lines.consume_current().unwrap();
          line.discard_assert(TokenKind::OpenBracket);
          line.discard_assert(TokenKind::OpenBracket);
          let anchor = self.parse_block_anchor(&mut line)?.unwrap();
          let mut anchor_attrs = AttrList::new(anchor.loc, self.bump);
          anchor_attrs.id = Some(anchor.id);
          anchor_attrs.positional.push(anchor.reftext);
          attrs.push(anchor_attrs);
        }
        // consume trailing comment lines for valid meta
        Some(line) if line.is_comment() && (!attrs.is_empty() || title.is_some()) => {
          lines.consume_current();
        }
        _ => break,
      }
    }
    Ok(ChunkMeta { attrs, title, start })
  }

  pub(crate) fn string(&self, s: &str) -> BumpString<'arena> {
    BumpString::from_str_in(s, self.bump)
  }
}

pub trait HasArena<'arena> {
  fn bump(&self) -> &'arena Bump;
  fn token(&self, kind: TokenKind, lexeme: &str, loc: SourceLocation) -> Token<'arena> {
    Token::new(kind, loc, BumpString::from_str_in(lexeme, self.bump()))
  }
}

impl<'arena> HasArena<'arena> for Parser<'arena> {
  fn bump(&self) -> &'arena Bump {
    self.bump
  }
}

#[derive(Debug)]
pub enum Chunk<'arena> {
  Block(Block<'arena>),
  Section(Section<'arena>),
}

pub enum DirectiveAction<'arena> {
  Passthrough,
  ReadNextLine,
  SkipLinesUntilEndIf,
  SubstituteLine(Line<'arena>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceFile {
  Stdin { cwd: Path },
  Path(Path),
  Tmp,
}

impl SourceFile {
  pub fn file_name(&self) -> &str {
    match self {
      SourceFile::Stdin { .. } => "<stdin>",
      SourceFile::Path(path) => path.file_name(),
      SourceFile::Tmp => "<temp-buffer>",
    }
  }

  pub fn matches_xref_target(&self, target: &str) -> bool {
    let SourceFile::Path(path) = self else {
      return false;
    };
    let filename = path.file_name();
    if filename == target {
      return true;
    }
    let xref_ext = file::ext(target);
    let path_ext = file::ext(filename);
    if xref_ext.is_some() && xref_ext != path_ext {
      return false;
    }
    let fullpath = path.to_string();
    if fullpath.ends_with(target) {
      true
    } else if xref_ext.is_some() {
      false
    } else {
      file::remove_ext(&fullpath).ends_with(target)
    }
  }
}

impl From<Diagnostic> for Vec<Diagnostic> {
  fn from(diagnostic: Diagnostic) -> Self {
    vec![diagnostic]
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use test_utils::*;

  fn resolve(src: &'static str) -> Box<dyn IncludeResolver> {
    struct MockResolver(pub Vec<u8>);
    impl IncludeResolver for MockResolver {
      fn resolve(
        &mut self,
        _: IncludeTarget,
        buffer: &mut dyn IncludeBuffer,
      ) -> std::result::Result<usize, ResolveError> {
        buffer.initialize(self.0.len());
        let bytes = buffer.as_bytes_mut();
        bytes.copy_from_slice(&self.0);
        Ok(self.0.len())
      }
      fn get_base_dir(&self) -> Option<String> {
        Some("/".to_string())
      }
    }
    Box::new(MockResolver(Vec::from(src.as_bytes())))
  }

  fn reassemble(lines: ContiguousLines) -> String {
    lines
      .iter()
      .map(|l| l.reassemble_src())
      .collect::<Vec<_>>()
      .join("\n")
  }

  #[test]
  fn test_attr_ref() {
    let mut parser = test_parser!("hello {foo} world");
    parser
      .document
      .meta
      .insert_doc_attr("foo", "_bar_")
      .unwrap();
    let mut lines = parser.read_lines().unwrap().unwrap();
    let line = lines.consume_current().unwrap();
    let tokens = line.into_iter().collect::<Vec<_>>();
    expect_eq!(
      &tokens,
      &[
        Token::new(TokenKind::Word, 0..5, bstr!("hello")),
        Token::new(TokenKind::Whitespace, 5..6, bstr!(" ")),
        Token::new(TokenKind::AttrRef, 6..11, bstr!("{foo}")),
        // these are inserted as an inline preprocessing step
        // NB: we will use the source loc of the attr ref token to know how
        // to skip over the resolve attribute in no-attr-ref subs contexts
        Token::new(TokenKind::Underscore, 6..11, bstr!("_")),
        Token::new(TokenKind::Word, 6..11, bstr!("bar")),
        Token::new(TokenKind::Underscore, 6..11, bstr!("_")),
        // end inserted.
        Token::new(TokenKind::Whitespace, 11..12, bstr!(" ")),
        Token::new(TokenKind::Word, 12..17, bstr!("world")),
      ]
    );
  }

  #[test]
  fn invalid_directive_line_passed_thru() {
    let input = adoc! {"
      foo
      include::invalid []
      bar
    "};

    let mut parser = test_parser!(input);
    assert_eq!(
      reassemble(parser.read_lines().unwrap().unwrap()),
      input.trim_end()
    );
  }

  #[test]
  fn safe_mode_include_to_link() {
    let input = adoc! {"
      foo
      include::include-file.adoc[]
      baz
    "};

    let mut parser = test_parser!(input);
    parser.apply_job_settings(JobSettings::secure());
    assert_eq!(
      reassemble(parser.read_lines().unwrap().unwrap()),
      adoc! {"
        foo
        link:include-file.adoc[role=include,]
        baz"
      }
    );

    // assert on the tokens and positions
    let mut parser = test_parser!(input);
    parser.apply_job_settings(JobSettings::secure());

    let mut line = parser.read_line().unwrap().unwrap();
    expect_eq!(
      line.consume_current().unwrap(),
      Token::new(TokenKind::Word, 0..3, bstr!("foo"))
    );
    assert!(line.consume_current().is_none());

    assert_eq!(&input[8..13], "ude::");
    assert_eq!(&input[30..32], "[]");

    let mut line = parser.read_line().unwrap().unwrap();
    expect_eq!(
      std::array::from_fn(|_| line.consume_current().unwrap()),
      [
        // we "drop" positions 4-7, the `inc` of `include::`
        // which becomes `••••link:`, keeping rest of token positions
        Token::new(TokenKind::MacroName, 8..13, bstr!("link:")),
        Token::new(TokenKind::Word, 13..20, bstr!("include")),
        Token::new(TokenKind::Dashes, 20..21, bstr!("-")),
        Token::new(TokenKind::Word, 21..25, bstr!("file")),
        Token::new(TokenKind::Dots, 25..26, bstr!(".")),
        Token::new(TokenKind::Word, 26..30, bstr!("adoc")),
        Token::new(TokenKind::OpenBracket, 30..31, bstr!("[")),
        // these tokens are inserted, they have no true source so we
        // represent their position as empty at the insertion point
        Token::new(TokenKind::Word, 31..31, bstr!("role")),
        Token::new(TokenKind::EqualSigns, 31..31, bstr!("=")),
        Token::new(TokenKind::Word, 31..31, bstr!("include")),
        Token::new(TokenKind::Comma, 31..31, bstr!(",")),
        // /end `role=include` inserted tokens
        Token::new(TokenKind::CloseBracket, 31..32, bstr!("]")),
      ]
    );
    assert!(line.consume_current().is_none());
  }

  #[test]
  fn attrs_preserved_when_replacing_include() {
    let input = "include::some-file.adoc[leveloffset+=1]";
    let mut parser = test_parser!(input);
    parser.apply_job_settings(JobSettings::secure());
    assert_eq!(
      parser.read_line().unwrap().unwrap().reassemble_src(),
      "link:some-file.adoc[role=include,leveloffset+=1]"
    );
  }

  #[test]
  fn spaces_in_include_file_to_pass_macro_link() {
    let input = "include::foo bar baz.adoc[]";
    let mut parser = test_parser!(input);
    parser.apply_job_settings(JobSettings::secure());
    assert_eq!(
      parser.read_line().unwrap().unwrap().reassemble_src(),
      "link:pass:c[foo bar baz.adoc][role=include,]"
    );
  }

  #[test]
  fn uri_read_not_allowed_include_non_strict() {
    // non-strict mode replaced with link
    let input = "include::https://my.com/foo bar.adoc[]";
    let mut parser = test_parser!(input);
    let mut settings = JobSettings::r#unsafe();
    settings.strict = false;
    parser.apply_job_settings(settings);
    expect_eq!(
      parser.read_line().unwrap().unwrap().reassemble_src(),
      "link:pass:c[https://my.com/foo bar.adoc][role=include,]",
      from: input
    );
  }
}
