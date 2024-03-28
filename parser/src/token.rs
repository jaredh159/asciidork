use crate::internal::*;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub enum TokenKind {
  Ampersand,
  Backtick,
  Backslash,
  Bang,
  Caret,
  CloseBrace,
  CloseBracket,
  Colon,
  Comma,
  CommentBlock,
  CommentLine,
  Dashes,
  DelimiterLine,
  Digits,
  Discard,
  DoubleQuote,
  Dots,
  EqualSigns,
  #[default]
  Eof,
  GreaterThan,
  Hash,
  LessThan,
  MacroName,
  MaybeEmail,
  Newline,
  OpenBrace,
  OpenBracket,
  Percent,
  Plus,
  SemiColon,
  SingleQuote,
  Star,
  TermDelimiter,
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

impl<'src> Token<'src> {
  pub fn to_source_string<'bmp>(&self, bump: &'bmp Bump) -> SourceString<'bmp> {
    let bump_str = BumpString::from_str_in(self.lexeme, bump);
    SourceString::new(bump_str, self.loc)
  }

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

  pub const fn len(&self) -> usize {
    self.lexeme.len()
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
