use super::source_lexer::SourceLexer;
use crate::internal::*;

#[derive(Debug)]
pub struct RootLexer<'arena> {
  bump: &'arena Bump,
  idx: usize, // u16?
  action: Action,
  source_stack: SmallVec<[u16; 4]>,
  sources: BumpVec<'arena, SourceLexer<'arena>>,
}

#[derive(Debug)]
enum Action {
  Enter(u16),
  Exit(u16),
  Consume,
}

impl<'arena> RootLexer<'arena> {
  pub fn new(src: BumpVec<'arena, u8>, bump: &'arena Bump) -> Self {
    Self {
      bump,
      idx: 0,
      action: Action::Consume,
      source_stack: SmallVec::new(),
      sources: bvec![in bump; SourceLexer::new(src, bump)],
    }
  }

  pub fn from_str(bump: &'arena Bump, src: &str) -> Self {
    Self {
      bump,
      idx: 0,
      action: Action::Consume,
      source_stack: SmallVec::new(),
      sources: bvec![in bump; SourceLexer::from_str(src, bump)],
    }
  }

  pub fn from_byte_slice(bytes: &[u8], bump: &'arena Bump) -> Self {
    Self {
      bump,
      idx: 0,
      action: Action::Consume,
      source_stack: SmallVec::new(),
      sources: bvec![in bump; SourceLexer::from_byte_slice(bytes, bump)],
    }
  }

  pub fn push_source(&mut self, _filename: &str, src: BumpVec<'arena, u8>) {
    self.sources.push(SourceLexer::new(src, self.bump));
    let next_idx = self.sources.len() as u16 - 1;
    self.action = Action::Enter(next_idx);
  }

  pub fn adjust_offset(&mut self, offset_adjustment: u32) {
    self.sources[self.idx].offset = offset_adjustment;
  }

  pub fn peek(&self) -> Option<u8> {
    self.sources[self.idx].peek()
  }

  pub fn consume_empty_lines(&mut self) {
    self.sources[self.idx].consume_empty_lines();
  }

  pub fn raw_lines(&'arena self) -> impl Iterator<Item = &'arena str> {
    self.sources[self.idx].raw_lines()
  }

  pub fn loc(&self) -> SourceLocation {
    SourceLocation::from(self.sources[self.idx].pos)
  }

  pub fn at_delimiter_line(&self) -> Option<(u32, u8)> {
    self.sources[self.idx].at_delimiter_line()
  }

  pub fn is_eof(&self) -> bool {
    self.peek().is_none()
  }

  pub fn peek_is(&self, c: u8) -> bool {
    self.peek() == Some(c)
  }

  pub fn line_of(&self, location: u32) -> BumpString<'arena> {
    self.sources[self.idx].line_of(location)
  }

  pub fn line_number(&self, location: u32) -> u32 {
    let (line_number, _) = self.line_number_with_offset(location);
    line_number
  }

  pub fn line_number_with_offset(&self, location: u32) -> (u32, u32) {
    self.sources[self.idx].line_number_with_offset(location)
  }

  pub fn consume_line(&mut self) -> Option<Line<'arena>> {
    self.sources[self.idx].consume_line()
  }

  pub fn next_token(&mut self) -> Token<'arena> {
    match self.action {
      Action::Enter(next_idx) => {
        let mut include_loc = self.loc().decr();
        let mut line = self.line_of(include_loc.start);
        line.replace_range(0..9, &format!("{{->{:05}}}", next_idx));
        include_loc.start -= line.len() as u32;
        self.source_stack.push(self.idx as u16);
        self.idx = next_idx as usize;
        self.action = Action::Consume;
        Token::new(TokenKind::BeginInclude, include_loc, line)
      }
      Action::Exit(_idx) => {
        todo!("exit cursor")
      }
      Action::Consume => match self.sources[self.idx].next_token() {
        Some(token) => token,
        None => {
          dbg!(&self);
          let Some(prev_idx) = self.source_stack.pop() else {
            return Token::new(
              TokenKind::Eof,
              self.loc(),
              BumpString::from_str_in("", self.bump),
            );
          };

          // wed jared 👍, maybe should hold a vec of SourceLocation, which
          // would give me both the depth, and the full loc of the line

          todo!("thingy")
        }
      },
    }
  }

  pub fn truncate(&mut self) {
    self.sources[self.idx].truncate();
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::token::TokenKind;
  use crate::variants::token::*;
  use ast::SourceLocation;
  use test_utils::{assert_eq, *};

  // #[test]
  fn test_include_boundaries() {
    let input = adoc! {"
      foo
      include::bar.adoc[]
      baz
    "};
    let bump = &Bump::new();
    let mut lexer = RootLexer::from_str(bump, input);
    // assert_eq!(lexer.next_token(), Token::new(Word, 0..3, bstr!("foo")));
    // assert_eq!(lexer.next_token(), Token::new(Newline, 3..4, bstr!("\n")));
    // assert_eq!(
    //   lexer.next_token(),
    //   Token::new(Directive, 4..13, bstr!("include::"))
    // );
    // assert_eq!(lexer.next_token(), Token::new(Word, 13..16, bstr!("foo")));
    // assert_eq!(lexer.next_token(), Token::new(Dots, 16..17, bstr!(".")));
    // assert_eq!(lexer.next_token(), Token::new(Word, 17..21, bstr!("adoc")));
    // assert_eq!(
    //   lexer.next_token(),
    //   Token::new(OpenBracket, 21..22, bstr!("["))
    // );

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
    // tues jared, get this passing, express depth in source location
    // consider dropping location from `Block`, deriving from contents
    // remember maybe u16 the idxs in lexers
    assert_eq!(
      lexer.next_token(),
      Token::new(Word, 0..3 /* /1 */, bstr!("bar"))
    );
    assert_eq!(
      lexer.next_token(),
      Token::new(Newline, 3..4 /* /1 */, bstr!("\n"))
    );
    assert_eq!(
      lexer.next_token(),
      Token::new(EndInclude, 4..24, bstr!("{<-00001}bar.adoc[]\n"))
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
      (
        "include::foo",
        vec![(Directive, "include::"), (Word, "foo")],
      ),
      (
        "foo include::foo",
        vec![
          (Word, "foo"),
          (Whitespace, " "),
          (Word, "include"),
          (Colon, ":"),
          (Colon, ":"),
          (Word, "foo"),
        ],
      ),
      (
        "include:: foo",
        vec![
          (Word, "include"),
          (TermDelimiter, "::"),
          (Whitespace, " "),
          (Word, "foo"),
        ],
      ),
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
    let bump = &Bump::new();
    for (input, expected) in cases {
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
        assert_eq!(lexer.next_token(), expected_token, from: input);
        index = end;
      }
      assert_eq!(lexer.next_token().kind, Eof);
    }
  }

  #[test]
  fn test_term_delimiters() {
    let col = (Colon, ":");
    let semi = (SemiColon, ";");
    let foo = (Word, "foo");
    let space = (Whitespace, " ");
    let cases = vec![
      ("foo:: foo", vec![foo, (TermDelimiter, "::"), space, foo]),
      ("foo::foo", vec![foo, col, col, foo]),
      ("foo::", vec![foo, (TermDelimiter, "::")]),
      ("foo;;", vec![foo, (TermDelimiter, ";;")]),
      ("foo;;;", vec![foo, semi, semi, semi]),
      ("foo:::", vec![foo, (TermDelimiter, ":::")]),
      ("foo::::", vec![foo, (TermDelimiter, "::::")]),
      ("foo:::::", vec![foo, col, col, col, col, col]),
      ("foo:::::foo", vec![foo, col, col, col, col, col, foo]),
      (":: foo", vec![col, col, space, foo]),
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
    let bump = &Bump::new();
    for (input, expected) in cases {
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
        assert_eq!(lexer.next_token(), expected_token, from: input);
        index = end;
      }
      assert_eq!(lexer.next_token().kind, Eof);
    }
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
