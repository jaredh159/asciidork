use std::str::{Bytes, Lines};

use crate::internal::*;
use crate::variants::token::*;

#[derive(Debug)]
pub struct Lexer<'src> {
  src: &'src str,
  bytes: Bytes<'src>,
  peek: Option<u8>,
  at_line_start: bool,
  pattern_breaker: Option<TokenKind>,
}

impl<'src> Lexer<'src> {
  pub fn new(src: &'src str) -> Lexer<'src> {
    let mut lexer = Lexer {
      src,
      bytes: src.bytes(),
      peek: None,
      at_line_start: true,
      pattern_breaker: None,
    };
    lexer.peek = lexer.bytes.next();
    lexer
  }

  pub fn consume_empty_lines(&mut self) {
    while self.peek == Some(b'\n') {
      self.advance();
    }
  }

  pub fn raw_lines(&self) -> Lines<'src> {
    self.src.lines()
  }

  pub fn loc(&self) -> SourceLocation {
    SourceLocation::from(self.offset())
  }

  pub const fn is_eof(&self) -> bool {
    self.peek.is_none()
  }

  pub fn peek_is(&self, c: u8) -> bool {
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

    for c in self.src.bytes().rev().skip(self.src.len() - location) {
      if start == 0 || c == b'\n' {
        break;
      } else {
        start -= 1;
      }
    }

    for c in self.src.bytes().skip(location) {
      if c == b'\n' {
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
    for c in self.src.bytes().take(location) {
      if c == b'\n' {
        offset = 0;
        line_number += 1;
      } else {
        offset += 1;
      }
    }
    (line_number, offset)
  }

  pub fn consume_line<'bmp>(&mut self, bump: &'bmp Bump) -> Option<Line<'bmp, 'src>> {
    if self.is_eof() {
      return None;
    }
    let start = self.offset();
    let mut end = start;
    let mut tokens = BumpVec::new_in(bump);
    while !self.peek_is(b'\n') && !self.is_eof() {
      let token = self.next_token();
      end = token.loc.end;
      tokens.push(token);
    }
    if self.peek_is(b'\n') {
      self.advance();
    }
    Some(Line::new(tokens, &self.src[start..end]))
  }

  pub fn at_empty_line(&self) -> bool {
    self.at_line_start && self.peek_is(b'\n')
  }

  pub fn at_delimiter_line(&self) -> Option<(usize, u8)> {
    if !self.at_line_start
      || self.is_eof()
      || !matches!(
        self.peek,
        Some(b'_') | Some(b'-') | Some(b'*') | Some(b'=') | Some(b'.') | Some(b'+') | Some(b'/')
      )
    {
      return None;
    }
    // | , !
    let mut c = self.bytes.clone();
    let sequence = [self.peek, c.next(), c.next(), c.next(), c.next()];
    match sequence {
      [Some(b'-'), Some(b'-'), Some(b'\n') | None, _, _] => Some((2, b'-')),
      [Some(b'*'), Some(b'*'), Some(b'*'), Some(b'*'), Some(b'\n') | None]
      | [Some(b'_'), Some(b'_'), Some(b'_'), Some(b'_'), Some(b'\n') | None]
      | [Some(b'-'), Some(b'-'), Some(b'-'), Some(b'-'), Some(b'\n') | None]
      | [Some(b'+'), Some(b'+'), Some(b'+'), Some(b'+'), Some(b'\n') | None]
      | [Some(b'.'), Some(b'.'), Some(b'.'), Some(b'.'), Some(b'\n') | None]
      | [Some(b'/'), Some(b'/'), Some(b'/'), Some(b'/'), Some(b'\n') | None]
      | [Some(b'='), Some(b'='), Some(b'='), Some(b'='), Some(b'\n') | None] => {
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
    let breaker = self.pattern_breaker.take();
    if let Some(token) = self.delimiter_line() {
      return token;
    }
    let at_line_start = self.at_line_start;
    match self.advance() {
      Some(b'=') => self.repeating(b'=', EqualSigns),
      Some(b'-') => self.repeating(b'-', Dashes),
      Some(b' ' | b'\t') => self.whitespace(),
      Some(b'&') => self.single(Ampersand),
      Some(b'\n') => self.single(Newline),
      Some(b'<') => self.maybe_callout_number(),
      Some(b'>') => self.single(GreaterThan),
      Some(b',') => self.single(Comma),
      Some(b'^') => self.single(Caret),
      Some(b'~') => self.single(Tilde),
      Some(b'_') => self.single(Underscore),
      Some(b'*') => self.single(Star),
      Some(b'.') => self.repeating(b'.', Dots),
      Some(b'/') => self.repeating(b'/', ForwardSlashes),
      Some(b'!') => self.single(Bang),
      Some(b'`') => self.single(Backtick),
      Some(b'+') => self.single(Plus),
      Some(b'[') => self.single(OpenBracket),
      Some(b']') => self.single(CloseBracket),
      Some(b'{') => self.single(OpenBrace),
      Some(b'}') => self.single(CloseBrace),
      Some(b'#') => self.single(Hash),
      Some(b'%') => self.single(Percent),
      Some(b'"') => self.single(DoubleQuote),
      Some(b'|') => self.single(Pipe),
      Some(b'\'') => self.single(SingleQuote),
      Some(b'\\') => self.single(Backslash),
      Some(ch) if ch.is_ascii_digit() => self.digits(),
      Some(ch) if ch == b';' || ch == b':' => self.maybe_term_delimiter(ch, at_line_start, breaker),
      Some(_) => self.word(),
      None => self.token(Eof, self.offset(), self.offset()),
    }
  }

  fn offset(&self) -> usize {
    self.src.len() - self.bytes.len() - self.peek.is_some() as usize // O(1) √
  }

  fn advance(&mut self) -> Option<u8> {
    let next = std::mem::replace(&mut self.peek, self.bytes.next());
    self.at_line_start = matches!(next, Some(b'\n'));
    next
  }

  pub fn skip(&mut self, n: usize) -> Option<u8> {
    debug_assert!(n > 1);
    let next = std::mem::replace(&mut self.peek, self.bytes.nth(n - 1));
    self.at_line_start = matches!(next, Some(b'\n'));
    next
  }

  fn advance_if(&mut self, c: u8) -> bool {
    if self.peek == Some(c) {
      self.advance();
      true
    } else {
      false
    }
  }

  fn advance_while(&mut self, c: u8) -> usize {
    while self.advance_if(c) {}
    self.offset()
  }

  fn single(&self, kind: TokenKind) -> Token<'src> {
    let end = self.offset();
    let start = end - 1;
    self.token(kind, start, end)
  }

  fn repeating(&mut self, c: u8, kind: TokenKind) -> Token<'src> {
    let start = self.offset() - 1;
    let end = self.advance_while(c);
    self.token(kind, start, end)
  }

  fn digits(&mut self) -> Token<'src> {
    let start = self.offset() - 1;
    let end = self.advance_while_with(|c| c.is_ascii_digit());
    self.token(Digits, start, end)
  }

  fn advance_while_with(&mut self, f: impl Fn(u8) -> bool) -> usize {
    while self.peek.map_or(false, &f) {
      self.advance();
    }
    self.offset()
  }

  fn advance_to_word_boundary(&mut self, with_at: bool) -> usize {
    self.advance_until_one_of(&[
      b' ',
      b'\t',
      b'\n',
      b':',
      b';',
      b'<',
      b'>',
      b',',
      b'^',
      b'_',
      b'~',
      b'*',
      b'!',
      b'`',
      b'+',
      b'.',
      b'[',
      b']',
      b'{',
      b'}',
      b'=',
      b'"',
      b'\'',
      b'\\',
      b'%',
      b'#',
      b'&',
      if with_at { b'@' } else { b'&' },
    ])
  }

  fn word(&mut self) -> Token<'src> {
    let start = self.offset() - 1;
    let end = self.advance_to_word_boundary(true);
    // PERF: if i feel clear about the safety of how i move across
    // bytes and word boundaries, i could change all of these to get_unchecked
    let lexeme = &self.src[start..end];

    // special cases
    match self.peek {
      // macros
      Some(b':') if !self.peek_term_delimiter() => {
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
      Some(b'@') => {
        self.advance();
        let domain_end = self
          .advance_while_with(|c| c.is_ascii_alphanumeric() || c == b'.' || c == b'-' || c == b'_');
        let domain = &self.src[end + 1..domain_end];
        if domain.len() > 3 && domain.contains('.') && !self.peek_is(b'@') {
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
    self.bytes = self.src[self.offset() - n..].bytes();
    self.peek = self.bytes.next();
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
        | "toc"
        | "xref"
    )
  }

  fn advance_until(&mut self, stop: u8) {
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

  fn advance_until_one_of(&mut self, chars: &[u8]) -> usize {
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

  fn advance_while_one_of(&mut self, chars: &[u8]) {
    loop {
      match self.peek {
        Some(c) if chars.contains(&c) => {}
        _ => break,
      }
      self.advance();
    }
  }

  fn whitespace(&mut self) -> Token<'src> {
    let start = self.offset() - 1;
    self.advance_while_one_of(&[b' ', b'\t']);
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

  fn maybe_term_delimiter(
    &mut self,
    ch: u8,
    at_line_start: bool,
    breaker: Option<TokenKind>,
  ) -> Token<'src> {
    let kind = if ch == b':' { Colon } else { SemiColon };
    if at_line_start || self.peek != Some(ch) {
      return self.single(kind);
    }
    if breaker == Some(kind) {
      self.pattern_breaker = Some(kind); // propagate the pattern breaker
      return self.single(kind);
    }
    let mut c = self.bytes.clone();
    match c.next() {
      None | Some(b' ' | b'\n' | b'\t') => {
        self.advance();
        let end = self.offset();
        return self.token(TermDelimiter, end - 2, end);
      }
      Some(n) if ch == b':' && n == b':' => {
        let mut num_colons = 3;
        let mut next = c.next();
        if next == Some(b':') {
          num_colons += 1;
          next = c.next();
        }
        if matches!(next, None | Some(b' ' | b'\n' | b'\t')) {
          self.skip(num_colons - 1);
          let end = self.offset();
          return self.token(TermDelimiter, end - num_colons, end);
        }
      }
      _ => {}
    }
    // if we get here, we've determined there are repeating colons or semicolons
    // but the pattern is NOT a term delimiter, so we set the pattern breaker so
    // that the next time around we don't find a term delimiter
    self.pattern_breaker = Some(kind);
    self.single(kind)
  }

  fn peek_term_delimiter(&self) -> bool {
    let mut c = self.bytes.clone();
    if c.next() != Some(b':') {
      return false;
    }
    matches!(
      (c.next(), c.next(), c.next()),
      (Some(b' ' | b'\t' | b'\n') | None, _, _)
        | (Some(b':'), Some(b' ' | b'\t' | b'\n') | None, _)
        | (Some(b':'), Some(b':'), Some(b' ' | b'\t' | b'\n') | None)
    )
  }

  fn maybe_callout_number(&mut self) -> Token<'src> {
    let start = self.offset() - 1;
    match self.peek {
      Some(c) if c.is_ascii_digit() => {
        self.advance();
        while self.peek.map_or(false, |c| c.is_ascii_digit()) {
          self.advance();
        }
        if self.peek == Some(b'>') {
          self.advance();
          return self.token(CalloutNumber, start, self.offset());
        } else {
          self.reverse_by(self.offset() - start - 1);
        }
      }
      Some(b'.') => {
        self.advance();
        if self.peek == Some(b'>') {
          self.advance();
          return self.token(CalloutNumber, start, self.offset());
        } else {
          self.reverse_by(self.offset() - start - 1);
        }
      }
      Some(b'!') => {
        let mut peek = self.bytes.clone();
        match (peek.next(), peek.next(), peek.next()) {
          (Some(b'-'), Some(b'-'), Some(b'.')) => {
            if let (Some(b'-'), Some(b'-'), Some(b'>')) = (peek.next(), peek.next(), peek.next()) {
              self.skip(7); // lexeme is exactly `<!--.-->`
              return self.token(CalloutNumber, start, self.offset());
            }
          }
          (Some(b'-'), Some(b'-'), Some(c)) if c.is_ascii_digit() => {
            let mut num_digits = 1;
            loop {
              match peek.next() {
                Some(c) if c.is_ascii_digit() => num_digits += 1,
                Some(b'-') => break,
                _ => return self.single(LessThan),
              }
            }
            if let (Some(b'-'), Some(b'>')) = (peek.next(), peek.next()) {
              self.skip(num_digits + 6);
              return self.token(CalloutNumber, start, self.offset());
            }
          }
          _ => {}
        }
      }
      _ => {}
    }
    self.single(LessThan)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::token::TokenKind;
  use ast::SourceLocation;
  use test_utils::{adoc, assert_eq};

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
        assert_eq!(lexer.next_token(), expected_token, from: input);
        index = end;
      }
      assert_eq!(lexer.next_token().kind, Eof);
    }
  }

  #[test]
  fn test_tokens_full() {
    let input = "&^foobar[//";
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
        kind: TokenKind::ForwardSlashes,
        loc: SourceLocation::new(9, 11),
        lexeme: "//",
      }
    );
    assert_eq!(
      lexer.next_token(),
      Token {
        kind: TokenKind::Eof,
        loc: SourceLocation::new(11, 11),
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
