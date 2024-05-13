use crate::internal::*;

mod context;
mod parse_table;
mod parse_table_spec;

#[derive(Debug, Clone, Copy)]
enum DataFormat {
  Prefix(u8),
  Csv(u8),
  Delimited(u8),
}

#[derive(Debug, Clone)]
pub struct TableTokens<'bmp, 'src>(Line<'bmp, 'src>);

impl<'bmp, 'src> TableTokens<'bmp, 'src> {
  pub fn new(tokens: BumpVec<'bmp, Token<'src>>, src: &'src str) -> Self {
    Self(Line::new(tokens, src))
  }

  pub fn discard(&mut self, n: usize) {
    self.0.discard(n);
  }

  pub fn current(&self) -> Option<&Token<'src>> {
    self.0.current_token()
  }

  pub fn current_mut(&mut self) -> Option<&mut Token<'src>> {
    self.0.current_token_mut()
  }

  pub fn nth(&self, n: usize) -> Option<&Token<'src>> {
    self.0.nth_token(n)
  }

  pub fn has_seq_at(&self, kinds: &[TokenKind], offset: usize) -> bool {
    self.0.has_seq_at(kinds, offset)
  }

  pub fn consume_current(&mut self) -> Option<Token<'src>> {
    self.0.consume_current()
  }

  pub fn is_empty(&self) -> bool {
    self.0.is_empty()
  }
}

impl DataFormat {
  pub const fn sep(&self) -> u8 {
    match self {
      DataFormat::Prefix(c) => *c,
      DataFormat::Csv(c) => *c,
      DataFormat::Delimited(c) => *c,
    }
  }
}
