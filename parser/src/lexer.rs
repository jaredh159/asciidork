use std::str::Chars;

use bumpalo::collections::Vec as BumpVec;
use bumpalo::Bump;

use crate::ast::SourceLocation;
use crate::line::Line;
use crate::token::{Token, TokenKind, TokenKind::*};

#[derive(Debug)]
pub struct Lexer<'src> {
  src: &'src str,
  chars: Chars<'src>,
  peek: Option<char>,
}

impl<'src> Lexer<'src> {
  pub fn new(src: &'src str) -> Lexer<'src> {
    let mut lexer = Lexer { src, chars: src.chars(), peek: None };
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

  pub fn next_token(&mut self) -> Token<'src> {
    match self.advance() {
      Some('=') => self.repeating('=', EqualSigns),
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
      Some('.') => self.single(Dot),
      Some('!') => self.single(Bang),
      Some('`') => self.single(Backtick),
      Some('+') => self.single(Plus),
      Some('[') => self.single(OpenBracket),
      Some(']') => self.single(CloseBracket),
      Some('#') => self.single(Hash),
      Some('%') => self.single(Percent),
      Some('"') => self.single(DoubleQuote),
      Some('/') if self.starts_comment() => self.comment(),
      Some('\'') => self.single(SingleQuote),
      Some('\\') => self.single(Backslash),
      Some(_) => self.word(),
      None => Token {
        kind: Eof,
        loc: SourceLocation::new(self.offset(), self.offset()),
        lexeme: "",
      },
    }
  }

  fn offset(&self) -> usize {
    self.src.len() - self.chars.as_str().len() - self.peek.is_some() as usize // O(1) √
  }

  fn advance(&mut self) -> Option<char> {
    std::mem::replace(&mut self.peek, self.chars.next())
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
    Token {
      kind,
      loc: SourceLocation::new(start, end),
      lexeme: &self.src[start..end],
    }
  }

  fn repeating(&mut self, c: char, kind: TokenKind) -> Token<'src> {
    let start = self.offset() - 1;
    let end = self.advance_while(c);
    Token {
      kind,
      loc: SourceLocation::new(start, end),
      lexeme: &self.src[start..end],
    }
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

    // 👍 monday jared... not 100% sure, but, i think i want to
    // encode a new token type of Email, by checking for the @
    // and peeking ahead
    // also, i should try to handle the `\` to opt out of macro and autolinking
    // per https://docs.asciidoctor.org/asciidoc/latest/macros/autolinks/#escaping-urls-and-email-addresses

    // special cases
    match self.peek {
      // macros
      Some(':') => {
        if self.is_macro_name(lexeme) {
          self.advance();
          return Token {
            kind: MacroName,
            loc: SourceLocation::new(start, end + 1),
            lexeme: &self.src[start..end + 1],
          };
          // ...checking for contiguous footnote `somethingfootnote:[]`
        } else if lexeme.ends_with('e') && lexeme.ends_with("footnote") {
          self.reverse_by(8);
          return Token {
            kind: Word,
            loc: SourceLocation::new(start, end - 8),
            lexeme: &self.src[start..end - 8],
          };
        }
      }
      // maybe email
      Some('@') => {
        self.advance();
        let domain_end = self
          .advance_while_with(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_');
        let domain = &self.src[end + 1..domain_end];
        if domain.len() > 3 && domain.contains('.') && !self.peek_is('@') {
          return Token {
            kind: MaybeEmail,
            loc: SourceLocation::new(start, domain_end),
            lexeme: &self.src[start..domain_end],
          };
        }
        self.reverse_by(domain.len());
        let end = self.advance_to_word_boundary(false);
        return Token {
          kind: Word,
          loc: SourceLocation::new(start, end),
          lexeme: &self.src[start..end],
        };
      }
      _ => {}
    }
    Token {
      kind: Word,
      loc: SourceLocation::new(start, end),
      lexeme: &self.src[start..end],
    }
  }
  fn reverse_by(&mut self, n: usize) {
    self.chars = self.src[self.offset() - n..].chars();
    self.peek = self.chars.next();
  }

  fn is_macro_name(&self, lexeme: &str) -> bool {
    matches!(
      lexeme,
      "footnote" | "image" | "irc" | "icon" | "kbd" | "link" | "http" | "https" | "ftp" | "mailto"
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
    Token {
      kind: CommentLine,
      loc: SourceLocation::new(start, end),
      lexeme: &self.src[start..end],
    }
  }

  fn whitespace(&mut self) -> Token<'src> {
    let start = self.offset() - 1;
    self.advance_while_one_of(&[' ', '\t']);
    let end = self.offset();
    Token {
      kind: Whitespace,
      loc: SourceLocation::new(start, end),
      lexeme: &self.src[start..end],
    }
  }

  fn starts_comment(&self) -> bool {
    if self.peek != Some('/') {
      return false;
    }
    let offset = self.offset();
    if offset == 1 {
      return true;
    }
    let prev_offset = self.offset().saturating_sub(1);
    // must be at the beginning of a line, so `https://foobar` not match
    matches!(self.src.chars().nth(prev_offset), Some('\n') | None)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::ast::SourceLocation;
  use crate::token::TokenKind;

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
      ("foo@bar", vec![(Word, "foo@bar")]),
      (
        "foo@bar.com@",
        vec![(Word, "foo@bar"), (Dot, "."), (Word, "com@")],
      ),
      ("@foo@bar", vec![(Word, "@foo@bar")]),
      ("foo@", vec![(Word, "foo@")]),
      ("foo@.a", vec![(Word, "foo@"), (Dot, "."), (Word, "a")]),
      ("foo.bar", vec![(Word, "foo"), (Dot, "."), (Word, "bar")]),
      (
        "roflfootnote:",
        vec![(Word, "rofl"), (MacroName, "footnote:")],
      ),
      ("footnote:", vec![(MacroName, "footnote:")]),
      ("==", vec![(EqualSigns, "==")]),
      ("===", vec![(EqualSigns, "===")]),
      ("// foo", vec![(CommentLine, "// foo")]),
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
          (Dot, "."),
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
        "// this comment line is ignored
= Document Title
Kismet R. Lee <kismet@asciidoctor.org>
:description: The document's description.
:sectanchors:
:url-repo: https://my-git-repo.com

The document body starts here.
",
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
          (Dot, "."),
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
          (Dot, "."),
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
          (Dot, "."),
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
          (Dot, "."),
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
        assert_eq!(lexer.next_token(), expected_token);
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
