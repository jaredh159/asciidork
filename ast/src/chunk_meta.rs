use crate::internal::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkMeta<'arena> {
  pub attrs: MultiAttrList<'arena>,
  pub title: Option<InlineNodes<'arena>>,
  pub start: u32, // rename
}

impl<'arena> ChunkMeta<'arena> {
  pub fn empty(start: u32, bump: &'arena Bump) -> Self {
    Self {
      title: None,
      attrs: MultiAttrList::new_in(bump),
      start,
    }
  }

  pub fn new(
    attrs: impl Into<MultiAttrList<'arena>>,
    title: Option<InlineNodes<'arena>>,
    start: u32,
  ) -> Self {
    Self { title, attrs: attrs.into(), start }
  }

  pub fn is_empty(&self) -> bool {
    self.attrs.is_empty() && self.title.is_none()
  }
}
