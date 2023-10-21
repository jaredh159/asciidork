use bumpalo::collections::Vec as BumpVec;

use crate::{line::Line, token::TokenKind};

pub struct Block<'alloc, 'src> {
  // NB: lines kept in reverse, as there is no VeqDeque in bumpalo
  // and we almost always want to consume from the front, so fake it
  lines: BumpVec<'alloc, Line<'alloc, 'src>>,
}

impl<'alloc, 'src> Block<'alloc, 'src> {
  pub fn new(mut lines: BumpVec<'alloc, Line<'alloc, 'src>>) -> Self {
    lines.reverse();
    Block { lines }
  }

  pub fn current_line(&self) -> Option<&Line<'alloc, 'src>> {
    self.lines.last()
  }

  pub fn is_empty(&self) -> bool {
    self.lines.is_empty()
  }

  pub fn consume_current(&mut self) -> Option<Line<'alloc, 'src>> {
    self.lines.pop()
  }

  pub fn restore(&mut self, line: Line<'alloc, 'src>) {
    self.lines.push(line);
  }

  pub fn contains_seq(&self, kinds: &[TokenKind]) -> bool {
    self.lines.iter().any(|line| line.contains_seq(kinds))
  }

  pub fn terminates_constrained(&self, stop_tokens: &[TokenKind]) -> bool {
    self
      .lines
      .iter()
      .any(|line| line.terminates_constrained(stop_tokens))
  }
}
