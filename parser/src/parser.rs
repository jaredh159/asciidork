use std::fmt::Debug;
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
  pub(super) attr_locs: Vec<(SourceLocation, bool)>,
  pub(super) include_resolver: Option<Box<dyn IncludeResolver>>,
  #[cfg(feature = "attr_ref_observation")]
  pub(super) attr_ref_observer: Option<Box<dyn AttrRefObserver>>,
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
      attr_locs: Vec::with_capacity(16),
      #[cfg(feature = "attr_ref_observation")]
      attr_ref_observer: None,
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

  #[cfg(feature = "attr_ref_observation")]
  pub fn set_attr_ref_observer(&mut self, observer: Box<dyn AttrRefObserver>) {
    self.attr_ref_observer = Some(observer);
  }

  pub fn cell_parser(&mut self, src: BumpVec<'arena, u8>, offset: u32) -> Parser<'arena> {
    let mut cell_parser = Parser::new(src, self.lexer.source_file().clone(), self.bump);
    cell_parser.include_resolver = self.include_resolver.as_ref().map(|r| r.clone_box());
    cell_parser.strict = self.strict;
    cell_parser.lexer.adjust_offset(offset);
    cell_parser.ctx = self.ctx.clone_for_cell(self.bump);
    cell_parser.document.meta = self.document.meta.clone_for_cell();
    cell_parser.document.anchors = Rc::clone(&self.document.anchors);

    #[cfg(feature = "attr_ref_observation")]
    {
      cell_parser.attr_ref_observer = self.attr_ref_observer.take();
    }

    cell_parser
  }

  pub(crate) fn loc(&self) -> SourceLocation {
    self
      .peeked_lines
      .as_ref()
      .and_then(|lines| lines.first_loc())
      .unwrap_or_else(|| self.lexer.loc())
  }

  pub(crate) fn read_line(&mut self) -> Result<Option<Line<'arena>>> {
    Ok(self._read_line(false)?.map(|(line, _)| line))
  }

  fn _read_line(&mut self, ignored_last: bool) -> Result<Option<(Line<'arena>, bool)>> {
    assert!(self.peeked_lines.is_none());
    if self.lexer.is_eof() {
      return Ok(None);
    }

    let mut drop_line = false;
    let mut line = Line::empty(self.bump);
    while !self.lexer.at_newline() && !self.lexer.is_eof() {
      let token = self.lexer.next_token();
      self.push_token_replacing_attr_ref(token, &mut line, &mut drop_line)?;
    }
    self.lexer.skip_newline();
    if drop_line {
      return self._read_line(false);
    }
    if line.starts(TokenKind::Directive) && !self.ctx.within_block_comment() {
      match self.try_process_directive(&mut line)? {
        DirectiveAction::Passthrough => Ok(Some((line, ignored_last))),
        DirectiveAction::SubstituteLine(line) => Ok(Some((line, ignored_last))),
        DirectiveAction::IgnoreNotIncluded => self._read_line(true),
        DirectiveAction::ReadNextLine => self._read_line(false),
        DirectiveAction::SkipLinesUntilEndIf => Ok(
          self
            .skip_lines_until_endif(&line)?
            .map(|l| (l, ignored_last)),
        ),
      }
    } else {
      Ok(Some((line, ignored_last)))
    }
  }

  pub(crate) fn read_lines(&mut self) -> Result<Option<ContiguousLines<'arena>>> {
    self.ctx.comment_delim_in_lines = false;
    if let Some(peeked) = self.peeked_lines.take() {
      if peeked.iter().all(|line| line.is_empty()) {
        return Ok(None);
      } else {
        return Ok(Some(peeked));
      }
    }
    self.lexer.consume_empty_lines();
    if self.lexer.is_eof() {
      return Ok(None);
    }
    let mut lines = Deq::new(self.bump);
    while let Some((line, ignored_removed_include_line)) = self._read_line(false)? {
      if line.is_emptyish() {
        if lines.is_empty() {
          // this case can happen if our first non-empty line was an include directive
          // that then resolved to an initial empty line, otherwise consume_empty_lines
          // would have skipped over it, so we keep going
          continue;
        } else if !ignored_removed_include_line {
          // this case can happen if our first non-empty line was an include directive
          // this case happens only when we DROP a line
          break;
        }
      }
      if line.is_delimiter_kind(DelimiterKind::Comment) {
        self.ctx.comment_delim_in_lines = true;
      }
      lines.push(line);
      if self.lexer.at_newline() {
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

    while lines.last().map(|l| l.is_empty()) == Some(true) {
      lines.pop();
    }
    Ok(Some(lines))
  }

  fn at_delimiter(&self, delimiter: Delimiter) -> bool {
    match delimiter.kind {
      DelimiterKind::BlockQuote => self.lexer.at_delimiter_line() == Some((4, b'_')),
      DelimiterKind::Example => {
        self.lexer.at_delimiter_line() == Some((delimiter.len as u32, b'='))
      }
      DelimiterKind::Open => self.lexer.at_delimiter_line() == Some((2, b'-')),
      DelimiterKind::Sidebar => self.lexer.at_delimiter_line() == Some((4, b'*')),
      DelimiterKind::Listing => self.lexer.at_delimiter_line() == Some((4, b'-')),
      DelimiterKind::Literal => self.lexer.at_delimiter_line() == Some((4, b'.')),
      DelimiterKind::Passthrough => self.lexer.at_delimiter_line() == Some((4, b'+')),
      DelimiterKind::Comment => self.lexer.at_delimiter_line() == Some((4, b'/')),
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
    let doc_attrs = self.document.meta.doc_attrs_snapshot();
    self.parse_document_header()?;
    self.prepare_toc();

    // ensure we only read a single "paragraph" for `inline` doc_type
    // https://docs.asciidoctor.org/asciidoc/latest/document/doctype/#inline-doctype-rules
    if self.document.meta.get_doctype() == DocType::Inline {
      if self.peeked_lines.is_none() {
        // tmp:
        self.peeked_lines = self.read_lines().expect("tmp");
      }
      self.lexer.truncate();
    }

    if let Some(book_content) = self.parse_book()? {
      self.document.content = book_content;
    } else {
      let sectioned = self.parse_sectioned()?;
      self.document.content = sectioned.into_doc_content(self.bump);
    }

    // so the backend can see them replayed in decl order
    self.document.meta.restore_doc_attrs(doc_attrs);
    self.diagnose_document()?;
    Ok(self.into())
  }

  pub(crate) fn parse_sectioned(&mut self) -> Result<Sectioned<'arena>> {
    let mut blocks = bvec![in self.bump];
    while let Some(block) = self.parse_block()? {
      blocks.push(block);
    }
    let preamble = if blocks.is_empty() { None } else { Some(blocks) };
    let mut sections = bvec![in self.bump];
    while let Some(section) = self.parse_section()? {
      sections.push(section);
    }
    Ok(Sectioned { preamble, sections })
  }

  pub(crate) fn parse_chunk_meta(
    &mut self,
    lines: &mut ContiguousLines<'arena>,
  ) -> Result<ChunkMeta<'arena>> {
    if let Some(meta) = self.peeked_meta.take() {
      return Ok(meta);
    }
    assert!(!lines.is_empty());
    let start_loc = lines.current_token().unwrap().loc;
    let mut attrs = MultiAttrList::new_in(self.bump);
    let mut title = None;
    if !lines.current().unwrap().is_fully_unconsumed() {
      return Ok(ChunkMeta::new(attrs, title, start_loc));
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
          let first = line.discard_assert(TokenKind::OpenBracket);
          line.discard_assert(TokenKind::OpenBracket);
          let Some(anchor) = self.parse_block_anchor(&mut line)? else {
            self.err_line_starting("Invalid block anchor", first.loc)?;
            return Ok(ChunkMeta::new(attrs, title, start_loc));
          };
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
    Ok(ChunkMeta::new(attrs, title, start_loc))
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

pub enum DirectiveAction<'arena> {
  Passthrough,
  ReadNextLine,
  IgnoreNotIncluded,
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
    #[derive(Clone)]
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
      fn clone_box(&self) -> Box<dyn IncludeResolver> {
        Box::new(self.clone())
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
        Token::new(TokenKind::Word, loc!(0..5), bstr!("hello")),
        Token::new(TokenKind::Whitespace, loc!(5..6), bstr!(" ")),
        Token::new(TokenKind::AttrRef, loc!(6..11), bstr!("{foo}")),
        // these are inserted as an inline preprocessing step
        // NB: we will use the source loc of the attr ref token to know how
        // to skip over the resolve attribute in no-attr-ref subs contexts
        Token::new(TokenKind::Underscore, loc!(6..11), bstr!("_")),
        Token::new(TokenKind::Word, loc!(6..11), bstr!("bar")),
        Token::new(TokenKind::Underscore, loc!(6..11), bstr!("_")),
        // end inserted.
        Token::new(TokenKind::Whitespace, loc!(11..12), bstr!(" ")),
        Token::new(TokenKind::Word, loc!(12..17), bstr!("world")),
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
      Token::new(TokenKind::Word, loc!(0..3), bstr!("foo"))
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
        Token::new(TokenKind::MacroName, loc!(8..13), bstr!("link:")),
        Token::new(TokenKind::Word, loc!(13..20), bstr!("include")),
        Token::new(TokenKind::Dashes, loc!(20..21), bstr!("-")),
        Token::new(TokenKind::Word, loc!(21..25), bstr!("file")),
        Token::new(TokenKind::Dots, loc!(25..26), bstr!(".")),
        Token::new(TokenKind::Word, loc!(26..30), bstr!("adoc")),
        Token::new(TokenKind::OpenBracket, loc!(30..31), bstr!("[")),
        // these tokens are inserted, they have no true source so we
        // represent their position as empty at the insertion point
        Token::new(TokenKind::Word, loc!(31..31), bstr!("role")),
        Token::new(TokenKind::EqualSigns, loc!(31..31), bstr!("=")),
        Token::new(TokenKind::Word, loc!(31..31), bstr!("include")),
        Token::new(TokenKind::Comma, loc!(31..31), bstr!(",")),
        // /end `role=include` inserted tokens
        Token::new(TokenKind::CloseBracket, loc!(31..32), bstr!("]")),
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
