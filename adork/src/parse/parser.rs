use core::result::Result as CoreResult;
use std::collections::VecDeque;
use std::fs::File;

use crate::ast;
use crate::ast::DocContent;
use crate::lexer::Lexer;
use crate::tok::{self, Token, TokenType};

use super::diagnostic::Diagnostic;
use super::Result;

#[derive(Debug)]
pub struct Parser {
  pub(super) lexer: Lexer,
  pub(super) document: ast::Document,
  pub(super) peeked_block: Option<tok::Block>,
  pub(super) errors: Vec<Diagnostic>,
  pub(super) warnings: Vec<Diagnostic>,
  pub(super) bail: bool, // todo: naming...
}

pub struct ParseResult {
  pub document: ast::Document,
  pub warnings: Vec<Diagnostic>,
}

impl From<Diagnostic> for Vec<Diagnostic> {
  fn from(diagnostic: Diagnostic) -> Self {
    vec![diagnostic]
  }
}

impl Parser {
  pub fn new(lexer: Lexer) -> Parser {
    Parser {
      lexer,
      document: ast::Document {
        header: None,
        content: DocContent::Blocks(vec![]),
      },
      peeked_block: None,
      errors: vec![],
      warnings: vec![],
      bail: true,
    }
  }

  pub fn from_file(file: File, path: Option<impl Into<String>>) -> Self {
    Parser::new(Lexer::from_file(file, path))
  }

  pub fn parse(mut self) -> CoreResult<ParseResult, Vec<Diagnostic>> {
    self.document.header = self.parse_document_header()?;

    // while let Some(block) = self.read_block() {
    //   if block.starts_section() {
    //     self.document.content.ensure_sectioned();
    //     self.parse_section(block)?;
    //   } else {
    //     self.parse_block(block)?;
    //   }
    // }

    Ok(ParseResult {
      document: self.document,
      warnings: vec![],
    })
  }

  pub fn parse_section(&mut self, _first_block: tok::Block) -> Result<ast::Section> {
    todo!("parse section")
  }

  pub(crate) fn lexeme_string(&self, token: &Token) -> String {
    self.lexer.string(token)
  }

  pub(crate) fn lexeme_str(&self, token: &Token) -> &str {
    self.lexer.lexeme(token)
  }

  pub(super) fn expect_group<const N: usize>(
    &mut self,
    expected: [TokenType; N],
    msg: &'static str,
    line: &mut tok::Line,
  ) -> Result<Option<[Token; N]>> {
    for (i, token_type) in expected.into_iter().enumerate() {
      match line.nth_token(i) {
        Some(token) if token.token_type == token_type => {}
        _ => {
          self.err_expected_token(line.nth_token(0), msg)?;
          return Ok(None);
        }
      }
    }

    let tokens: [Token; N] = std::array::from_fn(|_| line.consume_current().unwrap());
    Ok(Some(tokens))
  }

  pub(super) fn expect_each<const N: usize>(
    &mut self,
    expected: [(TokenType, &'static str); N],
    line: &mut tok::Line,
  ) -> Result<Option<[Token; N]>> {
    for (i, (token_type, msg)) in expected.into_iter().enumerate() {
      match line.nth_token(i) {
        Some(token) if token.token_type == token_type => {}
        token => {
          self.err_expected_token(token.or(line.nth_token(i.saturating_sub(1))), msg)?;
          return Ok(None);
        }
      }
    }

    let tokens: [Token; N] = std::array::from_fn(|_| line.consume_current().unwrap());
    Ok(Some(tokens))
  }

  pub(super) fn expect(
    &mut self,
    token_type: TokenType,
    line: &mut tok::Line,
    msg: &str,
  ) -> Result<Option<Token>> {
    match line.current_token() {
      Some(token) if token.token_type == token_type => Ok(Some(line.consume_current().unwrap())),
      token => {
        self.err_expected_token(token, msg)?;
        Ok(None)
      }
    }
  }

  pub(crate) fn read_line(&mut self) -> Option<tok::Line> {
    if self.lexer.is_eof() {
      return None;
    }

    let mut tokens = vec![];
    while !self.lexer.current_is(b'\n') && !self.lexer.is_eof() {
      tokens.push(self.lexer.next().unwrap());
    }
    self.lexer.consume_newline();

    Some(tok::Line::new(tokens))
  }

  pub(crate) fn read_block(&mut self) -> Option<tok::Block> {
    if let Some(block) = self.peeked_block.take() {
      return Some(block);
    }

    self.lexer.consume_empty_lines();
    if self.lexer.is_eof() {
      return None;
    }

    let mut lines = VecDeque::new();
    while let Some(line) = self.read_line() {
      lines.push_back(line);
      if self.lexer.current_is(b'\n') {
        break;
      }
    }

    debug_assert!(!lines.is_empty());
    Some(tok::Block::new(lines))
  }

  pub(super) fn restore_block(&mut self, block: tok::Block) {
    self.peeked_block = Some(block);
  }
}

impl From<&'static str> for Parser {
  fn from(static_str: &'static str) -> Self {
    Parser::new(Lexer::from(static_str))
  }
}

impl From<String> for Parser {
  fn from(string: String) -> Self {
    Parser::new(Lexer::from(string))
  }
}

// tests

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_read_line() {
    let input = "hello world\ngoodbye\n\nfoo\n";
    let mut parser = Parser::from(input);

    let hello_world = parser.read_line().unwrap();
    assert_eq!(parser.lexeme_str(&hello_world.tokens[0]), "hello");
    assert_eq!(hello_world.tokens.len(), 3);

    let goodbye = parser.read_line().unwrap();
    assert_eq!(parser.lexeme_str(&goodbye.tokens[0]), "goodbye");
    assert_eq!(goodbye.tokens.len(), 1);

    assert_eq!(parser.read_line().unwrap().tokens.len(), 0); // empty line

    let foo = parser.read_line().unwrap();
    assert_eq!(parser.lexeme_str(&foo.tokens[0]), "foo");
    assert_eq!(foo.tokens.len(), 1);

    assert!(parser.read_line().is_none()); // eof
  }

  #[test]
  fn test_read_blocks() {
    let input = "hello\n\ngoodbye\n";
    let mut parser = Parser::from(input);
    assert!(parser.read_block().is_some());
    assert!(parser.read_block().is_some());
    assert!(parser.read_block().is_none());

    let input = "// comment\nhello\n\ngoodbye\n";
    let mut parser = Parser::from(input);
    assert!(parser.read_block().is_some());
    assert!(parser.read_block().is_some());
    assert!(parser.read_block().is_none());
  }
}
