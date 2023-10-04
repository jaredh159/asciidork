use bumpalo::collections::String;

use super::source_location::SourceLocation;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TokenKind {
  Ampersand,
  Backtick,
  Backslash,
  Bang,
  Caret,
  CloseBracket,
  Colon,
  Comma,
  CommentBlock,
  CommentLine,
  DoubleQuote,
  Dot,
  EqualSigns,
  Eof,
  GreaterThan,
  Hash,
  LessThan,
  MacroName,
  Newline,
  OpenBracket,
  Percent,
  Plus,
  SemiColon,
  SingleQuote,
  Star,
  Tilde,
  Underscore,
  Whitespace,
  Word,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TokenValue<'alloc> {
  None,
  String(String<'alloc>),
}

impl<'alloc> TokenValue<'alloc> {
  pub fn as_string(&self) -> &String<'alloc> {
    match self {
      Self::String(s) => s,
      Self::None => panic!("expected string"),
    }
  }
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Token<'alloc> {
  pub kind: TokenKind,
  pub loc: SourceLocation,
  pub value: TokenValue<'alloc>,
}

impl<'alloc> Default for TokenValue<'alloc> {
  fn default() -> Self {
    Self::None
  }
}

impl Default for TokenKind {
  fn default() -> Self {
    Self::Eof
  }
}
