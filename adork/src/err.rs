use std::fmt;

use crate::token::{Token, TokenType};

#[derive(Debug)]
pub enum AsciiDorkError {
  Parse(ParseErr),
}

#[derive(Debug, PartialEq, Eq)]
pub enum ParseErr {
  ExpectedTokenNotFound(SourceLocation, &'static str),
  Error(String, Option<Token>),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct SourceLocation {
  pub start: usize,
  pub end: usize,
  pub token_type: Option<TokenType>,
  pub is_exact: bool, // TODO: maybe not so much?
}

impl From<&Token> for SourceLocation {
  fn from(token: &Token) -> Self {
    SourceLocation {
      start: token.start,
      end: token.end,
      token_type: Some(token.token_type),
      is_exact: true,
    }
  }
}

impl SourceLocation {
  pub fn new(start: usize, end: usize) -> Self {
    SourceLocation {
      start,
      end,
      token_type: None,
      is_exact: true,
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

impl fmt::Display for ParseErr {
  fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
    todo!()
    // match self {
    //   ParseErr::ExpectedTokenNotFound(_, msg) => write!(f, "Expected token not found: {:?}", msg),
    //   ParseErr::Error(msg, token) => write!(f, "{}: {:?}", msg, token),
    // }
  }
}

pub type Result<T> = std::result::Result<T, AsciiDorkError>;

impl std::error::Error for ParseErr {}
