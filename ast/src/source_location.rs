use std::fmt::{Debug, Formatter, Result};
use std::ops::Range;

use crate::internal::*;

#[derive(PartialEq, Eq, Clone, Copy, Hash, Default)]
pub struct SourceLocation {
  pub start: u32,
  pub end: u32,
  pub include_depth: u16,
}

impl SourceLocation {
  pub fn new(start: u32, end: u32) -> Self {
    debug_assert!(start <= end);
    Self { start, end, include_depth: 0 }
  }

  pub fn new_depth(start: u32, end: u32, include_depth: u16) -> Self {
    debug_assert!(start <= end);
    Self { start, end, include_depth }
  }

  pub fn extend(&mut self, other: SourceLocation) {
    self.start = self.start.min(other.start);
    self.end = self.end.max(other.end);
  }

  pub fn clamp_start(&self) -> SourceLocation {
    Self::new_depth(self.start, self.start, self.include_depth)
  }

  pub fn clamp_end(&self) -> SourceLocation {
    Self::new_depth(self.end, self.end, self.include_depth)
  }

  pub fn decr_end(&self) -> SourceLocation {
    Self::new_depth(self.start, self.end - 1, self.include_depth)
  }

  pub fn incr_end(&self) -> SourceLocation {
    Self::new_depth(self.start, self.end + 1, self.include_depth)
  }

  pub fn decr_start(&self) -> SourceLocation {
    Self::new_depth(self.start - 1, self.end, self.include_depth)
  }

  pub fn incr_start(&self) -> SourceLocation {
    Self::new_depth(self.start + 1, self.end, self.include_depth)
  }

  pub fn incr(&self) -> SourceLocation {
    Self::new_depth(self.start + 1, self.end + 1, self.include_depth)
  }

  pub fn decr(&self) -> SourceLocation {
    Self::new_depth(self.start - 1, self.end - 1, self.include_depth)
  }

  pub const fn size(&self) -> u32 {
    self.end - self.start
  }
}

impl From<u32> for SourceLocation {
  fn from(offset: u32) -> Self {
    Self::new(offset, offset)
  }
}

impl From<Range<u32>> for SourceLocation {
  fn from(range: Range<u32>) -> Self {
    Self::new(range.start, range.end)
  }
}

impl From<Range<i32>> for SourceLocation {
  fn from(range: Range<i32>) -> Self {
    Self::new(range.start as u32, range.end as u32)
  }
}

impl From<Range<usize>> for SourceLocation {
  fn from(range: Range<usize>) -> Self {
    Self::new(range.start as u32, range.end as u32)
  }
}

impl Debug for SourceLocation {
  fn fmt(&self, f: &mut Formatter<'_>) -> Result {
    write!(
      f,
      "{}..{}{}",
      self.start,
      self.end,
      if self.include_depth > 0 {
        format!("/{}", self.include_depth)
      } else {
        String::new()
      }
    )
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
