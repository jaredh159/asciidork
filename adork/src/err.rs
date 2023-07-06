use std::fmt;

use crate::token::Token;

#[derive(Debug)]
pub enum AsciiDorkError {
  Parse(ParseErr),
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
  UnexpectedToken(Option<Token>),
  Error(String, Option<Token>),
}

impl fmt::Display for ParseErr {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      ParseErr::UnexpectedToken(token) => write!(f, "Unexpected token: {:?}", token),
      ParseErr::Error(msg, token) => write!(f, "{}: {:?}", msg, token),
    }
  }
}

pub type Result<T> = std::result::Result<T, AsciiDorkError>;

impl std::error::Error for ParseErr {}
