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
    self.strict = settings.strict;
    self.document.meta = settings.into();
    self.set_source_file_attrs();
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
      if token.is(TokenKind::PreprocEndInclude) && line.is_empty() {
        line.push(token);
        if self.lexer.peek_boundary_is(b'\n') {
          self.lexer.skip_byte();
        }
        return Ok(Some(line));
      }
      self.push_token_replacing_attr_ref(token, &mut line, &mut drop_line)?;
    }
    if self.lexer.peek_boundary_is(b'\n') {
      self.lexer.skip_byte();
    }
    if drop_line {
      return self.read_line();
    }
    if line.starts(TokenKind::Directive) {
      let copy = line.clone();
      return match self.try_process_directive(&mut line)? {
        DirectiveAction::Passthrough => Ok(Some(copy)),
        DirectiveAction::SubstituteLine(line) => Ok(Some(line)),
        DirectiveAction::ReadNextLine => self.read_line(),
        DirectiveAction::SkipLinesUntilEndIf => todo!(),
      };
    }
    Ok(Some(line))
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
      lines.push(line);
      if self.lexer.peek_boundary_is(b'\n') {
        break;
      }
    }
    debug_assert!(!lines.is_empty());
    Ok(Some(ContiguousLines::new(lines)))
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
    if self.ctx.table_cell_ctx == TableCellContext::None {
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
  fn test_include_boundaries_no_newline_end() {
    let input = adoc! {"
      foo
      include::bar.adoc[]
    "};
    let mut parser = test_parser!(input);
    parser.apply_job_settings(JobSettings::r#unsafe());
    parser.set_resolver(resolve("bar")); // <-- no newline
    let lines = parser.read_lines().unwrap().unwrap();
    assert_eq!(
      reassemble(lines),
      adoc! {"
        foo
        {->00001}bar.adoc[]bar
        {<-00001}bar.adoc[]"
      }
    );
    assert!(parser.read_lines().unwrap().is_none());
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
        Token::new(TokenKind::Word, 13..25, bstr!("include-file")),
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
  fn uri_read_not_allowed_include() {
    // strict mode error
    let input = "include::https://my.com/foo.adoc[]";
    let mut parser = test_parser!(input);
    parser.apply_job_settings(JobSettings::r#unsafe());
    let err = parser.read_line().err().unwrap();
    let expected_err = error! {"
      1: include::https://my.com/foo.adoc[]
                  ^^^^^^^^^^^^^^^^^^^^^^^ Cannot include URL contents (allow-uri-read not enabled)
    "};
    expect_eq!(err.plain_text(), expected_err, from: input);

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

  #[test]
  fn include_resolver_error_uri_read_not_supported() {
    let mut parser = test_parser!("include::http://a.com/b[]");
    let mut settings = JobSettings::r#unsafe();
    settings
      .job_attrs
      .insert_unchecked("allow-uri-read", asciidork_meta::JobAttr::readonly(true));
    parser.apply_job_settings(settings);
    parser.set_resolver(Box::new(ErrorResolver(ResolveError::UriReadNotSupported)));
    let expected_err = error! {"
      1: include::http://a.com/b[]
                  ^^^^^^^^^^^^^^ Include resolver error: URI read not supported
    "};
    expect_eq!(parser.read_line().err().unwrap().plain_text(), expected_err);
  }

  #[test]
  fn include_resolver_error_no_resolver() {
    let mut parser = test_parser!("include::file.adoc[]");
    parser.apply_job_settings(JobSettings::r#unsafe());
    let expected_err = error! {"
      1: include::file.adoc[]
         ^^^^^^^^^ No resolver supplied for include directive
    "};
    expect_eq!(parser.read_line().err().unwrap().plain_text(), expected_err);
  }

  #[test]
  fn include_resolver_error_bad_encoding() {
    let mut parser = test_parser!("include::file.adoc[]");
    parser.apply_job_settings(JobSettings::r#unsafe());
    let invalid_utf8 = vec![0xFF, 0xFE, 0x68, 0x00, 0xFF, 0xDC];
    parser.set_resolver(Box::new(ConstResolver(invalid_utf8)));
    let expected_err = error! {"
      1: include::file.adoc[]
                  ^^^^^^^^^ Error resolving file contents: Invalid UTF-16 (LE)
    "};
    expect_eq!(parser.read_line().err().unwrap().plain_text(), expected_err);
  }

  #[test]
  fn include_resolver_gets_passed_correct_target() {
    struct AssertResolver(&'static str);
    impl IncludeResolver for AssertResolver {
      fn resolve(
        &mut self,
        target: IncludeTarget,
        _: &mut dyn IncludeBuffer,
      ) -> std::result::Result<usize, ResolveError> {
        assert_eq!(target, IncludeTarget::FilePath(self.0.to_string()));
        Ok(0)
      }
      fn get_base_dir(&self) -> Option<String> {
        Some("".to_string())
      }
    }
    let cases = [
      ("include::spaced file.adoc[]", "spaced file.adoc"),
      ("include::with{sp}attr.adoc[]", "with attr.adoc"),
      (":myfile: foo.adoc\n\ninclude::{myfile}[]", "foo.adoc"),
    ];
    for (input, expected) in cases {
      let mut parser = test_parser!(input);
      parser.apply_job_settings(JobSettings::r#unsafe());
      parser.set_resolver(Box::new(AssertResolver(expected)));
      assert!(parser.parse().is_ok());
    }
  }
}
