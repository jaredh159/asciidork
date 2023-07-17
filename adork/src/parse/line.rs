use std::collections::VecDeque;

use super::Result;
use crate::err::{ParseErr, ParseErrLoc};
use crate::token::{Token, TokenType, TokenType::*};

#[derive(Debug, PartialEq, Eq)]
pub struct Line {
  pub tokens: VecDeque<Token>,
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

  pub fn consume_expecting_seq(&mut self, token_types: &[TokenType], msg: &str) -> Result<()> {
    for token_type in token_types {
      self.consume_expecting(*token_type, msg)?;
    }
    Ok(())
  }

  pub fn consume_expecting(&mut self, token_type: TokenType, msg: &str) -> Result<Token> {
    match (self.consume_current(), self.current_token_loc) {
      (Some(token), _) if token.is(token_type) => Ok(token),
      (Some(token), _) => Err(ParseErr::ExpectedTokenNotFound(
        ParseErrLoc::from(token),
        msg.to_string(),
      )),
      (None, Some(loc)) => Err(ParseErr::ExpectedTokenNotFound(
        ParseErrLoc::new(loc.0, loc.1),
        msg.to_string(),
      )),
      (None, None) => todo!(),
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

  pub fn current_is(&self, token_type: TokenType) -> bool {
    self.starts(token_type)
  }

  pub fn starts(&self, token_type: TokenType) -> bool {
    if self.tokens.len() == 0 {
      return false;
    }
    self.tokens[0].token_type == token_type
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
    return self.current_token().unwrap().len() == len;
  }
}
