use std::fmt::{Debug, Formatter, Result};
use std::ops::Range;

use crate::internal::*;

#[derive(PartialEq, Eq, Clone, Copy, Hash, Default)]
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

  pub fn clamp_start(&self) -> SourceLocation {
    Self::new(self.start, self.start)
  }

  pub fn clamp_end(&self) -> SourceLocation {
    Self::new(self.end, self.end)
  }

  pub fn decr_end(&self) -> SourceLocation {
    Self::new(self.start, self.end - 1)
  }

  pub fn incr_end(&self) -> SourceLocation {
    Self::new(self.start, self.end + 1)
  }

  pub fn decr_start(&self) -> SourceLocation {
    Self::new(self.start - 1, self.end)
  }

  pub fn incr_start(&self) -> SourceLocation {
    Self::new(self.start + 1, self.end)
  }

  pub fn incr(&self) -> SourceLocation {
    Self::new(self.start + 1, self.end + 1)
  }

  pub fn decr(&self) -> SourceLocation {
    Self::new(self.start - 1, self.end - 1)
  }
}

impl From<usize> for SourceLocation {
  fn from(offset: usize) -> Self {
    Self::new(offset, offset)
  }
}

impl From<Range<usize>> for SourceLocation {
  fn from(range: Range<usize>) -> Self {
    Self::new(range.start, range.end)
  }
}

impl Debug for SourceLocation {
  fn fmt(&self, f: &mut Formatter<'_>) -> Result {
    write!(f, "{}..{}", self.start, self.end)
  }
}

impl Json for SourceLocation {
  fn to_json_in(&self, buf: &mut JsonBuf) {
    buf.push('[');
    buf.push_str(&self.start.to_string());
    buf.push(',');
    buf.push_str(&self.end.to_string());
    buf.push(']');
  }
}
