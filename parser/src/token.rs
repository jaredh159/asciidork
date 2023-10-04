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

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Token<'alloc> {
  pub kind: TokenKind,
  pub loc: SourceLocation,
  pub lexeme: &'alloc str,
}

impl Default for TokenKind {
  fn default() -> Self {
    Self::Eof
  }
}
