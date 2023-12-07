use std::ops::Deref;

use crate::ast::*;
use crate::utils::bump::*;

#[derive(PartialEq, Eq, Clone)]
pub struct SourceString<'bmp> {
  pub src: String<'bmp>,
  pub loc: SourceLocation,
}

impl<'bmp> SourceString<'bmp> {
  pub fn new(src: String<'bmp>, loc: SourceLocation) -> Self {
    Self { src, loc }
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
    write!(f, "SourceString {{ \"{}\", {:?} }}", self.src, self.loc)
  }
}
