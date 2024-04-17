use crate::internal::*;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ChunkMeta<'bmp> {
  pub attrs: Option<AttrList<'bmp>>,
  pub title: Option<InlineNodes<'bmp>>,
  pub start: usize, // rename
}

impl<'bmp> ChunkMeta<'bmp> {
  pub const fn empty(start: usize) -> Self {
    Self { title: None, attrs: None, start }
  }

  pub const fn new(
    attrs: Option<AttrList<'bmp>>,
    title: Option<InlineNodes<'bmp>>,
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

  pub const fn is_empty(&self) -> bool {
    self.attrs.is_none() && self.title.is_none()
  }
}

impl Json for ChunkMeta<'_> {
  fn to_json_in(&self, buf: &mut JsonBuf) {
    buf.begin_obj("ChunkMeta");
    buf.add_option_member("attrs", self.attrs.as_ref());
    buf.add_option_member("title", self.title.as_ref());
    buf.finish_obj();
  }
}
