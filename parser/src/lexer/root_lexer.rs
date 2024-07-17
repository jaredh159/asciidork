use super::source_lexer::SourceLexer;
use crate::internal::*;

#[derive(Debug)]
pub struct RootLexer<'arena> {
  bump: &'arena Bump,
  idx: u16,
  next_idx: Option<u16>,
  source_stack: Vec<SourceLocation>,
  sources: BumpVec<'arena, SourceLexer<'arena>>,
}

impl<'arena> RootLexer<'arena> {
  pub fn new(src: BumpVec<'arena, u8>, bump: &'arena Bump) -> Self {
    Self {
      bump,
      idx: 0,
      next_idx: None,
      source_stack: Vec::new(),
      sources: bvec![in bump; SourceLexer::new(src, bump)],
    }
  }

  pub fn from_str(bump: &'arena Bump, src: &str) -> Self {
    Self {
      bump,
      idx: 0,
      next_idx: None,
      source_stack: Vec::new(),
      sources: bvec![in bump; SourceLexer::from_str(src, bump)],
    }
  }

  pub fn from_byte_slice(bytes: &[u8], bump: &'arena Bump) -> Self {
    Self {
      bump,
      idx: 0,
      next_idx: None,
      source_stack: Vec::new(),
      sources: bvec![in bump; SourceLexer::from_byte_slice(bytes, bump)],
    }
  }

  pub fn push_source(&mut self, _filename: &str, mut src: BumpVec<'arena, u8>) {
    // match asciidoctor - its include processor returns an array of lines
    // so even if the source has no trailing newline, it's inserted as a full line
    // NB: the include resolver has taken care of reserving space for the newline
    if src.last() != Some(&b'\n') {
      src.push(b'\n');
    }
    self.sources.push(SourceLexer::new(src, self.bump));
    let next_idx = self.sources.len() as u16 - 1;
    self.next_idx = Some(next_idx);
  }

  pub fn adjust_offset(&mut self, offset_adjustment: u32) {
    self.sources[self.idx as usize].offset = offset_adjustment;
  }

  pub fn peek(&self) -> Option<u8> {
    // case: we're about to switch to a new source
    if self.next_idx.is_some() {
      return Some(b'{'); // fake peek the generated boundary start include-token
    }
    // case: normal path: we're peeking at the current source, and have bytes
    if let Some(c) = self.sources[self.idx as usize].peek() {
      return Some(c);
    }
    // case: we're out of bytes, check if we're returning to a previous source
    if !self.source_stack.is_empty() {
      return Some(b'{'); // fake peek the generated boundary end include-token
    }
    // case: nothing left in any source, root EOF
    None
  }

  pub fn consume_empty_lines(&mut self) {
    self.sources[self.idx as usize].consume_empty_lines();
  }

  pub fn raw_lines(&'arena self) -> impl Iterator<Item = &'arena str> {
    self.sources[self.idx as usize].raw_lines()
  }

  pub fn loc(&self) -> SourceLocation {
    SourceLocation::from(self.sources[self.idx as usize].pos)
  }

  pub fn at_delimiter_line(&self) -> Option<(u32, u8)> {
    self.sources[self.idx as usize].at_delimiter_line()
  }

  pub fn is_eof(&self) -> bool {
    self.peek().is_none()
  }

  pub fn peek_is(&self, c: u8) -> bool {
    self.peek() == Some(c)
  }

  pub fn line_of(&self, location: u32) -> BumpString<'arena> {
    self.sources[self.idx as usize].line_of(location)
  }

  pub fn line_number(&self, location: u32) -> u32 {
    let (line_number, _) = self.line_number_with_offset(location);
    line_number
  }

  pub fn line_number_with_offset(&self, location: u32) -> (u32, u32) {
    self.sources[self.idx as usize].line_number_with_offset(location)
  }

  pub fn consume_line(&mut self) -> Option<Line<'arena>> {
    if self.is_eof() {
      return None;
    }
    let mut tokens = Deq::new(self.bump);
    while !self.peek_is(b'\n') && !self.is_eof() {
      let token = self.next_token();
      tokens.push(token);
    }
    if self.peek_is(b'\n') {
      self.sources[self.idx as usize].pos += 1;
    }
    Some(Line::new(tokens))
  }

  pub fn next_token(&mut self) -> Token<'arena> {
    match self.next_idx.take() {
      Some(next_idx) => {
        let mut include_loc = self.loc().decr();
        let mut line = self.line_of(include_loc.start);
        line.replace_range(0..9, &format!("{{->{:05}}}", next_idx));
        include_loc.start -= line.len() as u32;
        include_loc.include_depth = self.idx;
        self.source_stack.push(include_loc);
        self.idx = next_idx;
        Token::new(TokenKind::BeginInclude, include_loc, line)
      }
      None => match self.sources[self.idx as usize].next_token() {
        Some(mut token) => {
          token.loc.include_depth = self.idx;
          token
        }
        None => {
          let Some(prev_loc) = self.source_stack.pop() else {
            let empty = BumpString::from_str_in("", self.bump);
            return Token::new(TokenKind::Eof, self.loc(), empty);
          };
          let prev_idx = self.idx;
          self.idx = prev_loc.include_depth;
          let mut line = self.line_of(prev_loc.start);
          line.replace_range(0..9, &format!("{{<-{:05}}}", prev_idx));
          Token::new(TokenKind::EndInclude, prev_loc, line)
        }
      },
    }
  }

  pub fn truncate(&mut self) {
    self.sources[self.idx as usize].truncate();
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::token::TokenKind;
  use crate::variants::token::*;
  use ast::SourceLocation;
  use test_utils::*;

  macro_rules! assert_token_cases {
    ($cases:expr) => {{
      let bump = &Bump::new();
      for (input, expected) in $cases {
        let mut lexer = RootLexer::from_str(bump, input);
        let mut index = 0;
        for (token_type, lexeme) in expected {
          let start = index;
          let end = start + lexeme.len();
          let expected_token = Token {
            kind: token_type,
            loc: SourceLocation::from(start..end),
            lexeme: BumpString::from_str_in(lexeme, bump),
          };
          assert_eq!(lexer.next_token(), expected_token);
          index = end;
        }
        assert_eq!(lexer.next_token().kind, Eof);
      }
    }};
  }

  macro_rules! refute_produces_token {
    ($kind:ident, $cases:expr) => {{
      for input in $cases {
        let bump = &Bump::new();
        let mut lexer = RootLexer::from_str(bump, input);
        loop {
          match lexer.next_token().kind {
            Eof => {
              assert_eq!(true, true);
              break;
            }
            $kind => panic!("unexpected TokenKind::{:?} in input `{input}`", $kind),
            _ => {}
          }
        }
      }
    }};
  }

  #[test]
  fn test_include_boundaries() {
    let input = adoc! {"
      foo
      include::bar.adoc[]
      baz
    "};
    let bump = &Bump::new();
    let mut lexer = RootLexer::from_str(bump, input);

    // parse up to the end of the include directive
    (0..7).for_each(|_| _ = lexer.next_token());
    assert_eq!(
      lexer.next_token(),
      Token::new(CloseBracket, 22..23, bstr!("]"))
    );
    assert_eq!(lexer.next_token(), Token::new(Newline, 23..24, bstr!("\n")));

    // now mimic the parser resolving the include directive with `b"bar\n"`
    lexer.push_source("bar.adoc", bvec![in bump; b'b', b'a', b'r', b'\n']);
    assert_eq!(
      lexer.next_token(),
      Token::new(BeginInclude, 4..23, bstr!("{->00001}bar.adoc[]"))
    );
    assert_eq!(&input[4..23], "include::bar.adoc[]");

    assert_eq!(
      lexer.next_token(),
      Token::new(Word, SourceLocation::new_depth(0, 3, 1), bstr!("bar"))
    );
    assert_eq!(
      lexer.next_token(),
      Token::new(Newline, SourceLocation::new_depth(3, 4, 1), bstr!("\n"))
    );
    assert_eq!(
      lexer.next_token(),
      Token::new(EndInclude, 4..23, bstr!("{<-00001}bar.adoc[]"))
    );
  }

  #[test]
  fn test_consume_line() {
    let bump = &Bump::new();
    let mut lexer = RootLexer::from_str(bump, "foo bar\nso baz\n");
    assert_eq!(lexer.consume_line().unwrap().reassemble_src(), "foo bar");
    assert_eq!(lexer.consume_line().unwrap().reassemble_src(), "so baz");
    assert!(lexer.consume_line().is_none());
  }

  #[test]
  fn test_tokens() {
    let cases = vec![
      ("|===", vec![(Pipe, "|"), (EqualSigns, "===")]),
      ("////", vec![(DelimiterLine, "////")]),
      ("<.>", vec![(CalloutNumber, "<.>")]),
      ("<1>", vec![(CalloutNumber, "<1>")]),
      ("<255>", vec![(CalloutNumber, "<255>")]),
      ("<!--.-->", vec![(CalloutNumber, "<!--.-->")]),
      ("<!--2-->", vec![(CalloutNumber, "<!--2-->")]),
      ("<!--255-->", vec![(CalloutNumber, "<!--255-->")]),
      (
        "<..>",
        vec![(LessThan, "<"), (Dots, ".."), (GreaterThan, ">")],
      ),
      (
        "<1.1>",
        vec![
          (LessThan, "<"),
          (Digits, "1"),
          (Dots, "."),
          (Digits, "1"),
          (GreaterThan, ">"),
        ],
      ),
      (
        "<!--1x-->",
        vec![
          (LessThan, "<"),
          (Bang, "!"),
          (Dashes, "--"),
          (Digits, "1"),
          (Word, "x--"),
          (GreaterThan, ">"),
        ],
      ),
      (
        "<x>",
        vec![(LessThan, "<"), (Word, "x"), (GreaterThan, ">")],
      ),
      ("él", vec![(Word, "él")]),
      (
        "foo él",
        vec![(Word, "foo"), (Whitespace, " "), (Word, "él")],
      ),
      ("{}", vec![(OpenBrace, "{"), (CloseBrace, "}")]),
      (
        "{foo}",
        vec![(OpenBrace, "{"), (Word, "foo"), (CloseBrace, "}")],
      ),
      ("  ", vec![(Whitespace, "  ")]),
      (".", vec![(Dots, ".")]),
      ("..", vec![(Dots, "..")]),
      ("1", vec![(Digits, "1")]),
      ("12345", vec![(Digits, "12345")]),
      ("-", vec![(Dashes, "-")]),
      ("---", vec![(Dashes, "---")]),
      (
        "---- foo",
        vec![(Dashes, "----"), (Whitespace, " "), (Word, "foo")],
      ),
      ("-----", vec![(Dashes, "-----")]),
      ("--", vec![(DelimiterLine, "--")]),
      ("--\n", vec![(DelimiterLine, "--"), (Newline, "\n")]),
      ("****", vec![(DelimiterLine, "****")]),
      ("====", vec![(DelimiterLine, "====")]),
      ("____", vec![(DelimiterLine, "____")]),
      ("----", vec![(DelimiterLine, "----")]),
      ("....", vec![(DelimiterLine, "....")]),
      ("++++", vec![(DelimiterLine, "++++")]),
      (
        "****\nfoo",
        vec![(DelimiterLine, "****"), (Newline, "\n"), (Word, "foo")],
      ),
      (
        "foo****",
        vec![
          (Word, "foo"),
          (Star, "*"),
          (Star, "*"),
          (Star, "*"),
          (Star, "*"),
        ],
      ),
      ("foo@bar", vec![(Word, "foo@bar")]),
      (
        "foo@bar.com@",
        vec![(Word, "foo@bar"), (Dots, "."), (Word, "com@")],
      ),
      ("@foo@bar", vec![(Word, "@foo@bar")]),
      ("foo@", vec![(Word, "foo@")]),
      ("foo@.a", vec![(Word, "foo@"), (Dots, "."), (Word, "a")]),
      ("foo.bar", vec![(Word, "foo"), (Dots, "."), (Word, "bar")]),
      (
        "roflfootnote:",
        vec![(Word, "rofl"), (MacroName, "footnote:")],
      ),
      ("footnote:", vec![(MacroName, "footnote:")]),
      ("==", vec![(EqualSigns, "==")]),
      ("===", vec![(EqualSigns, "===")]),
      (
        "// foo",
        vec![(ForwardSlashes, "//"), (Whitespace, " "), (Word, "foo")],
      ),
      (
        "foo\n//-\nbar",
        vec![
          (Word, "foo"),
          (Newline, "\n"),
          (ForwardSlashes, "//"),
          (Dashes, "-"),
          (Newline, "\n"),
          (Word, "bar"),
        ],
      ),
      (
        "foo     ;", // whitespace is grouped
        vec![(Word, "foo"), (Whitespace, "     "), (SemiColon, ";")],
      ),
      (
        "foo=;",
        vec![(Word, "foo"), (EqualSigns, "="), (SemiColon, ";")],
      ),
      (
        "foo;=",
        vec![(Word, "foo"), (SemiColon, ";"), (EqualSigns, "=")],
      ),
      (
        "foo;image:&bar,lol^~_*.!`+[]#'\"\\%",
        vec![
          (Word, "foo"),
          (SemiColon, ";"),
          (MacroName, "image:"),
          (Ampersand, "&"),
          (Word, "bar"),
          (Comma, ","),
          (Word, "lol"),
          (Caret, "^"),
          (Tilde, "~"),
          (Underscore, "_"),
          (Star, "*"),
          (Dots, "."),
          (Bang, "!"),
          (Backtick, "`"),
          (Plus, "+"),
          (OpenBracket, "["),
          (CloseBracket, "]"),
          (Hash, "#"),
          (SingleQuote, "'"),
          (DoubleQuote, "\""),
          (Backslash, "\\"),
          (Percent, "%"),
        ],
      ),
      (
        "Foobar\n\n",
        vec![(Word, "Foobar"), (Newline, "\n"), (Newline, "\n")],
      ),
      (
        "== Title",
        vec![(EqualSigns, "=="), (Whitespace, " "), (Word, "Title")],
      ),
      (
        adoc! { "
          // ignored
          = Document Title
          Kismet R. Lee <kismet@asciidoctor.org>
          :description: The document's description.
          :sectanchors:
          :url-repo: https://my-git-repo.com

          The document body starts here.
        "},
        vec![
          (ForwardSlashes, "//"),
          (Whitespace, " "),
          (Word, "ignored"),
          (Newline, "\n"),
          (EqualSigns, "="),
          (Whitespace, " "),
          (Word, "Document"),
          (Whitespace, " "),
          (Word, "Title"),
          (Newline, "\n"),
          (Word, "Kismet"),
          (Whitespace, " "),
          (Word, "R"),
          (Dots, "."),
          (Whitespace, " "),
          (Word, "Lee"),
          (Whitespace, " "),
          (LessThan, "<"),
          (MaybeEmail, "kismet@asciidoctor.org"),
          (GreaterThan, ">"),
          (Newline, "\n"),
          (Colon, ":"),
          (Word, "description"),
          (Colon, ":"),
          (Whitespace, " "),
          (Word, "The"),
          (Whitespace, " "),
          (Word, "document"),
          (SingleQuote, "'"),
          (Word, "s"),
          (Whitespace, " "),
          (Word, "description"),
          (Dots, "."),
          (Newline, "\n"),
          (Colon, ":"),
          (Word, "sectanchors"),
          (Colon, ":"),
          (Newline, "\n"),
          (Colon, ":"),
          (Word, "url-repo"),
          (Colon, ":"),
          (Whitespace, " "),
          (MacroName, "https:"),
          (ForwardSlashes, "//"),
          (Word, "my-git-repo"),
          (Dots, "."),
          (Word, "com"),
          (Newline, "\n"),
          (Newline, "\n"),
          (Word, "The"),
          (Whitespace, " "),
          (Word, "document"),
          (Whitespace, " "),
          (Word, "body"),
          (Whitespace, " "),
          (Word, "starts"),
          (Whitespace, " "),
          (Word, "here"),
          (Dots, "."),
          (Newline, "\n"),
        ],
      ),
    ];
    assert_token_cases!(cases);
  }

  #[test]
  fn test_directives() {
    assert_token_cases!([
      (
        "include::foo",
        vec![(Directive, "include::"), (Word, "foo")],
      ),
      (
        // not valid, but should lex as Directive
        // parser will reject it as a match
        "include::not-include []",
        vec![
          (Directive, "include::"),
          (Word, "not-include"),
          (Whitespace, " "),
          (OpenBracket, "["),
          (CloseBracket, "]")
        ],
      )
    ]);

    refute_produces_token!(
      Directive,
      [
        "foo include::foo",        // not at start of line
        "include:: foo",           // space after ::
        "include:: not-include[]", // space after ::
        "include:: []",            // space after ::
        "include::[]",             // empty target
      ]
    );
  }

  #[test]
  fn test_term_delimiters() {
    let foo = (Word, "foo");
    let space = (Whitespace, " ");
    let cases = vec![
      ("foo:: foo", vec![foo, (TermDelimiter, "::"), space, foo]),
      ("foo::", vec![foo, (TermDelimiter, "::")]),
      ("foo;;", vec![foo, (TermDelimiter, ";;")]),
      ("foo:::", vec![foo, (TermDelimiter, ":::")]),
      ("foo::::", vec![foo, (TermDelimiter, "::::")]),
      // doesn't trip up on macros
      (
        "image:: foo",
        vec![(Word, "image"), (TermDelimiter, "::"), space, foo],
      ),
      (
        "xfootnote:: foo",
        vec![(Word, "xfootnote"), (TermDelimiter, "::"), space, foo],
      ),
      (
        "kbd::: foo",
        vec![(Word, "kbd"), (TermDelimiter, ":::"), space, foo],
      ),
      (
        "footnote:::: foo",
        vec![(Word, "footnote"), (TermDelimiter, "::::"), space, foo],
      ),
    ];
    assert_token_cases!(cases);

    refute_produces_token!(
      TermDelimiter,
      ["foo::foo", "foo;;;", "foo:::::", "foo:::::foo", ":: foo"]
    );
  }

  #[test]
  fn test_tokens_full() {
    let input = "&^foobar[//";
    let bump = &Bump::new();
    let mut lexer = RootLexer::from_str(bump, input);
    assert_eq!(
      lexer.next_token(),
      Token {
        kind: TokenKind::Ampersand,
        loc: SourceLocation::new(0, 1),
        lexeme: BumpString::from_str_in("&", bump),
      }
    );
    assert_eq!(
      lexer.next_token(),
      Token {
        kind: TokenKind::Caret,
        loc: SourceLocation::new(1, 2),
        lexeme: BumpString::from_str_in("^", bump),
      }
    );
    assert_eq!(
      lexer.next_token(),
      Token {
        kind: TokenKind::Word,
        loc: SourceLocation::new(2, 8),
        lexeme: BumpString::from_str_in("foobar", bump),
      }
    );
    assert_eq!(
      lexer.next_token(),
      Token {
        kind: TokenKind::OpenBracket,
        loc: SourceLocation::new(8, 9),
        lexeme: BumpString::from_str_in("[", bump),
      }
    );
    assert_eq!(
      lexer.next_token(),
      Token {
        kind: TokenKind::ForwardSlashes,
        loc: SourceLocation::new(9, 11),
        lexeme: BumpString::from_str_in("//", bump),
      }
    );
    assert_eq!(
      lexer.next_token(),
      Token {
        kind: TokenKind::Eof,
        loc: SourceLocation::new(12, 12),
        lexeme: BumpString::from_str_in("", bump),
      }
    );
  }

  #[test]
  fn test_line_of() {
    let bump = &Bump::new();
    let input = "foo\nbar\n\nbaz\n";
    let lexer = RootLexer::from_str(bump, input);
    assert_eq!(lexer.line_of(1), "foo");
    assert_eq!(lexer.line_of(2), "foo");
    assert_eq!(lexer.line_of(3), "foo"); // newline

    assert_eq!(lexer.line_of(4), "bar");
    assert_eq!(lexer.line_of(7), "bar");
    assert_eq!(lexer.line_of(8), ""); // empty line
    assert_eq!(lexer.line_of(9), "baz");
  }

  #[test]
  fn test_line_num() {
    let input = "= :
foo

;
";
    let bump = &Bump::new();
    let mut lexer = RootLexer::from_str(bump, input);

    assert_next_token_line(&mut lexer, 1, EqualSigns);
    assert_next_token_line(&mut lexer, 1, Whitespace);
    assert_next_token_line(&mut lexer, 1, Colon);
    assert_next_token_line(&mut lexer, 1, Newline);
    assert_next_token_line(&mut lexer, 2, Word);
    assert_next_token_line(&mut lexer, 2, Newline);
    assert_next_token_line(&mut lexer, 3, Newline);
  }

  fn assert_next_token_line(lexer: &mut RootLexer, line: u32, expected_kind: TokenKind) {
    let token = lexer.next_token();
    assert_eq!(token.kind, expected_kind);
    assert_eq!(lexer.line_number(token.loc.start), line);
  }

  #[test]
  fn test_consume_empty_lines() {
    let bump = &Bump::new();
    let input = "\n\n\n\n\n";
    let mut lexer = RootLexer::from_str(bump, input);
    lexer.consume_empty_lines();
    assert!(lexer.is_eof());
  }
}
