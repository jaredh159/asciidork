use crate::internal::*;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ChunkMeta<'arena> {
  pub attrs: Option<AttrList<'arena>>,
  pub title: Option<InlineNodes<'arena>>,
  pub start: u32, // rename
}

impl<'arena> ChunkMeta<'arena> {
  pub const fn empty(start: u32) -> Self {
    Self { title: None, attrs: None, start }
  }

  pub const fn new(
    attrs: Option<AttrList<'arena>>,
    title: Option<InlineNodes<'arena>>,
    start: u32,
  ) -> Self {
    Self { title, attrs, start }
  }

  pub fn has_attr_option(&self, name: &str) -> bool {
    self
      .attrs
      .as_ref()
      .map_or(false, |attrs| attrs.has_option(name))
  }

  pub fn has_str_positional(&self, positional: &str) -> bool {
    self
      .attrs
      .as_ref()
      .map_or(false, |attrs| attrs.has_str_positional(positional))
  }

  pub fn attr_named(&self, name: &str) -> Option<&str> {
    self.attrs.as_ref().and_then(|attrs| attrs.named(name))
  }

  pub const fn is_empty(&self) -> bool {
    self.attrs.is_none() && self.title.is_none()
  }
}
