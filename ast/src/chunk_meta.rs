use crate::internal::*;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ChunkMeta<'bmp> {
  pub attrs: Option<AttrList<'bmp>>,
  pub title: Option<SourceString<'bmp>>,
  pub start: usize, // rename
}

impl<'bmp> ChunkMeta<'bmp> {
  pub const fn empty(start: usize) -> Self {
    Self { title: None, attrs: None, start }
  }

  pub const fn new(
    attrs: Option<AttrList<'bmp>>,
    title: Option<SourceString<'bmp>>,
    start: usize,
  ) -> Self {
    Self { title, attrs, start }
  }

  pub fn has_attr_option(&self, name: &str) -> bool {
    self
      .attrs
      .as_ref()
      .map_or(false, |attrs| attrs.has_option(name))
  }
}
