use crate::internal::*;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ChunkMeta<'bmp> {
  pub attrs: Option<AttrList<'bmp>>,
  pub title: Option<SourceString<'bmp>>,
  pub start: usize, // rename
}

impl<'bmp> ChunkMeta<'bmp> {
  pub fn empty(start: usize) -> Self {
    Self { title: None, attrs: None, start }
  }

  pub fn new(
    attrs: Option<AttrList<'bmp>>,
    title: Option<SourceString<'bmp>>,
    start: usize,
  ) -> Self {
    Self { title, attrs, start }
  }
}
