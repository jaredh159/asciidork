use std::ops::Deref;

use crate::ast::*;
use crate::utils::bump::*;

#[derive(Debug, PartialEq, Eq, Clone)]
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
