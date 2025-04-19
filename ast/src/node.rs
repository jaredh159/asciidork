use std::fmt::{Debug, Formatter, Result};

use crate::internal::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Anchor<'arena> {
  pub reftext: Option<InlineNodes<'arena>>,
  pub title: InlineNodes<'arena>,
  pub source_loc: Option<SourceLocation>,
  /// can be used to identify the source file in which the anchor was found
  pub source_idx: u16,
  pub is_biblio: bool,
}

#[derive(Default, Clone, Copy, Eq, PartialEq)]
pub struct Callout {
  /// list index, e.g. `0` maps to `1` in dr id: `CO1-3`
  pub list_idx: u8,
  /// callout index w/in list, e.g. `2` maps to `3` in dr id: `CO1-3`
  pub callout_idx: u8,
  /// the reader-facing callout number, i.e. `1` in `<1>`
  pub number: u8,
}

impl Callout {
  pub const fn new(list_idx: u8, callout_idx: u8, number: u8) -> Self {
    Self { list_idx, callout_idx, number }
  }
}

impl Debug for Callout {
  fn fmt(&self, f: &mut Formatter<'_>) -> Result {
    write!(
      f,
      "Callout(list_idx: {}, callout_idx: {}, number: {})",
      self.list_idx, self.callout_idx, self.number
    )
  }
}
