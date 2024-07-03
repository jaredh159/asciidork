use std::ops::Deref;

use crate::internal::*;

#[derive(PartialEq, Eq, Clone, Hash)]
pub struct SourceString<'arena> {
  pub src: BumpString<'arena>,
  pub loc: SourceLocation,
}

impl<'arena> SourceString<'arena> {
  pub const fn new(src: BumpString<'arena>, loc: SourceLocation) -> Self {
    Self { src, loc }
  }

  pub fn split_once(self, separator: &str, bump: &'arena Bump) -> (Self, Option<Self>) {
    match self.src.split_once(separator) {
      Some((left, right)) => (
        Self::new(
          BumpString::from_str_in(left, bump),
          SourceLocation::new(self.loc.start, self.loc.start + left.len() as u32),
        ),
        Some(Self::new(
          BumpString::from_str_in(right, bump),
          SourceLocation::new(
            self.loc.start + left.len() as u32 + separator.len() as u32,
            self.loc.end,
          ),
        )),
      ),
      None => (self, None),
    }
  }

  pub fn drop_first(&mut self) {
    self.src.drain(..1);
    self.loc.start += 1;
  }
}

impl Json for SourceString<'_> {
  fn to_json_in(&self, buf: &mut JsonBuf) {
    self.src.to_json_in(buf);
  }
}

impl<'arena> Deref for SourceString<'arena> {
  type Target = str;

  fn deref(&self) -> &Self::Target {
    &self.src
  }
}

impl<'arena> std::cmp::PartialEq<str> for SourceString<'arena> {
  fn eq(&self, other: &str) -> bool {
    self.src == other
  }
}

impl<'arena> std::fmt::Debug for SourceString<'arena> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "SourceString{{\"{}\",{:?}}}", self.src, self.loc)
  }
}
