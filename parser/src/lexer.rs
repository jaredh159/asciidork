use std::str::Chars;

use crate::internal::*;
use crate::variants::token::*;

#[derive(Debug)]
pub struct Lexer<'src> {
  src: &'src str,
  chars: Chars<'src>,
  peek: Option<char>,
  at_line_start: bool,
}

impl<'src> Lexer<'src> {
  pub fn new(src: &'src str) -> Lexer<'src> {
    let mut lexer = Lexer {
      src,
      chars: src.chars(),
      peek: None,
      at_line_start: true,
    };
    lexer.peek = lexer.chars.next();
    lexer
  }

  pub fn consume_empty_lines(&mut self) {
    while self.peek == Some('\n') {
      self.advance();
    }
  }

  pub fn loc(&self) -> SourceLocation {
    SourceLocation::from(self.offset())
  }

  pub fn is_eof(&self) -> bool {
    self.peek.is_none()
  }

  pub fn peek_is(&self, c: char) -> bool {
    self.peek == Some(c)
  }

  pub fn loc_src(&self, loc: SourceLocation) -> &'src str {
    &self.src[loc.start..loc.end]
  }

  pub fn print_current_line(&self) {
    let (line_num, _) = self.line_number_with_offset(self.offset());
    let line = self.line_of(self.offset());
    println!("{}: {}", line_num, line);
  }

  pub fn line_of(&self, location: usize) -> &'src str {
    let mut start = location;
    let mut end = location;

    for c in self.src.chars().rev().skip(self.src.len() - location) {
      if start == 0 || c == '\n' {
        break;
      } else {
        start -= 1;
      }
    }

    for c in self.src.chars().skip(location) {
      if c == '\n' {
        break;
      } else {
        end += 1;
      }
    }

    &self.src[start..end]
  }

  pub fn line_number(&self, location: usize) -> usize {
    let (line_number, _) = self.line_number_with_offset(location);
    line_number
  }

  pub fn line_number_with_offset(&self, location: usize) -> (usize, usize) {
    let mut line_number = 1;
    let mut offset: usize = 0;
    for c in self.src.chars().take(location) {
      if c == '\n' {
        offset = 0;
        line_number += 1;
      } else {
        offset += 1;
      }
    }
    (line_number, offset.saturating_sub(1))
  }

  pub fn consume_line<'bmp>(&mut self, bump: &'bmp Bump) -> Option<Line<'bmp, 'src>> {
    if self.is_eof() {
      return None;
    }
    let start = self.offset();
    let mut end = start;
    let mut tokens = BumpVec::new_in(bump);
    while !self.peek_is('\n') && !self.is_eof() {
      let token = self.next_token();
      end = token.loc.end;
      tokens.push(token);
    }
    if self.peek_is('\n') {
      self.advance();
    }
    Some(Line::new(tokens, &self.src[start..end]))
  }

  pub fn at_empty_line(&self) -> bool {
    self.at_line_start && self.peek_is('\n')
  }

  pub fn at_delimiter_line(&self) -> Option<(usize, char)> {
    if !self.at_line_start
      || self.is_eof()
      || !matches!(self.peek, Some('_') | Some('-') | Some('*') | Some('='))
    {
      return None;
    }
    let mut c = self.chars.clone();
    let sequence = [self.peek, c.next(), c.next(), c.next(), c.next()];
    match sequence {
      [Some('-'), Some('-'), Some('\n') | None, _, _] => Some((2, '-')),
      [Some('*'), Some('*'), Some('*'), Some('*'), Some('\n') | None]
      | [Some('_'), Some('_'), Some('_'), Some('_'), Some('\n') | None]
      | [Some('-'), Some('-'), Some('-'), Some('-'), Some('\n') | None]
      | [Some('='), Some('='), Some('='), Some('='), Some('\n') | None] => {
        Some((4, sequence[0].unwrap()))
      }
      _ => None,
    }
  }

  fn delimiter_line(&mut self) -> Option<Token<'src>> {
    let (len, _) = self.at_delimiter_line()?;
    let start = self.offset();
    self.skip(len);
    Some(self.token(DelimiterLine, start, start + len))
  }

  pub fn next_token(&mut self) -> Token<'src> {
    if let Some(token) = self.delimiter_line() {
      return token;
    }
    let at_line_start = self.at_line_start;
    match self.advance() {
      Some('=') => self.repeating('=', EqualSigns),
      Some('-') => self.repeating('-', Dashes),
      Some(' ') | Some('\t') => self.whitespace(),
      Some('&') => self.single(Ampersand),
      Some('\n') => self.single(Newline),
      Some(':') => self.single(Colon),
      Some(';') => self.single(SemiColon),
      Some('<') => self.single(LessThan),
      Some('>') => self.single(GreaterThan),
      Some(',') => self.single(Comma),
      Some('^') => self.single(Caret),
      Some('~') => self.single(Tilde),
      Some('_') => self.single(Underscore),
      Some('*') => self.single(Star),
      Some('.') => self.repeating('.', Dots),
      Some('!') => self.single(Bang),
      Some('`') => self.single(Backtick),
      Some('+') => self.single(Plus),
      Some('[') => self.single(OpenBracket),
      Some(']') => self.single(CloseBracket),
      Some('{') => self.single(OpenBrace),
      Some('}') => self.single(CloseBrace),
      Some('#') => self.single(Hash),
      Some('%') => self.single(Percent),
      Some('"') => self.single(DoubleQuote),
      Some('/') if at_line_start && self.peek == Some('/') => self.comment(),
      Some('\'') => self.single(SingleQuote),
      Some('\\') => self.single(Backslash),
      Some(ch) if ch.is_ascii_digit() => self.digits(),
      Some(_) => self.word(),
      None => self.token(Eof, self.offset(), self.offset()),
    }
  }

  fn offset(&self) -> usize {
    self.src.len() - self.chars.as_str().len() - self.peek.is_some() as usize // O(1) √
  }

  fn advance(&mut self) -> Option<char> {
    let next = std::mem::replace(&mut self.peek, self.chars.next());
    self.at_line_start = matches!(next, Some('\n'));
    next
  }

  pub fn skip(&mut self, n: usize) -> Option<char> {
    debug_assert!(n > 1);
    let next = std::mem::replace(&mut self.peek, self.chars.nth(n - 1));
    self.at_line_start = matches!(next, Some('\n'));
    next
  }

  fn advance_if(&mut self, c: char) -> bool {
    if self.peek == Some(c) {
      self.advance();
      true
    } else {
      false
    }
  }

  fn advance_while(&mut self, c: char) -> usize {
    while self.advance_if(c) {}
    self.offset()
  }

  fn single(&self, kind: TokenKind) -> Token<'src> {
    let end = self.offset();
    let start = end - 1;
    self.token(kind, start, end)
  }

  fn repeating(&mut self, c: char, kind: TokenKind) -> Token<'src> {
    let start = self.offset() - 1;
    let end = self.advance_while(c);
    self.token(kind, start, end)
  }

  fn digits(&mut self) -> Token<'src> {
    let start = self.offset() - 1;
    let end = self.advance_while_with(|c| c.is_ascii_digit());
    self.token(Digits, start, end)
  }

  fn advance_while_with(&mut self, f: impl Fn(char) -> bool) -> usize {
    while self.peek.map_or(false, &f) {
      self.advance();
    }
    self.offset()
  }

  fn advance_to_word_boundary(&mut self, with_at: bool) -> usize {
    self.advance_until_one_of(&[
      ' ',
      '\t',
      '\n',
      ':',
      ';',
      '<',
      '>',
      ',',
      '^',
      '_',
      '~',
      '*',
      '!',
      '`',
      '+',
      '.',
      '[',
      ']',
      '{',
      '}',
      '=',
      '"',
      '\'',
      '\\',
      '%',
      '#',
      '&',
      if with_at { '@' } else { '&' },
    ])
  }

  fn word(&mut self) -> Token<'src> {
    let start = self.offset() - 1;
    let end = self.advance_to_word_boundary(true);
    let lexeme = &self.src[start..end];

    // special cases
    match self.peek {
      // macros
      Some(':') => {
        if self.is_macro_name(lexeme) {
          self.advance();
          return self.token(MacroName, start, end + 1);
          // ...checking for contiguous footnote `somethingfootnote:[]`
        } else if lexeme.ends_with('e') && lexeme.ends_with("footnote") {
          self.reverse_by(8);
          return self.token(Word, start, end - 8);
        }
      }
      // maybe email
      Some('@') => {
        self.advance();
        let domain_end = self
          .advance_while_with(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_');
        let domain = &self.src[end + 1..domain_end];
        if domain.len() > 3 && domain.contains('.') && !self.peek_is('@') {
          return self.token(MaybeEmail, start, domain_end);
        }
        self.reverse_by(domain.len());
        let end = self.advance_to_word_boundary(false);
        return self.token(Word, start, end);
      }
      _ => {}
    }
    self.token(Word, start, end)
  }
  fn reverse_by(&mut self, n: usize) {
    self.chars = self.src[self.offset() - n..].chars();
    self.peek = self.chars.next();
  }

  fn is_macro_name(&self, lexeme: &str) -> bool {
    matches!(
      lexeme,
      "footnote"
        | "image"
        | "irc"
        | "icon"
        | "kbd"
        | "link"
        | "http"
        | "https"
        | "ftp"
        | "mailto"
        | "pass"
        | "btn"
        | "menu"
    )
  }

  fn advance_until(&mut self, stop: char) {
    loop {
      match self.peek {
        None => break,
        Some(c) if c == stop => break,
        _ => {
          self.advance();
        }
      }
    }
  }

  fn advance_until_one_of(&mut self, chars: &[char]) -> usize {
    loop {
      match self.peek {
        Some(c) if chars.contains(&c) => break,
        None => break,
        _ => {
          self.advance();
        }
      }
    }
    self.offset()
  }

  fn advance_while_one_of(&mut self, chars: &[char]) {
    loop {
      match self.peek {
        Some(c) if chars.contains(&c) => {}
        _ => break,
      }
      self.advance();
    }
  }

  fn comment(&mut self) -> Token<'src> {
    let start = self.offset() - 1;
    // TODO: block comments, testing if we have 2 more slashes
    self.advance_until('\n');
    let end = self.offset();
    self.token(CommentLine, start, end)
  }

  fn whitespace(&mut self) -> Token<'src> {
    let start = self.offset() - 1;
    self.advance_while_one_of(&[' ', '\t']);
    let end = self.offset();
    self.token(Whitespace, start, end)
  }

  fn token(&self, kind: TokenKind, start: usize, end: usize) -> Token<'src> {
    Token {
      kind,
      loc: SourceLocation::new(start, end),
      lexeme: &self.src[start..end],
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::token::TokenKind;
  use ast::SourceLocation;
  use indoc::indoc;

  #[test]
  fn test_consume_line() {
    let bump = &Bump::new();
    let mut lexer = Lexer::new("foo bar\nso baz\n");
    assert_eq!(lexer.consume_line(bump).unwrap().src, "foo bar");
    assert_eq!(lexer.consume_line(bump).unwrap().src, "so baz");
    assert!(lexer.consume_line(bump).is_none());
  }

  #[test]
  fn test_tokens() {
    let cases = vec![
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
      ("// foo", vec![(CommentLine, "// foo")]),
      (
        "foo\n//-\nbar",
        vec![
          (Word, "foo"),
          (Newline, "\n"),
          (CommentLine, "//-"),
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
        indoc! { "
          // this comment line is ignored
          = Document Title
          Kismet R. Lee <kismet@asciidoctor.org>
          :description: The document's description.
          :sectanchors:
          :url-repo: https://my-git-repo.com

          The document body starts here.
        "},
        vec![
          (CommentLine, "// this comment line is ignored"),
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
          (Word, "//my-git-repo"),
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
    for (input, expected) in cases {
      let mut lexer = Lexer::new(input);
      let mut index = 0;
      for (token_type, lexeme) in expected {
        let start = index;
        let end = start + lexeme.len();
        let expected_token = Token {
          kind: token_type,
          loc: SourceLocation::new(start, end),
          lexeme,
        };
        assert_eq!(
          lexer.next_token(),
          expected_token,
          "input was:\n\n```\n{}\n```\n",
          input
        );
        index = end;
      }
      assert_eq!(lexer.next_token().kind, Eof);
    }
  }

  #[test]
  fn test_tokens_full() {
    let input = "&^foobar[";
    let mut lexer = Lexer::new(input);
    assert_eq!(lexer.src, input);
    assert_eq!(
      lexer.next_token(),
      Token {
        kind: TokenKind::Ampersand,
        loc: SourceLocation::new(0, 1),
        lexeme: "&",
      }
    );
    assert_eq!(
      lexer.next_token(),
      Token {
        kind: TokenKind::Caret,
        loc: SourceLocation::new(1, 2),
        lexeme: "^",
      }
    );
    assert_eq!(
      lexer.next_token(),
      Token {
        kind: TokenKind::Word,
        loc: SourceLocation::new(2, 8),
        lexeme: "foobar",
      }
    );
    assert_eq!(
      lexer.next_token(),
      Token {
        kind: TokenKind::OpenBracket,
        loc: SourceLocation::new(8, 9),
        lexeme: "[",
      }
    );
    assert_eq!(
      lexer.next_token(),
      Token {
        kind: TokenKind::Eof,
        loc: SourceLocation::new(9, 9),
        lexeme: "",
      }
    );
  }

  #[test]
  fn test_line_of() {
    let input = "foo\nbar\n\nbaz\n";
    let lexer = Lexer::new(input);
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
    let mut lexer = Lexer::new(input);

    assert_next_token_line(&mut lexer, 1, EqualSigns);
    assert_next_token_line(&mut lexer, 1, Whitespace);
    assert_next_token_line(&mut lexer, 1, Colon);
    assert_next_token_line(&mut lexer, 1, Newline);
    assert_next_token_line(&mut lexer, 2, Word);
    assert_next_token_line(&mut lexer, 2, Newline);
    assert_next_token_line(&mut lexer, 3, Newline);
  }

  fn assert_next_token_line(lexer: &mut Lexer, line: usize, expected_kind: TokenKind) {
    let token = lexer.next_token();
    assert_eq!(token.kind, expected_kind);
    assert_eq!(lexer.line_number(token.loc.start), line);
  }

  #[test]
  fn test_consume_empty_lines() {
    let input = "\n\n\n\n\n";
    let mut lexer = Lexer::new(input);
    lexer.consume_empty_lines();
    assert!(lexer.is_eof());
  }
}
