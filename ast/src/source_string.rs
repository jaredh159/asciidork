use std::ops::Deref;

use crate::internal::*;

#[derive(PartialEq, Eq, Clone)]
pub struct SourceString<'bmp> {
  pub src: BumpString<'bmp>,
  pub loc: SourceLocation,
}

impl<'bmp> SourceString<'bmp> {
  pub const fn new(src: BumpString<'bmp>, loc: SourceLocation) -> Self {
    Self { src, loc }
  }

  pub fn split_once(self, separator: &str, bump: &'bmp Bump) -> (Self, Option<Self>) {
    match self.src.split_once(separator) {
      Some((left, right)) => (
        Self::new(
          BumpString::from_str_in(left, bump),
          SourceLocation::new(self.loc.start, self.loc.start + left.len()),
        ),
        Some(Self::new(
          BumpString::from_str_in(right, bump),
          SourceLocation::new(self.loc.start + left.len() + separator.len(), self.loc.end),
        )),
      ),
      None => (self, None),
    }
  }
}

impl Json for SourceString<'_> {
  fn to_json_in(&self, buf: &mut JsonBuf) {
    self.src.to_json_in(buf);
  }
}

impl<'bmp> Deref for SourceString<'bmp> {
  type Target = str;

  fn deref(&self) -> &Self::Target {
    &self.src
  }
}

impl<'bmp> std::cmp::PartialEq<str> for SourceString<'bmp> {
  fn eq(&self, other: &str) -> bool {
    self.src == other
  }
}

impl<'bmp> std::fmt::Debug for SourceString<'bmp> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "SourceString{{\"{}\",{:?}}}", self.src, self.loc)
  }
}
