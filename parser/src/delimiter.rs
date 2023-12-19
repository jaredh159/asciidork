use crate::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Delimiter {
  Sidebar,
  Open,
}

impl From<Delimiter> for BlockContext {
  fn from(delimiter: Delimiter) -> Self {
    match delimiter {
      Delimiter::Sidebar => BlockContext::Sidebar,
      Delimiter::Open => BlockContext::Open,
    }
  }
}

impl<'src> Token<'src> {
  pub fn to_delimeter(&self) -> Option<Delimiter> {
    if self.kind != TokenKind::DelimiterLine {
      return None;
    }
    match self.lexeme {
      "****" => Some(Delimiter::Sidebar),
      "--" => Some(Delimiter::Open),
      _ => None,
    }
  }
}
