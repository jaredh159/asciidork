use crate::internal::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkMeta<'arena> {
  pub attrs: MultiAttrList<'arena>,
  pub dot_line_title: Option<InlineNodes<'arena>>,
  pub start_loc: SourceLocation,
}

impl<'arena> ChunkMeta<'arena> {
  pub fn empty(start_loc: SourceLocation, bump: &'arena Bump) -> Self {
    Self {
      dot_line_title: None,
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
      dot_line_title: title,
      attrs: attrs.into(),
      start_loc: start_loc.into(),
    };
    if cm.is_empty() {
      cm.start_loc = cm.start_loc.clamp_start();
    }
    cm
  }

  pub fn is_empty(&self) -> bool {
    self.attrs.is_empty() && self.dot_line_title.is_none()
  }

  pub fn title(&self) -> Option<&InlineNodes<'_>> {
    self
      .attrs
      .named_nodes("title")
      .or(self.dot_line_title.as_ref())
  }
}
