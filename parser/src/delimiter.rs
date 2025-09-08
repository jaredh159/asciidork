use crate::internal::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DelimiterKind {
  BlockQuote,
  Example,
  Open,
  Sidebar,
  Literal,
  Listing,
  Passthrough,
  Comment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Delimiter {
  pub kind: DelimiterKind,
  pub len: u8,
}

impl From<DelimiterKind> for BlockContext {
  fn from(delimiter: DelimiterKind) -> Self {
    match delimiter {
      DelimiterKind::Sidebar => BlockContext::Sidebar,
      DelimiterKind::Open => BlockContext::Open,
      DelimiterKind::Example => BlockContext::Example,
      DelimiterKind::BlockQuote => BlockContext::BlockQuote,
      DelimiterKind::Listing => BlockContext::Listing,
      DelimiterKind::Literal => BlockContext::Literal,
      DelimiterKind::Passthrough => BlockContext::Passthrough,
      DelimiterKind::Comment => BlockContext::Comment,
    }
  }
}

impl Token<'_> {
  pub fn to_delimiter(&self) -> Option<Delimiter> {
    let kind = self.to_delimiter_kind()?;
    Some(Delimiter { kind, len: self.lexeme.len() as u8 })
  }

  pub fn to_delimiter_kind(&self) -> Option<DelimiterKind> {
    if self.kind != TokenKind::DelimiterLine {
      return None;
    }
    match self.lexeme.as_str() {
      "****" => Some(DelimiterKind::Sidebar),
      "____" => Some(DelimiterKind::BlockQuote),
      "----" | "```" => Some(DelimiterKind::Listing),
      "...." => Some(DelimiterKind::Literal),
      "++++" => Some(DelimiterKind::Passthrough),
      "////" => Some(DelimiterKind::Comment),
      "--" => Some(DelimiterKind::Open),
      _ => Some(DelimiterKind::Example), // length can vary for nesting
    }
  }
}
