use crate::token::Token;
use crate::token::TokenType::{self, *};
use std::io::Read;
use std::{fmt, str};

const BUFFER_SIZE: usize = 4096;

pub struct Lexer<R: Read> {
  buffer: [u8; BUFFER_SIZE],
  buffer_index: usize,
  buffer_size: usize,
  index: usize,
  reader: R,
  source: Vec<u8>,
  prev: u8,
  current: Option<u8>,
  peek: Option<u8>,
}

impl<R: Read> Lexer<R> {
  pub fn new(reader: R) -> Lexer<R> {
    Lexer::with_capacity(reader, BUFFER_SIZE)
  }

  pub fn new_from(input: &str) -> Lexer<&[u8]> {
    Lexer::with_capacity(input.as_bytes(), input.len())
  }

  pub fn with_capacity(reader: R, capacity: usize) -> Lexer<R> {
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

  pub fn lexeme(&self, token: &Token) -> &str {
    unsafe { std::str::from_utf8_unchecked(&self.source[token.start..token.end]) }
  }

  pub fn string(&self, token: &Token) -> String {
    self.lexeme(token).to_string()
  }

  pub fn is_eof(&self) -> bool {
    self.current.is_none()
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
    self.advance_until_one_of(&[b' ', b'\t', b'\n', b':', b';', b'<', b'>']);
    self.advance();
    Token::new(Word, start, self.index)
  }

  fn starts_comment(&self) -> bool {
    if self.prev != b'\n' && self.prev != b'\0' {
      false
    } else if self.peek != Some(b'/') {
      false
    } else {
      true
    }
  }
}

impl<R: Read> Iterator for Lexer<R> {
  type Item = Token;

  fn next(&mut self) -> Option<Self::Item> {
    if self.current.is_none() {
      return None;
    }
    let token = match self.current.unwrap() {
      b'=' => self.repeating(b'=', EqualSigns),
      b'\n' => self.repeating(b'\n', Newlines),
      b' ' | b'\t' => self.whitespace(),
      b'/' if self.starts_comment() => self.comment(),
      b':' => self.single(Colon),
      b';' => self.single(SemiColon),
      b'<' => self.single(LessThan),
      b'>' => self.single(GreaterThan),
      _ => self.word(),
    };
    Some(token)
  }
}

impl<R: Read> fmt::Debug for Lexer<R> {
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
        "foo;bar",
        vec![(Word, "foo"), (SemiColon, ";"), (Word, "bar")],
      ),
      ("Foobar\n\n", vec![(Word, "Foobar"), (Newlines, "\n\n")]),
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
          (Newlines, "\n"),
          (EqualSigns, "="),
          (Whitespace, " "),
          (Word, "Document"),
          (Whitespace, " "),
          (Word, "Title"),
          (Newlines, "\n"),
          (Word, "Kismet"),
          (Whitespace, " "),
          (Word, "R."),
          (Whitespace, " "),
          (Word, "Lee"),
          (Whitespace, " "),
          (LessThan, "<"),
          (Word, "kismet@asciidoctor.org"),
          (GreaterThan, ">"),
          (Newlines, "\n"),
          (Colon, ":"),
          (Word, "description"),
          (Colon, ":"),
          (Whitespace, " "),
          (Word, "The"),
          (Whitespace, " "),
          (Word, "document's"),
          (Whitespace, " "),
          (Word, "description."),
          (Newlines, "\n"),
          (Colon, ":"),
          (Word, "sectanchors"),
          (Colon, ":"),
          (Newlines, "\n"),
          (Colon, ":"),
          (Word, "url-repo"),
          (Colon, ":"),
          (Whitespace, " "),
          (Word, "https"),
          (Colon, ":"),
          (Word, "//my-git-repo.com"),
          (Newlines, "\n\n"),
          (Word, "The"),
          (Whitespace, " "),
          (Word, "document"),
          (Whitespace, " "),
          (Word, "body"),
          (Whitespace, " "),
          (Word, "starts"),
          (Whitespace, " "),
          (Word, "here."),
          (Newlines, "\n"),
        ],
      ),
    ];
    for (input, expected) in cases {
      println!();
      let mut lexer = Lexer::<&[u8]>::new_from(input);
      let mut index = 0;
      for (token_type, lexeme) in expected {
        let start = index;
        let end = start + lexeme.len();
        let expected_token = Token::new(token_type, start, end);
        assert_eq!(lexer.next(), Some(expected_token.clone()));
        assert_eq!(lexer.lexeme(&expected_token), lexeme);
        print!("{}", lexeme);
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
      let mut lexer = Lexer::<&[u8]>::new_from(input);
      for (expected_token, lexeme) in &expected {
        assert_eq!(lexer.next(), Some(expected_token.clone()));
        assert_eq!(lexer.lexeme(expected_token), *lexeme);
      }
      assert_eq!(lexer.next(), None);
    }
  }
}
