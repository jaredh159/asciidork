use crate::internal::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Delimiter {
  Sidebar,
  Example,
  Open,
}

impl From<Delimiter> for BlockContext {
  fn from(delimiter: Delimiter) -> Self {
    match delimiter {
      Delimiter::Sidebar => BlockContext::Sidebar,
      Delimiter::Open => BlockContext::Open,
      Delimiter::Example => BlockContext::Example,
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
      "====" => Some(Delimiter::Example),
      "--" => Some(Delimiter::Open),
      _ => None,
    }
  }
}
