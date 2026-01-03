use std::fmt;

use crate::internal::*;

mod context;
mod parse_csv_table;
mod parse_dsv_table;
mod parse_psv_table;
mod parse_table;
mod parse_table_spec;

#[derive(Clone, Copy)]
pub enum DataFormat {
  Prefix(char),
  Csv(char),
  Delimited(char),
}

impl DataFormat {
  const fn replace_separator(&mut self, sep: char) {
    match self {
      DataFormat::Prefix(c) => *c = sep,
      DataFormat::Csv(c) => *c = sep,
      DataFormat::Delimited(c) => *c = sep,
    };
  }

  pub const fn separator(&self) -> char {
    match self {
      DataFormat::Prefix(c) => *c,
      DataFormat::Csv(c) => *c,
      DataFormat::Delimited(c) => *c,
    }
  }

  pub const fn embeddable_separator(&self) -> Option<char> {
    match self.separator() {
      ':' | ';' | '|' | ',' => None,
      sep => Some(sep),
    }
  }

  pub const fn separator_token_kind(&self) -> Option<TokenKind> {
    match self.separator() {
      ':' => Some(TokenKind::Colon),
      ';' => Some(TokenKind::SemiColon),
      '|' => Some(TokenKind::Pipe),
      ',' => Some(TokenKind::Comma),
      _ => None,
    }
  }
}

impl fmt::Debug for DataFormat {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      DataFormat::Prefix(c) => write!(f, "Prefix('{}')", *c),
      DataFormat::Csv(c) => write!(f, "Csv('{}')", *c),
      DataFormat::Delimited(c) => write!(f, "Delimited('{}')", *c),
    }
  }
}

#[derive(Debug, Clone)]
pub struct TableTokens<'arena>(Line<'arena>);

impl<'arena> TableTokens<'arena> {
  pub fn new(tokens: Deq<'arena, Token<'arena>>) -> Self {
    Self(Line::new(tokens))
  }

  pub fn discard(&mut self, n: usize) {
    self.0.discard(n);
  }

  pub fn current(&self) -> Option<&Token<'arena>> {
    self.0.current_token()
  }

  pub fn current_mut(&mut self) -> Option<&mut Token<'arena>> {
    self.0.current_token_mut()
  }

  pub fn nth(&self, n: usize) -> Option<&Token<'arena>> {
    self.0.nth_token(n)
  }

  pub fn has_seq_at(&self, specs: &[TokenSpec], offset: u32) -> bool {
    self.0.has_seq_at(specs, offset)
  }

  pub fn consume_current(&mut self) -> Option<Token<'arena>> {
    self.0.consume_current()
  }

  pub fn drop_leading_bytes(&mut self, n: u32) {
    self.0.drop_leading_bytes(n);
  }

  pub fn consume_splitting(&mut self, embeddable_separator: Option<char>) -> Option<Token<'arena>> {
    let Some(sep) = embeddable_separator else {
      return self.consume_current();
    };
    if !self.current().kind(TokenKind::Word) {
      return self.consume_current();
    }

    let token = self.current().unwrap();
    if token.lexeme.contains(sep) {
      let (before, _) = token.lexeme.split_once(sep).unwrap();
      // NB: caller must check that lexeme doesn't START with sep
      debug_assert!(!before.is_empty());
      let lexeme = BumpString::from_str_in(before, self.0.bump_arena());
      let loc = token.loc;
      let before_len = before.len() as u32;
      self.drop_leading_bytes(before_len);
      Some(Token {
        kind: TokenKind::Word,
        lexeme,
        loc: SourceLocation::new(loc.start, loc.start + before_len, loc.include_depth),
        attr_replacement: false,
      })
    } else {
      self.consume_current()
    }
  }

  pub fn is_empty(&self) -> bool {
    self.0.is_empty()
  }
}
