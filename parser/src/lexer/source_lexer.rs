use std::fmt::{Debug, Formatter, Result};

use crate::internal::*;
use crate::variants::token::*;

pub struct SourceLexer<'arena> {
  pub bump: &'arena Bump,
  pub src: BumpVec<'arena, u8>,
  pub pos: usize,
  pub offset: usize,
}

impl<'arena> SourceLexer<'arena> {
  pub fn new(src: BumpVec<'arena, u8>, bump: &'arena Bump) -> Self {
    Self { bump, src, pos: 0, offset: 0 }
  }

  pub fn from_str(s: &str, bump: &'arena Bump) -> Self {
    Self::from_byte_slice(s.as_bytes(), bump)
  }

  pub fn from_byte_slice(bytes: &[u8], bump: &'arena Bump) -> Self {
    Self {
      bump,
      src: BumpVec::from_iter_in(bytes.iter().copied(), bump),
      pos: 0,
      offset: 0,
    }
  }

  pub fn next_token(&mut self) -> Token<'arena> {
    if let Some(token) = self.delimiter_line() {
      return token;
    }
    let at_line_start = self.at_line_start();
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
      Some(ch) if ch == b';' || ch == b':' => self.maybe_term_delimiter(ch, at_line_start),
      Some(_) => self.word(),
      None => self.token(Eof, self.pos, self.pos),
    }
  }

  pub fn peek(&self) -> Option<u8> {
    self.src.get(self.pos).copied()
  }

  pub fn is_eof(&self) -> bool {
    self.pos == self.src.len()
  }

  pub fn consume_line(&mut self) -> Option<Line<'arena>> {
    if self.is_eof() {
      return None;
    }
    let start = self.pos;
    let mut _end = start;
    let mut tokens = Deq::new(self.bump);
    while !self.peek_is(b'\n') && !self.is_eof() {
      let token = self.next_token();
      _end = token.loc.end - self.offset;
      tokens.push(token);
    }
    if self.peek_is(b'\n') {
      self.advance();
    }
    Some(Line::new(tokens))
  }

  pub fn consume_empty_lines(&mut self) {
    while self.peek() == Some(b'\n') {
      self.advance();
    }
  }

  pub fn at_delimiter_line(&self) -> Option<(usize, u8)> {
    if !self.at_line_start()
      || self.is_eof()
      || !matches!(
        self.peek(),
        Some(b'_' | b'-' | b'*' | b'=' | b'.' | b'+' | b'/')
      )
    {
      return None;
    }
    let sequence = [
      self.nth(0),
      self.nth(1),
      self.nth(2),
      self.nth(3),
      self.nth(4),
    ];
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

  pub fn truncate(&mut self) {
    self.src.truncate(self.offset);
  }

  pub fn raw_lines(&'arena self) -> impl Iterator<Item = &'arena str> {
    LinesIter { src: &self.src, start: 0, end: 0 }
  }

  pub fn line_of(&self, location: usize) -> BumpString<'arena> {
    let location = location - self.offset;
    let mut start = location;
    let mut end = location;

    while start > 0 && self.src[start - 1] != b'\n' {
      start -= 1;
    }

    while end < self.src.len() && self.src[end] != b'\n' {
      end += 1;
    }

    let str = std::str::from_utf8(&self.src[start..end]).unwrap();
    BumpString::from_str_in(str, self.bump)
  }

  pub fn line_number_with_offset(&self, location: usize) -> (usize, usize) {
    let mut line_number = 1;
    let mut offset: usize = 0;
    for idx in 0..location {
      if self.src[idx] == b'\n' {
        offset = 0;
        line_number += 1;
      } else {
        offset += 1;
      }
    }
    (line_number, offset)
  }

  fn peek_is(&self, c: u8) -> bool {
    self.peek() == Some(c)
  }

  fn at_line_start(&self) -> bool {
    self.pos == 0 || self.src.get(self.pos - 1) == Some(&b'\n')
  }

  fn at_empty_line(&self) -> bool {
    self.at_line_start() && self.peek_is(b'\n')
  }

  fn nth(&self, n: usize) -> Option<u8> {
    self.src.get(self.pos + n).copied()
  }

  fn delimiter_line(&mut self) -> Option<Token<'arena>> {
    let (len, _) = self.at_delimiter_line()?;
    let start = self.pos;
    self.skip(len);
    Some(self.token(DelimiterLine, start, start + len))
  }

  fn skip(&mut self, n: usize) {
    debug_assert!(n > 1);
    self.pos += n;
  }

  fn whitespace(&mut self) -> Token<'arena> {
    let start = self.pos - 1;
    self.advance_while_one_of(&[b' ', b'\t']);
    self.token(Whitespace, start, self.pos)
  }

  fn single(&self, kind: TokenKind) -> Token<'arena> {
    let end = self.pos;
    let start = end - 1;
    self.token(kind, start, end)
  }

  fn repeating(&mut self, c: u8, kind: TokenKind) -> Token<'arena> {
    let start = self.pos - 1;
    let end = self.advance_while(c);
    self.token(kind, start, end)
  }

  fn digits(&mut self) -> Token<'arena> {
    let start = self.pos - 1;
    let end = self.advance_while_with(|c| c.is_ascii_digit());
    self.token(Digits, start, end)
  }

  fn word(&mut self) -> Token<'arena> {
    let start = self.pos - 1;
    let end = self.advance_to_word_boundary(true);
    // PERF: if i feel clear about the safety of how i move across
    // bytes and word boundaries, i could change all of these to get_unchecked
    let lexeme = &self.src[start..end];

    // special cases
    match self.peek() {
      // macros
      Some(b':') if !self.peek_term_delimiter() => {
        if self.is_macro_name(lexeme) {
          self.advance();
          return self.token(MacroName, start, end + 1);
          // ...checking for contiguous footnote `somethingfootnote:[]`
        } else if lexeme.last() == Some(&b'e') && lexeme.ends_with(b"footnote") {
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
        if domain.len() > 3 && domain.contains(&b'.') && !self.peek_is(b'@') {
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

  fn token(&self, kind: TokenKind, start: usize, end: usize) -> Token<'arena> {
    let str = if end == start {
      ""
    } else {
      std::str::from_utf8(&self.src[start..end]).unwrap()
    };
    Token {
      kind,
      loc: SourceLocation::new(start + self.offset, end + self.offset),
      lexeme: BumpString::from_str_in(str, self.bump),
    }
  }

  fn reverse_by(&mut self, n: usize) {
    self.pos -= n;
  }

  const fn is_macro_name(&self, lexeme: &[u8]) -> bool {
    matches!(
      lexeme,
      b"footnote"
        | b"image"
        | b"irc"
        | b"icon"
        | b"kbd"
        | b"link"
        | b"http"
        | b"https"
        | b"ftp"
        | b"mailto"
        | b"pass"
        | b"btn"
        | b"menu"
        | b"toc"
        | b"xref"
    )
  }

  fn advance(&mut self) -> Option<u8> {
    let byte = self.nth(0);
    self.pos += 1;
    byte
  }

  fn advance_if(&mut self, c: u8) -> bool {
    if self.peek() == Some(c) {
      self.advance();
      true
    } else {
      false
    }
  }

  fn advance_while(&mut self, c: u8) -> usize {
    while self.advance_if(c) {}
    self.pos
  }

  fn advance_while_one_of(&mut self, chars: &[u8]) {
    loop {
      match self.peek() {
        Some(c) if chars.contains(&c) => {}
        _ => break,
      }
      self.advance();
    }
  }

  fn advance_while_with(&mut self, f: impl Fn(u8) -> bool) -> usize {
    while self.peek().map_or(false, &f) {
      self.advance();
    }
    self.pos
  }

  fn advance_until(&mut self, stop: u8) {
    loop {
      match self.peek() {
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
      match self.peek() {
        Some(c) if chars.contains(&c) => break,
        None => break,
        _ => {
          self.advance();
        }
      }
    }
    self.pos
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
      b'|',
      b'"',
      b'\'',
      b'\\',
      b'%',
      b'#',
      b'&',
      if with_at { b'@' } else { b'&' },
    ])
  }

  fn maybe_term_delimiter(&mut self, ch: u8, at_line_start: bool) -> Token<'arena> {
    let kind = if ch == b':' { Colon } else { SemiColon };
    if self.pos > 1 && self.src[self.pos - 2] == ch {
      return self.single(kind);
    }
    if at_line_start || self.peek() != Some(ch) {
      return self.single(kind);
    }

    let mut peek = self.src[self.pos + 1..].iter();
    match peek.next() {
      None | Some(b' ' | b'\n' | b'\t') => {
        self.advance();
        let end = self.pos;
        return self.token(TermDelimiter, end - 2, end);
      }
      Some(n) if ch == b':' && n == &b':' => {
        let mut num_colons = 3;
        let mut next = peek.next();
        if next == Some(&b':') {
          num_colons += 1;
          next = peek.next();
        }
        if matches!(next, None | Some(b' ' | b'\n' | b'\t')) {
          self.skip(num_colons - 1);
          let end = self.pos;
          return self.token(TermDelimiter, end - num_colons, end);
        }
      }
      _ => {}
    }
    self.single(kind)
  }

  fn peek_term_delimiter(&self) -> bool {
    let mut peek = self.src[self.pos + 1..].iter();
    if peek.next() != Some(&b':') {
      return false;
    }
    matches!(
      (peek.next(), peek.next(), peek.next()),
      (Some(b' ' | b'\t' | b'\n') | None, _, _)
        | (Some(b':'), Some(b' ' | b'\t' | b'\n') | None, _)
        | (Some(b':'), Some(b':'), Some(b' ' | b'\t' | b'\n') | None)
    )
  }

  fn maybe_callout_number(&mut self) -> Token<'arena> {
    let start = self.pos - 1;
    match self.peek() {
      Some(c) if c.is_ascii_digit() => {
        self.advance();
        while self.peek().map_or(false, |c| c.is_ascii_digit()) {
          self.advance();
        }
        if self.peek() == Some(b'>') {
          self.advance();
          return self.token(CalloutNumber, start, self.pos);
        } else {
          self.reverse_by(self.pos - start - 1);
        }
      }
      Some(b'.') => {
        self.advance();
        if self.peek() == Some(b'>') {
          self.advance();
          return self.token(CalloutNumber, start, self.pos);
        } else {
          self.reverse_by(self.pos - start - 1);
        }
      }
      Some(b'!') => {
        let mut peek = self.src[self.pos + 1..].iter();
        match (peek.next(), peek.next(), peek.next()) {
          (Some(b'-'), Some(b'-'), Some(b'.')) => {
            if let (Some(b'-'), Some(b'-'), Some(b'>')) = (peek.next(), peek.next(), peek.next()) {
              self.skip(7); // lexeme is exactly `<!--.-->`
              return self.token(CalloutNumber, start, self.pos);
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
              return self.token(CalloutNumber, start, self.pos);
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

impl<'arena> Debug for SourceLexer<'arena> {
  fn fmt(&self, f: &mut Formatter<'_>) -> Result {
    f.debug_struct("SourceLexer")
      .field("src", &String::from_utf8_lossy(&self.src))
      .field("pos", &self.pos)
      .field("offset", &self.offset)
      .finish()
  }
}

#[derive(Debug)]
struct LinesIter<'a> {
  src: &'a [u8],
  start: usize,
  end: usize,
}

impl<'a> Iterator for LinesIter<'a> {
  type Item = &'a str;

  fn next(&mut self) -> Option<Self::Item> {
    while self.end < self.src.len() {
      if self.src[self.end] == b'\n' {
        let line = std::str::from_utf8(&self.src[self.start..self.end]).unwrap();
        self.end += 1;
        self.start = self.end;
        return Some(line);
      }
      self.end += 1;
    }
    None
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use test_utils::assert_eq;

  #[test]
  fn test_raw_lines() {
    let bump = Bump::new();
    let input = "hello\nworld\n\n";
    let lexer = SourceLexer::from_str(input, &bump);
    let mut lines = lexer.raw_lines();
    assert_eq!(lines.next(), Some("hello"));
    assert_eq!(lines.next(), Some("world"));
    assert_eq!(lines.next(), Some(""));
    assert_eq!(lines.next(), None);
  }
}
