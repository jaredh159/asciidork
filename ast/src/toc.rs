use crate::internal::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TableOfContents<'arena> {
  pub title: BumpString<'arena>,
  pub nodes: BumpVec<'arena, TocNode<'arena>>,
  pub position: TocPosition,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TocNode<'arena> {
  pub level: u8,
  pub title: InlineNodes<'arena>,
  pub id: Option<BumpString<'arena>>,
  pub children: BumpVec<'arena, TocNode<'arena>>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TocPosition {
  Left,
  Right,
  Preamble,
  Macro,
  Auto,
}

impl Json for TableOfContents<'_> {
  fn to_json_in(&self, buf: &mut JsonBuf) {
    buf.begin_obj("TableOfContents");
    buf.add_member("title", &self.title);
    buf.add_member("nodes", &self.nodes);
    buf.add_member("position", &self.position);
    buf.finish_obj();
  }
}

impl Json for TocNode<'_> {
  fn to_json_in(&self, buf: &mut JsonBuf) {
    buf.begin_obj("TocNode");
    buf.add_member("level", &self.level);
    buf.add_member("title", &self.title);
    buf.add_option_member("id", self.id.as_ref());
    buf.add_member("children", &self.children);
    buf.finish_obj();
  }
}

impl Json for TocPosition {
  fn to_json_in(&self, buf: &mut JsonBuf) {
    buf.push_obj_enum_type("TocPosition", self);
  }
}
