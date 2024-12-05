use std::fmt::{Debug, Formatter, Result};

use crate::internal::*;
use crate::variants::token::*;

pub struct SourceLexer<'arena> {
  pub bump: &'arena Bump,
  pub src: BumpVec<'arena, u8>,
  pub pos: u32,
  pub offset: u32,
  pub file: SourceFile,
  pub leveloffset: i8,
  pub max_include_depth: Option<u16>,
}

impl<'arena> SourceLexer<'arena> {
  pub const fn new(
    src: BumpVec<'arena, u8>,
    file: SourceFile,
    leveloffset: i8,
    max_include_depth: Option<u16>,
    bump: &'arena Bump,
  ) -> Self {
    Self {
      bump,
      src,
      pos: 0,
      offset: 0,
      leveloffset,
      file,
      max_include_depth,
    }
  }

  pub fn from_str(s: &str, file: SourceFile, bump: &'arena Bump) -> Self {
    Self::from_byte_slice(s.as_bytes(), file, bump)
  }

  pub fn from_byte_slice(bytes: &[u8], file: SourceFile, bump: &'arena Bump) -> Self {
    Self {
      bump,
      src: BumpVec::from_iter_in(bytes.iter().copied(), bump),
      pos: 0,
      offset: 0,
      leveloffset: 0,
      file,
      max_include_depth: None,
    }
  }

  pub fn next_token(&mut self) -> Option<Token<'arena>> {
    if let Some(token) = self.delimiter_line() {
      return Some(token);
    }
    let at_line_start = self.at_line_start();
    let byte = self.nth(0);
    self.pos += 1;
    match byte {
      Some(b'=') => Some(self.repeating(b'=', EqualSigns)),
      Some(b'-') => Some(self.repeating(b'-', Dashes)),
      Some(b' ' | b'\t') => Some(self.whitespace()),
      Some(b'&') => Some(self.single(Ampersand)),
      Some(b'\n') => Some(self.single(Newline)),
      Some(b'<') => Some(self.maybe_callout_number()),
      Some(b'>') => Some(self.single(GreaterThan)),
      Some(b',') => Some(self.single(Comma)),
      Some(b'^') => Some(self.single(Caret)),
      Some(b'~') => Some(self.single(Tilde)),
      Some(b'_') => Some(self.single(Underscore)),
      Some(b'*') => Some(self.single(Star)),
      Some(b'.') => Some(self.repeating(b'.', Dots)),
      Some(b'/') => Some(self.repeating(b'/', ForwardSlashes)),
      Some(b'!') => Some(self.single(Bang)),
      Some(b'`') => Some(self.single(Backtick)),
      Some(b'+') => Some(self.repeating(b'+', Plus)),
      Some(b'[') => Some(self.single(OpenBracket)),
      Some(b']') => Some(self.single(CloseBracket)),
      Some(b'{') => Some(self.maybe_attr_ref()),
      Some(b'}') => Some(self.single(CloseBrace)),
      Some(b'#') => Some(self.single(Hash)),
      Some(b'%') => Some(self.single(Percent)),
      Some(b'"') => Some(self.single(DoubleQuote)),
      Some(b'(') => Some(self.repeating(b'(', OpenParens)),
      Some(b')') => Some(self.repeating(b'(', CloseParens)),
      Some(b'|') => Some(self.single(Pipe)),
      Some(b'\'') => Some(self.single(SingleQuote)),
      Some(b'\\') => Some(self.single(Backslash)),
      Some(b) if b.is_ascii_digit() => Some(self.digits()),
      Some(b) if b == b';' || b == b':' => Some(self.maybe_term_delimiter(b, at_line_start)),
      Some(0xC2) if self.peek_is(0xA0) => Some(self.codepoint(2)), // no-break space
      Some(0xE2) if self.peek_bytes::<2>() == Some(b"\x80\xAF") => Some(self.codepoint(3)), // narrow no-break space
      Some(0xE2) if self.peek_bytes::<2>() == Some(b"\x80\x87") => Some(self.codepoint(3)), // figure space
      Some(0xE2) if self.peek_bytes::<2>() == Some(b"\x81\xA0") => Some(self.codepoint(3)), // word joiner
      Some(0xE3) if self.peek_bytes::<2>() == Some(b"\x80\x80") => Some(self.codepoint(3)), // ideographic space
      Some(0xEF) if self.peek_bytes::<2>() == Some(b"\xBB\xBF") => Some(self.codepoint(3)), // zero-width no-break space
      Some(_) => Some(self.word(at_line_start)),
      None => None,
    }
  }

  pub fn codepoint(&mut self, n: u32) -> Token<'arena> {
    debug_assert!(n > 1);
    self.pos += n - 1;
    self.token(Word, self.pos - n, self.pos)
  }

  pub fn peek(&self) -> Option<u8> {
    self.src.get(self.pos as usize).copied()
  }

  pub fn peek_n(&self, n: usize) -> Option<u8> {
    self.src.get(self.pos as usize + n).copied()
  }

  pub fn is_eof(&self) -> bool {
    self.pos == self.src.len() as u32
  }

  pub fn consume_empty_lines(&mut self) {
    while self.peek() == Some(b'\n') {
      self.advance();
    }
  }

  pub fn at_delimiter_line(&self) -> Option<(u32, u8)> {
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
    self.src.truncate(self.offset as usize);
  }

  pub fn raw_lines(&'arena self) -> impl Iterator<Item = &'arena str> {
    LinesIter { src: &self.src, start: 0, end: 0 }
  }

  pub fn line_of(&self, location: u32) -> BumpString<'arena> {
    let location = location - self.offset;
    let mut start = location;
    let mut end = location;

    while start > 0 && self.src[start as usize - 1] != b'\n' {
      start -= 1;
    }

    while end < self.src.len() as u32 && self.src[end as usize] != b'\n' {
      end += 1;
    }

    let str = std::str::from_utf8(&self.src[start as usize..end as usize]).unwrap();
    BumpString::from_str_in(str, self.bump)
  }

  pub fn line_number_with_offset(&self, location: u32) -> (u32, u32) {
    let mut line_number = 1;
    let mut offset: u32 = 0;
    for idx in 0..location {
      if self.src[idx as usize] == b'\n' {
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
    self.pos == 0 || self.src.get(self.pos as usize - 1) == Some(&b'\n')
  }

  fn at_empty_line(&self) -> bool {
    self.at_line_start() && self.peek_is(b'\n')
  }

  fn nth(&self, n: u32) -> Option<u8> {
    self.src.get((self.pos + n) as usize).copied()
  }

  fn delimiter_line(&mut self) -> Option<Token<'arena>> {
    let (len, _) = self.at_delimiter_line()?;
    let start = self.pos;
    self.skip(len);
    Some(self.token(DelimiterLine, start, start + len))
  }

  fn skip(&mut self, n: u32) {
    debug_assert!(n > 1);
    self.pos += n;
  }

  fn whitespace(&mut self) -> Token<'arena> {
    let start = self.pos - 1;
    self.advance_while_one_of(b" \t");
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

  fn word(&mut self, at_line_start: bool) -> Token<'arena> {
    let start = self.pos - 1;
    let end = self.advance_to_word_boundary(true);
    // PERF: if i feel clear about the safety of how i move across
    // bytes and word boundaries, i could change all of these to get_unchecked
    let lexeme = &self.src[start as usize..end as usize];

    // special cases
    match self.peek() {
      // directives
      Some(b':')
        if at_line_start
          && matches!(
            lexeme,
            b"include" | b"ifdef" | b"ifndef" | b"endif" | b"ifeval"
          )
          && self.remaining_len() > 4 =>
      {
        if self.peek_n(1) == Some(b':') && !self.peek_n(2).unwrap().is_ascii_whitespace() {
          self.advance();
          self.advance();
          return self.token(Directive, start, end + 2);
        }
      }

      // macros
      Some(b':') if !self.peek_term_delimiter() => {
        if let Some(n) = self.continues_uri_scheme(lexeme) {
          self.pos += n;
          return self.token(UriScheme, start, end + n);
        } else if self.is_macro_name(lexeme) {
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
        let domain = &self.src[end as usize + 1..domain_end as usize];
        if domain.len() > 3 && domain.contains(&b'.') && !self.peek_is(b'@') {
          return self.token(MaybeEmail, start, domain_end);
        }
        self.reverse_by(domain.len() as u32);
        let end = self.advance_to_word_boundary(false);
        return self.token(Word, start, end);
      }
      _ => {}
    }
    self.token(Word, start, end)
  }

  fn token(&self, kind: TokenKind, start: u32, end: u32) -> Token<'arena> {
    let str = if end == start {
      ""
    } else {
      std::str::from_utf8(&self.src[start as usize..end as usize]).unwrap()
    };
    Token {
      kind,
      loc: SourceLocation::new(start + self.offset, end + self.offset),
      lexeme: BumpString::from_str_in(str, self.bump),
    }
  }

  fn reverse_by(&mut self, n: u32) {
    self.pos -= n;
  }

  const fn is_macro_name(&self, lexeme: &[u8]) -> bool {
    matches!(
      lexeme,
      b"footnote"
        | b"image"
        | b"anchor"
        | b"icon"
        | b"kbd"
        | b"link"
        | b"pass"
        | b"btn"
        | b"menu"
        | b"toc"
        | b"xref"
    )
  }

  fn advance(&mut self) {
    self.pos += 1;
  }

  fn advance_if(&mut self, c: u8) -> bool {
    if self.peek() == Some(c) {
      self.advance();
      true
    } else {
      false
    }
  }

  fn advance_while(&mut self, c: u8) -> u32 {
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

  fn advance_while_with(&mut self, f: impl Fn(u8) -> bool) -> u32 {
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

  fn advance_until_one_of(&mut self, chars: &[u8]) -> u32 {
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

  fn advance_to_word_boundary(&mut self, with_at: bool) -> u32 {
    loop {
      match self.peek() {
        Some(b'@') if with_at => break,
        Some(
          b' ' | b'\t' | b'\n' | b':' | b';' | b'<' | b'>' | b',' | b'^' | b'_' | b'~' | b'*'
          | b'!' | b'?' | b'`' | b'+' | b'.' | b'[' | b']' | b'{' | b'}' | b'(' | b')' | b'='
          | b'|' | b'"' | b'\'' | b'\\' | b'%' | b'#' | b'&',
        ) => break,
        None => break,
        _ => {
          self.advance();
        }
      }
    }
    self.pos
  }

  fn maybe_term_delimiter(&mut self, ch: u8, at_line_start: bool) -> Token<'arena> {
    let kind = if ch == b':' { Colon } else { SemiColon };
    if self.pos > 1 && self.src[self.pos as usize - 2] == ch {
      return self.single(kind);
    }
    if at_line_start || self.peek() != Some(ch) {
      return self.single(kind);
    }

    let mut peek = self.src[self.pos as usize + 1..].iter();
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
    let mut peek = self.src[self.pos as usize + 1..].iter();
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

  // according to asciidoctor docs, attr ref names must:
  //   - be only a-z, A-Z, 0-9, -, and _
  //   - be at least one letter long
  fn maybe_attr_ref(&mut self) -> Token<'arena> {
    if self.pos >= 2 && self.src.get((self.pos - 2) as usize).copied() == Some(b'\\') {
      return self.single(OpenBrace);
    }
    let mut len: u32 = 0;
    let peek = self.src[self.pos as usize..].iter();
    for c in peek {
      match *c {
        b'}' => {
          if len == 0 {
            return self.single(OpenBrace);
          }
          let token = self.token(AttrRef, self.pos - 1, self.pos + len + 1);
          self.pos += len + 1;
          return token;
        }
        b'-' | b'_' => len += 1,
        c if c.is_ascii_alphanumeric() => len += 1,
        _ => return self.single(OpenBrace),
      }
    }
    self.single(OpenBrace)
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
        let mut peek = self.src[self.pos as usize + 1..].iter();
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

  fn remaining_len(&self) -> u32 {
    self.src.len() as u32 - self.pos
  }

  fn peek_bytes<const N: usize>(&self) -> Option<&[u8]> {
    self.src.get(self.pos as usize..self.pos as usize + N)
  }

  fn continues_uri_scheme(&self, lexeme: &[u8]) -> Option<u32> {
    match lexeme {
      b"http" | b"https" | b"ftp" | b"mailto" | b"irc"
        if self.peek_bytes::<3>() == Some(b"://") =>
      {
        Some(3)
      }
      b"file" if self.peek_bytes::<4>() == Some(b":///") => Some(4),
      _ => None,
    }
  }
}

impl Debug for SourceLexer<'_> {
  fn fmt(&self, f: &mut Formatter<'_>) -> Result {
    f.debug_struct("SourceLexer")
      .field("src", &String::from_utf8_lossy(&self.src))
      .field("pos", &self.pos)
      .field("offset", &self.offset)
      .field("leveloffset", &self.leveloffset)
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
  use test_utils::*;

  #[test]
  fn test_raw_lines() {
    let bump = Bump::new();
    let input = "hello\nworld\n\n";
    let lexer = SourceLexer::from_str(input, SourceFile::Tmp, &bump);
    let mut lines = lexer.raw_lines();
    expect_eq!(lines.next(), Some("hello"));
    expect_eq!(lines.next(), Some("world"));
    expect_eq!(lines.next(), Some(""));
    expect_eq!(lines.next(), None);
  }
}
