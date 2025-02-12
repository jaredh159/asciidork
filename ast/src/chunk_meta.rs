use crate::internal::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkMeta<'arena> {
  pub attrs: MultiAttrList<'arena>,
  pub title: Option<InlineNodes<'arena>>,
  pub start_loc: SourceLocation,
}

impl<'arena> ChunkMeta<'arena> {
  pub fn empty(start_loc: SourceLocation, bump: &'arena Bump) -> Self {
    Self {
      title: None,
      attrs: MultiAttrList::new_in(bump),
      start_loc,
    }
  }

  pub fn new(
    attrs: impl Into<MultiAttrList<'arena>>,
    title: Option<InlineNodes<'arena>>,
    start_loc: impl Into<SourceLocation>,
  ) -> Self {
    let mut cm = Self {
      title,
      attrs: attrs.into(),
      start_loc: start_loc.into(),
    };
    if cm.is_empty() {
      cm.start_loc = cm.start_loc.clamp_start();
    }
    cm
  }

  pub fn is_empty(&self) -> bool {
    self.attrs.is_empty() && self.title.is_none()
  }
}
