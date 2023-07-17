use std::fmt;

use crate::token::{Token, TokenType};

#[derive(Debug)]
pub enum AsciiDorkError {
  Parse(ParseErr),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct ParseErrLoc {
  pub start: usize,
  pub end: usize,
  pub token_type: Option<TokenType>,
}

impl From<Token> for ParseErrLoc {
  fn from(token: Token) -> Self {
    ParseErrLoc {
      start: token.start,
      end: token.end,
      token_type: Some(token.token_type),
    }
  }
}

impl ParseErrLoc {
  pub fn new(start: usize, end: usize) -> Self {
    ParseErrLoc {
      start,
      end,
      token_type: None,
    }
  }
}

impl fmt::Display for AsciiDorkError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      AsciiDorkError::Parse(err) => write!(f, "{}", err),
    }
  }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ParseErr {
  ExpectedTokenNotFound(ParseErrLoc, String),
  Error(String, Option<Token>),
}

impl fmt::Display for ParseErr {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      ParseErr::ExpectedTokenNotFound(_, msg) => write!(f, "Expected token not found: {:?}", msg),
      ParseErr::Error(msg, token) => write!(f, "{}: {:?}", msg, token),
    }
  }
}

pub type Result<T> = std::result::Result<T, AsciiDorkError>;

impl std::error::Error for ParseErr {}
