use crate::ast::*;

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
  DelimiterLine,
  DoubleQuote,
  Dot,
  EqualSigns,
  Eof,
  GreaterThan,
  Hash,
  LessThan,
  MacroName,
  MaybeEmail,
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

#[derive(Clone, PartialEq, Eq, Default)]
pub struct Token<'src> {
  pub kind: TokenKind,
  pub loc: SourceLocation,
  pub lexeme: &'src str,
}

impl Default for TokenKind {
  fn default() -> Self {
    Self::Eof
  }
}

impl<'src> Token<'src> {
  pub fn to_url_scheme(&self) -> Option<UrlScheme> {
    match self.kind {
      TokenKind::MacroName => match self.lexeme {
        "https:" => Some(UrlScheme::Https),
        "http:" => Some(UrlScheme::Http),
        "ftp:" => Some(UrlScheme::Ftp),
        "irc:" => Some(UrlScheme::Irc),
        "mailto:" => Some(UrlScheme::Mailto),
        _ => None,
      },
      _ => None,
    }
  }
}

pub trait TokenIs {
  fn is_url_scheme(&self) -> bool;
  fn is(&self, kind: TokenKind) -> bool;
  fn is_not(&self, kind: TokenKind) -> bool {
    !self.is(kind)
  }
}

impl<'src> TokenIs for Token<'src> {
  fn is(&self, kind: TokenKind) -> bool {
    self.kind == kind
  }

  fn is_url_scheme(&self) -> bool {
    self.to_url_scheme().is_some()
  }
}

impl<'src> TokenIs for Option<&Token<'src>> {
  fn is(&self, kind: TokenKind) -> bool {
    self.map_or(false, |t| t.is(kind))
  }

  fn is_url_scheme(&self) -> bool {
    self.map_or(false, |t| t.is_url_scheme())
  }
}

impl<'src> std::fmt::Debug for Token<'src> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "Token {{ {:?}, \"{}\", {:?} }}",
      self.kind, self.lexeme, self.loc
    )
  }
}
