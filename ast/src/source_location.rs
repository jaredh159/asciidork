use std::fmt::{Debug, Formatter, Result};

#[derive(PartialEq, Eq, Clone, Copy, Hash, Default)]
pub struct SourceLocation {
  pub start: u32,
  pub end: u32,
  pub include_depth: u16,
}

impl SourceLocation {
  pub fn new(start: u32, end: u32, include_depth: u16) -> Self {
    debug_assert!(start <= end);
    Self { start, end, include_depth }
  }

  pub fn spanning(start: Self, end: Self) -> Self {
    debug_assert!(start.start <= end.start);
    debug_assert!(start.end <= end.end);
    debug_assert!(start.include_depth == end.include_depth);
    Self {
      start: start.start,
      end: end.end,
      include_depth: start.include_depth,
    }
  }

  pub fn extend(&mut self, other: SourceLocation) {
    self.start = self.start.min(other.start);
    self.end = self.end.max(other.end);
  }

  pub fn adding_to_end(&self, adding: u32) -> SourceLocation {
    Self::new(self.start, self.end + adding, self.include_depth)
  }

  pub fn setting_end(&self, end: u32) -> SourceLocation {
    Self::new(self.start, end, self.include_depth)
  }

  #[must_use]
  pub fn clamp_start(&self) -> SourceLocation {
    Self::new(self.start, self.start, self.include_depth)
  }

  #[must_use]
  pub fn clamp_end(&self) -> SourceLocation {
    Self::new(self.end, self.end, self.include_depth)
  }

  #[must_use]
  pub fn decr_end(&self) -> SourceLocation {
    Self::new(self.start, self.end - 1, self.include_depth)
  }

  #[must_use]
  pub fn incr_end(&self) -> SourceLocation {
    Self::new(self.start, self.end + 1, self.include_depth)
  }

  #[must_use]
  pub fn decr_start(&self) -> SourceLocation {
    Self::new(self.start - 1, self.end, self.include_depth)
  }

  #[must_use]
  pub fn incr_start(&self) -> SourceLocation {
    Self::new(self.start + 1, self.end, self.include_depth)
  }

  #[must_use]
  pub fn incr(&self) -> SourceLocation {
    Self::new(self.start + 1, self.end + 1, self.include_depth)
  }

  #[must_use]
  pub fn decr(&self) -> SourceLocation {
    Self::new(self.start - 1, self.end - 1, self.include_depth)
  }

  #[must_use]
  pub fn offset(&self, offset: u32) -> SourceLocation {
    Self::new(self.start + offset, self.end + offset, self.include_depth)
  }

  #[must_use]
  pub const fn size(&self) -> u32 {
    self.end - self.start
  }

  #[must_use]
  pub const fn is_empty(&self) -> bool {
    self.start == self.end
  }

  #[must_use]
  pub const fn uid(&self) -> u64 {
    ((self.include_depth as u64) << 32) | (self.start as u64)
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
