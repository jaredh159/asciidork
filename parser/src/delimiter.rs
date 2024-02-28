use crate::internal::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Delimiter {
  BlockQuote,
  Example,
  Open,
  Sidebar,
  Literal,
  Listing,
}

impl From<Delimiter> for BlockContext {
  fn from(delimiter: Delimiter) -> Self {
    match delimiter {
      Delimiter::Sidebar => BlockContext::Sidebar,
      Delimiter::Open => BlockContext::Open,
      Delimiter::Example => BlockContext::Example,
      Delimiter::BlockQuote => BlockContext::BlockQuote,
      Delimiter::Listing => BlockContext::Listing,
      Delimiter::Literal => BlockContext::Literal,
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
      "____" => Some(Delimiter::BlockQuote),
      "----" => Some(Delimiter::Listing),
      "...." => Some(Delimiter::Literal),
      "--" => Some(Delimiter::Open),
      _ => None,
    }
  }
}
