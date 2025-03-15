use std::fmt::{Debug, Formatter, Result};
use std::ops::Range;

use crate::SourceLocation;

#[derive(PartialEq, Eq, Clone, Default)]
pub struct MultiSourceLocation {
  pub start_pos: u32,
  pub start_depth: u16,
  pub end_pos: u32,
  pub end_depth: u16,
}

impl Debug for MultiSourceLocation {
  fn fmt(&self, f: &mut Formatter<'_>) -> Result {
    write!(
      f,
      "MultiSourceLocation({}/{}..{}/{})",
      self.start_pos, self.start_depth, self.end_pos, self.end_depth
    )
  }
}

impl MultiSourceLocation {
  pub const fn spanning(start: SourceLocation, end: SourceLocation) -> Self {
    Self {
      start_pos: start.start,
      start_depth: start.include_depth,
      end_pos: end.end,
      end_depth: end.include_depth,
    }
  }

  pub const fn setting_start(&self, start: SourceLocation) -> Self {
    Self {
      start_pos: start.start,
      start_depth: start.include_depth,
      end_pos: self.end_pos,
      end_depth: self.end_depth,
    }
  }

  pub const fn setting_end(&self, end: SourceLocation) -> Self {
    Self {
      start_pos: self.start_pos,
      start_depth: self.start_depth,
      end_pos: end.end,
      end_depth: end.include_depth,
    }
  }

  pub fn extend_end(&mut self, other: &MultiSourceLocation) {
    self.end_pos = other.end_pos;
    self.end_depth = other.end_depth;
  }
}

impl From<SourceLocation> for MultiSourceLocation {
  fn from(loc: SourceLocation) -> Self {
    Self {
      start_pos: loc.start,
      start_depth: loc.include_depth,
      end_pos: loc.end,
      end_depth: loc.include_depth,
    }
  }
}

impl From<Range<u32>> for MultiSourceLocation {
  fn from(range: Range<u32>) -> Self {
    Self {
      start_pos: range.start,
      start_depth: 0,
      end_pos: range.end,
      end_depth: 0,
    }
  }
}

impl From<u32> for MultiSourceLocation {
  fn from(pos: u32) -> Self {
    Self {
      start_pos: pos,
      start_depth: 0,
      end_pos: pos,
      end_depth: 0,
    }
  }
}
