use std::str::Chars;

use bumpalo::collections::String;
use bumpalo::Bump;

use super::source_location::SourceLocation;
use super::token::{Token, TokenKind, TokenKind::*, TokenValue};

pub struct Lexer<'alloc> {
  allocator: &'alloc Bump,
  src: &'alloc str,
  chars: Chars<'alloc>,
  peek: Option<char>,
}

impl<'alloc> Lexer<'alloc> {
  pub fn new(allocator: &'alloc Bump, src: &'alloc str) -> Lexer<'alloc> {
    Lexer {
      allocator,
      src,
      chars: src.chars(),
      peek: None,
    }
  }

  fn next_char(&mut self) -> Option<char> {
    self.peek.take().or_else(|| self.chars.next())
  }

  pub fn next_token(&mut self) -> Token {
    match self.next_char() {
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
      Some('\'') => self.single(SingleQuote),
      Some('\\') => self.single(Backslash),
      Some(_) => self.word(),
      None => Token {
        kind: Eof,
        loc: SourceLocation::new(self.offset(), self.offset()),
        value: TokenValue::None,
      },
    }
  }

  fn offset(&self) -> usize {
    self.src.len() - self.chars.as_str().len() - self.peek.is_some() as usize // O(1) √
  }

  pub fn loc(&self) -> SourceLocation {
    SourceLocation::from(self.offset())
  }

  fn single(&self, kind: TokenKind) -> Token {
    let offset = self.offset();
    Token {
      kind,
      loc: SourceLocation::new(offset - 1, offset),
      value: TokenValue::None,
    }
  }

  fn word(&mut self) -> Token {
    let start = self.offset() - 1;
    self.advance_until_one_of(&[
      ' ', '\t', '\n', ':', ';', '<', '>', ',', '^', '_', '~', '*', '!', '`', '+', '.', '[', ']',
      '=', '"', '\'', '\\', '%', '#', '&',
    ]);
    let end = self.offset();
    let word = &self.src[start..end];
    Token {
      kind: Word,
      loc: SourceLocation::new(start, end),
      value: TokenValue::String(String::from_str_in(word, self.allocator)),
    }
  }

  fn advance_until_one_of(&mut self, chars: &[char]) {
    loop {
      match self.next_char() {
        Some(c) if chars.contains(&c) => {
          self.peek = Some(c);
          break;
        }
        None => break,
        _ => {}
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    source_location::SourceLocation,
    token::{TokenKind, TokenValue},
  };
  use bumpalo::collections::String;

  // retrieve a line, do i actually need to hold the str?
  // render an err message (not sure seq)
  // lol evaluator

  #[test]
  fn test_tokens() {
    let bump = &Bump::new();
    let input = "&^foobar[";
    let mut lexer = Lexer::new(bump, input);
    assert_eq!(lexer.chars.as_str(), input);
    assert_eq!(
      lexer.next_token(),
      Token {
        kind: TokenKind::Ampersand,
        loc: SourceLocation::new(0, 1),
        value: TokenValue::None,
      }
    );
    assert_eq!(
      lexer.next_token(),
      Token {
        kind: TokenKind::Caret,
        loc: SourceLocation::new(1, 2),
        value: TokenValue::None,
      }
    );
    assert_eq!(
      lexer.next_token(),
      Token {
        kind: TokenKind::Word,
        loc: SourceLocation::new(2, 8),
        value: TokenValue::String(String::from_str_in("foobar", bump)),
      }
    );
    assert_eq!(
      lexer.next_token(),
      Token {
        kind: TokenKind::OpenBracket,
        loc: SourceLocation::new(8, 9),
        value: TokenValue::None,
      }
    );
    assert_eq!(
      lexer.next_token(),
      Token {
        kind: TokenKind::Eof,
        loc: SourceLocation::new(9, 9),
        value: TokenValue::None,
      }
    );
  }
}
