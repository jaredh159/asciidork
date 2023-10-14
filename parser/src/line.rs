use bumpalo::collections::Vec as BumpVec;

use crate::token::Token;

#[derive(Debug)]
pub struct Line<'alloc, 'src> {
  pub src: &'src str,
  tokens: BumpVec<'alloc, Token<'src>>,
  pos: usize,
}

impl<'alloc, 'src> Line<'alloc, 'src> {
  pub fn new(tokens: BumpVec<'alloc, Token<'src>>, src: &'src str) -> Self {
    Line { tokens, src, pos: 0 }
  }

  pub fn current_token(&self) -> Option<&Token<'src>> {
    self.tokens.get(self.pos)
  }

  pub fn is_empty(&self) -> bool {
    self.pos >= self.tokens.len()
  }

  pub fn consume_current(&mut self) -> Option<Token<'src>> {
    if self.is_empty() {
      return None;
    }
    let token = std::mem::take(&mut self.tokens[self.pos]);
    self.pos += 1;
    Some(token)
  }
}
