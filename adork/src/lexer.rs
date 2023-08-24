use std::{fmt, fs::File, str};

use crate::err::SourceLocation;
use crate::reader::Reader;
use crate::tok::Token;
use crate::tok::TokenType::{self, *};

const BUFFER_SIZE: usize = 4096;

pub struct Lexer {
  buffer: [u8; BUFFER_SIZE],
  buffer_index: usize,
  buffer_size: usize,
  index: usize,
  reader: Reader,
  source: Vec<u8>,
  prev: u8,
  current: Option<u8>,
  peek: Option<u8>,
}

impl Lexer {
  pub fn with_capacity(reader: Reader, capacity: usize) -> Lexer {
    let mut lexer = Lexer {
      buffer: [0; BUFFER_SIZE],
      buffer_index: 0,
      buffer_size: 0,
      index: 0,
      reader,
      source: Vec::with_capacity(capacity),
      prev: b'\0',
      current: None,
      peek: None,
    };

    // fill current and peek
    lexer.advance();
    lexer.advance();

    // initialize index
    lexer.index = 0;

    lexer
  }

  pub fn from_file(file: File, path: Option<impl Into<String>>) -> Self {
    Lexer::from(Reader::from_file(file, path))
  }

  pub fn lexeme(&self, token: &Token) -> &str {
    unsafe { std::str::from_utf8_unchecked(&self.source[token.start..token.end]) }
  }

  pub fn string(&self, token: &Token) -> String {
    self.lexeme(token).to_string()
  }

  #[cfg(test)]
  pub fn print_peek(&self) {
    match self.peek {
      Some(byte) => println!("peek: `{}`", byte.escape_ascii()),
      None => println!("peek: None"),
    }
  }

  pub fn is_eof(&self) -> bool {
    self.current.is_none()
  }

  pub fn current_location(&self) -> SourceLocation {
    SourceLocation {
      start: self.index,
      end: self.index,
      token_type: None,
      is_exact: false,
    }
  }

  pub fn current_is(&self, ch: u8) -> bool {
    match self.current {
      Some(current) => current == ch,
      None => false,
    }
  }

  pub fn consume_newline(&mut self) -> bool {
    if self.current_is(b'\n') {
      self.advance();
      true
    } else {
      false
    }
  }

  pub fn consume_empty_lines(&mut self) {
    while self.consume_newline() {}
  }

  pub fn line_number(&self, location: usize) -> usize {
    let (line_number, _) = self.line_number_with_offset(location);
    line_number
  }

  pub fn line_number_with_offset(&self, location: usize) -> (usize, usize) {
    let mut line_number = 1;
    let mut offset = 0;
    for byte in &self.source[0..location] {
      if *byte == b'\n' {
        offset += 1;
        line_number += 1;
      }
    }
    (line_number, offset)
  }

  pub fn line_of(&self, location: usize) -> &str {
    let mut start = location;
    let mut end = location;

    loop {
      if start == 0 {
        break;
      }
      if self.source[start] == b'\n' && start != location {
        start += 1;
        break;
      }
      start -= 1;
    }

    loop {
      if end == self.source.len() {
        break;
      }
      if self.source[end] == b'\n' {
        break;
      }
      end += 1;
    }

    unsafe { str::from_utf8_unchecked(&self.source[start..end]) }
  }

  fn ensure_buffer(&mut self) -> bool {
    if self.buffer_index >= self.buffer_size {
      self.buffer_size = self.reader.read(&mut self.buffer).unwrap_or(0);
      if self.buffer_size == 0 {
        return false; // EOF
      }
      self.buffer_index = 0;
    }
    true
  }

  fn advance(&mut self) -> Option<()> {
    let peek = match self.ensure_buffer() {
      true => {
        let byte = self.buffer[self.buffer_index];
        self.source.push(byte);
        self.buffer_index += 1;
        Some(byte)
      }
      false => None,
    };
    self.index += 1;
    self.prev = self.current.unwrap_or(b'\0');
    self.current = self.peek;
    self.peek = peek;
    Some(())
  }

  fn repeating(&mut self, ch: u8, token_type: TokenType) -> Token {
    let start = self.index;
    self.advance_while(ch);
    Token::new(token_type, start, self.index)
  }

  fn single(&mut self, token_type: TokenType) -> Token {
    let start = self.index;
    self.advance();
    Token::new(token_type, start, self.index)
  }

  fn advance_if(&mut self, ch: u8) -> bool {
    match self.current {
      Some(c) if c == ch => {
        self.advance();
        true
      }
      _ => false,
    }
  }

  fn advance_while(&mut self, ch: u8) {
    while self.advance_if(ch) {}
  }

  fn advance_unless_one_of(&mut self, chars: &[u8]) -> bool {
    match self.peek {
      Some(c) if !chars.contains(&c) => {
        self.advance();
        true
      }
      _ => false,
    }
  }

  fn advance_until(&mut self, ch: u8) {
    while self.advance_unless_one_of(&[ch]) {}
  }

  fn advance_until_one_of(&mut self, chars: &[u8]) {
    while self.advance_unless_one_of(chars) {}
  }

  fn advance_while_one_of(&mut self, chars: &[u8]) {
    while self.advance_if_one_of(chars) {}
  }

  fn advance_if_one_of(&mut self, chars: &[u8]) -> bool {
    match self.peek {
      Some(c) if chars.contains(&c) => {
        self.advance();
        true
      }
      _ => false,
    }
  }

  fn whitespace(&mut self) -> Token {
    let start = self.index;
    self.advance_while_one_of(&[b' ', b'\t']);
    self.advance();
    Token::new(Whitespace, start, self.index)
  }

  fn comment(&mut self) -> Token {
    let start = self.index;
    // TODO: block comments, testing if we have 2 more slashes
    self.advance_until(b'\n');
    self.advance();
    Token::new(CommentLine, start, self.index)
  }

  fn word(&mut self) -> Token {
    let start = self.index;
    // TODO: dot should end word, but breaks tests currently
    self.advance_until_one_of(&[
      b' ', b'\t', b'\n', b':', b';', b'<', b'>', b',', b'^', b'_', b'~', b'*', b'!', b'`', b'+',
    ]);
    self.advance();
    Token::new(Word, start, self.index)
  }

  fn starts_comment(&self) -> bool {
    if self.prev != b'\n' && self.prev != b'\0' {
      return false;
    }
    self.peek == Some(b'/')
  }
}

impl Iterator for Lexer {
  type Item = Token;

  fn next(&mut self) -> Option<Self::Item> {
    let token = match self.current? {
      b'=' => self.repeating(b'=', EqualSigns),
      b' ' | b'\t' => self.whitespace(),
      b'/' if self.starts_comment() => self.comment(),
      b'\n' => self.single(Newline),
      b':' => self.single(Colon),
      b';' => self.single(SemiColon),
      b'<' => self.single(LessThan),
      b'>' => self.single(GreaterThan),
      b',' => self.single(Comma),
      b'^' => self.single(Caret),
      b'~' => self.single(Tilde),
      b'_' => self.single(Underscore),
      b'*' => self.single(Star),
      b'.' => self.single(Dot),
      b'!' => self.single(Bang),
      b'`' => self.single(Backtick),
      b'+' => self.single(Plus),
      _ => self.word(),
    };
    Some(token)
  }
}

impl fmt::Debug for Lexer {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(
      f,
      r"
 Lexer {{
   buffer_index: {},
   buffer_size: {},
   index: {},
   source: {},
   current: {:?},
   peek: {:?},
 }}",
      self.buffer_index,
      self.buffer_size,
      self.index,
      String::from_utf8(self.source.clone()).unwrap(),
      self.current.map(|c| c as char),
      self.peek.map(|c| c as char),
    )
  }
}

impl From<Reader> for Lexer {
  fn from(reader: Reader) -> Self {
    let capacity = reader.capacity_hint().unwrap_or(BUFFER_SIZE);
    Lexer::with_capacity(reader, capacity)
  }
}

impl From<String> for Lexer {
  fn from(string: String) -> Self {
    Lexer::from(Reader::from(string))
  }
}

impl From<&'static str> for Lexer {
  fn from(static_str: &'static str) -> Self {
    Lexer::from(Reader::from(static_str))
  }
}

// tests

#[cfg(test)]
mod tests {
  use super::*;
  use indoc::indoc;

  #[test]
  fn test_tokens() {
    let cases = vec![
      ("==", vec![(EqualSigns, "==")]),
      ("===", vec![(EqualSigns, "===")]),
      ("// foo", vec![(CommentLine, "// foo")]),
      (
        "foo;bar,lol^~_*.!`+",
        vec![
          (Word, "foo"),
          (SemiColon, ";"),
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
        indoc! {"
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
          (Word, "R."),
          (Whitespace, " "),
          (Word, "Lee"),
          (Whitespace, " "),
          (LessThan, "<"),
          (Word, "kismet@asciidoctor.org"),
          (GreaterThan, ">"),
          (Newline, "\n"),
          (Colon, ":"),
          (Word, "description"),
          (Colon, ":"),
          (Whitespace, " "),
          (Word, "The"),
          (Whitespace, " "),
          (Word, "document's"),
          (Whitespace, " "),
          (Word, "description."),
          (Newline, "\n"),
          (Colon, ":"),
          (Word, "sectanchors"),
          (Colon, ":"),
          (Newline, "\n"),
          (Colon, ":"),
          (Word, "url-repo"),
          (Colon, ":"),
          (Whitespace, " "),
          (Word, "https"),
          (Colon, ":"),
          (Word, "//my-git-repo.com"),
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
          (Word, "here."),
          (Newline, "\n"),
        ],
      ),
    ];
    for (input, expected) in cases {
      let mut lexer = Lexer::from(input);
      let mut index = 0;
      for (token_type, lexeme) in expected {
        let start = index;
        let end = start + lexeme.len();
        let expected_token = Token::new(token_type, start, end);
        assert_eq!(lexer.next(), Some(expected_token.clone()));
        assert_eq!(lexer.lexeme(&expected_token), lexeme);
        index = end;
      }
      assert_eq!(lexer.next(), None);
    }
  }

  #[test]
  fn test_tokens_manually_asserting_indexes() {
    let cases = vec![
      ("==", vec![(Token::new(EqualSigns, 0, 2), "==")]),
      ("===", vec![(Token::new(EqualSigns, 0, 3), "===")]),
      (
        "// foobar",
        vec![(Token::new(CommentLine, 0, 9), "// foobar")],
      ),
    ];
    for (input, expected) in cases {
      let mut lexer = Lexer::from(input);
      for (expected_token, lexeme) in &expected {
        assert_eq!(lexer.next(), Some(expected_token.clone()));
        assert_eq!(lexer.lexeme(expected_token), *lexeme);
      }
      assert_eq!(lexer.next(), None);
    }
  }

  #[test]
  fn test_consume_empty_lines() {
    let input = "\n\n\n\n\n";
    let mut lexer = Lexer::from(input);
    lexer.consume_empty_lines();
    assert!(lexer.is_eof());
  }

  #[test]
  fn test_line_of() {
    let input = "foo\nbar\n\nbaz\n";
    let mut lexer = Lexer::from(input);
    while lexer.next().is_some() {}
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
    let input = indoc! {"
      = :
      foo

      ;
    "};
    let mut lexer = Lexer::from(input);

    assert_next_token_line(&mut lexer, 1, EqualSigns);
    assert_next_token_line(&mut lexer, 1, Whitespace);
    assert_next_token_line(&mut lexer, 1, Colon);
    assert_next_token_line(&mut lexer, 1, Newline);
    assert_next_token_line(&mut lexer, 2, Word);
    assert_next_token_line(&mut lexer, 2, Newline);
    assert_next_token_line(&mut lexer, 3, Newline);
  }

  fn assert_next_token_line(lexer: &mut Lexer, line: usize, expected_type: TokenType) {
    let token = lexer.next().unwrap();
    assert_eq!(token.token_type, expected_type);
    assert_eq!(lexer.line_number(token.start), line);
  }
}
