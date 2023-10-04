// todo: move to AST

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, Default)]
pub struct SourceLocation {
  pub start: usize,
  pub end: usize,
}

impl SourceLocation {
  pub fn new(start: usize, end: usize) -> Self {
    debug_assert!(start <= end);
    Self { start, end }
  }

  pub fn extend(&mut self, other: SourceLocation) {
    self.start = self.start.min(other.start);
    self.end = self.end.max(other.end);
  }
}

impl From<usize> for SourceLocation {
  fn from(offset: usize) -> Self {
    Self::new(offset, offset)
  }
}
