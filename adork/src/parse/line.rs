use std::collections::VecDeque;

use crate::parse::Parser;
use crate::token::{Token, TokenType, TokenType::*};

#[derive(Debug, PartialEq, Eq)]
pub struct Line {
  pub tokens: VecDeque<Token>,
  // TODO: probaby can ditch this...
  current_token_loc: Option<(usize, usize)>,
}

impl Iterator for Line {
  type Item = Token;

  fn next(&mut self) -> Option<Self::Item> {
    self.tokens.pop_front()
  }
}

impl Line {
  pub fn new(tokens: Vec<Token>) -> Line {
    let tokens = VecDeque::from(tokens);
    let current_token_loc = tokens.front().map(|token| (token.start, token.end));
    Line {
      tokens,
      current_token_loc,
    }
  }

  pub fn is_empty(&self) -> bool {
    self.tokens.len() == 0
  }

  pub fn is_emptyish(&self) -> bool {
    self.tokens.len() == 0 || (self.tokens.len() == 1 && self.tokens[0].is(Newline))
  }

  pub fn remove_all(&mut self, token_type: TokenType) {
    self.tokens.retain(|token| !token.is(token_type))
  }

  pub fn current_token(&self) -> Option<&Token> {
    self.tokens.front()
  }

  pub fn consume<const N: usize>(&mut self) -> [Option<Token>; N] {
    std::array::from_fn(|_| self.consume_current())
  }

  pub fn consume_current(&mut self) -> Option<Token> {
    if let Some(token) = self.tokens.pop_front() {
      self.current_token_loc = Some((token.start, token.end));
      Some(token)
    } else {
      None
    }
  }

  pub fn consume_if(&mut self, token_type: TokenType) -> Option<Token> {
    match self.current_token() {
      Some(token) if token.is(token_type) => self.consume_current(),
      _ => None,
    }
  }

  pub fn consume_if_not(&mut self, token_type: TokenType) -> Option<Token> {
    match self.current_token() {
      Some(token) if !token.is(token_type) => self.consume_current(),
      _ => None,
    }
  }

  pub fn discard_until(&mut self, token_type: TokenType) {
    while let Some(token) = self.current_token() {
      if token.is(token_type) {
        break;
      }
      self.consume_current();
    }
  }

  pub fn discard_until_one_of(&mut self, token_types: &[TokenType]) {
    while let Some(token) = self.current_token() {
      if token_types.contains(&token.token_type) {
        break;
      }
      self.consume_current();
    }
  }

  pub fn peek_token(&self) -> Option<&Token> {
    self.tokens.get(1)
  }

  pub fn last_token(&self) -> Option<&Token> {
    self.tokens.back()
  }

  pub fn nth_token(&self, n: usize) -> Option<&Token> {
    self.tokens.get(n)
  }

  pub fn nth_token_is(&self, n: usize, token_type: TokenType) -> bool {
    match self.tokens.get(n) {
      Some(token) => token.is(token_type),
      None => false,
    }
  }

  pub fn nth_token_one_of(&self, n: usize, token_types: &[TokenType]) -> bool {
    match self.tokens.get(n) {
      Some(token) => token_types.contains(&token.token_type),
      None => false,
    }
  }

  pub fn current_is(&self, token_type: TokenType) -> bool {
    self.starts(token_type)
  }

  pub fn starts(&self, token_type: TokenType) -> bool {
    if self.tokens.len() == 0 {
      return false;
    }
    self.tokens[0].token_type == token_type
  }

  pub fn contains(&self, token_type: TokenType) -> bool {
    for token in &self.tokens {
      if token.token_type == token_type {
        return true;
      }
    }
    false
  }

  pub fn contains_any(&self, token_types: &[TokenType]) -> bool {
    for token in &self.tokens {
      for token_type in token_types {
        if token.token_type == *token_type {
          return true;
        }
      }
    }
    false
  }

  pub fn starts_with_one_of(&self, token_types: &[TokenType]) -> bool {
    for token_type in token_types {
      if self.starts(*token_type) {
        return true;
      }
    }
    false
  }

  pub fn starts_with_seq(&self, token_types: &[TokenType]) -> bool {
    if token_types.len() == 0 {
      return false;
    }
    if self.tokens.len() < token_types.len() {
      return false;
    }
    for (i, token_type) in token_types.iter().enumerate() {
      if self.tokens[i].token_type != *token_type {
        return false;
      }
    }
    true
  }

  pub fn is_header(&self, len: usize) -> bool {
    if !self.starts_with_seq(&[EqualSigns, Whitespace]) {
      return false;
    }
    self.current_token().unwrap().len() == len
  }

  pub fn to_string(&mut self, parser: &Parser) -> String {
    let mut s = String::with_capacity(self.len());
    while let Some(token) = self.consume_current() {
      s.push_str(parser.lexeme_str(&token));
    }
    s
  }

  pub fn to_string_until(&mut self, token_type: TokenType, parser: &Parser) -> String {
    let mut s = String::new();
    while let Some(token) = self.consume_if_not(token_type) {
      s.push_str(parser.lexeme_str(&token));
    }
    s
  }

  pub fn len(&self) -> usize {
    let end = self.tokens.back().map(|token| token.end).unwrap_or(0);
    let start = self.tokens.front().map(|token| token.start).unwrap_or(0);
    end.saturating_sub(start)
  }
}
