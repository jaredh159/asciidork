use std::fmt::{Debug, Formatter, Result};

use crate::internal::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Anchor<'arena> {
  pub reftext: Option<InlineNodes<'arena>>,
  pub title: InlineNodes<'arena>,
  /// can be used to identify the source file in which the anchor was found
  pub source_idx: u16,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Section<'arena> {
  pub meta: ChunkMeta<'arena>,
  pub level: u8,
  pub id: Option<BumpString<'arena>>,
  pub heading: InlineNodes<'arena>,
  pub blocks: BumpVec<'arena, Block<'arena>>,
}

#[derive(Default, Clone, Copy, Eq, PartialEq)]
pub struct Callout {
  /// list index, e.g. `0` maps to `1` in dr id: `CO1-3`
  pub list_idx: u8,
  /// callout index w/in list, e.g. `2` maps to `3` in dr id: `CO1-3`
  pub callout_idx: u8,
  /// the reader-facing callout number, i.e. `1` in `<1>`
  pub number: u8,
}

impl Callout {
  pub const fn new(list_idx: u8, callout_idx: u8, number: u8) -> Self {
    Self { list_idx, callout_idx, number }
  }
}

impl Debug for Callout {
  fn fmt(&self, f: &mut Formatter<'_>) -> Result {
    write!(
      f,
      "Callout(list_idx: {}, callout_idx: {}, number: {})",
      self.list_idx, self.callout_idx, self.number
    )
  }
}

// json

impl Json for Callout {
  fn to_json_in(&self, buf: &mut JsonBuf) {
    buf.begin_obj("Callout");
    buf.add_member("list_idx", &self.list_idx);
    buf.add_member("callout_idx", &self.callout_idx);
    buf.add_member("number", &self.number);
    buf.finish_obj();
  }
}

impl Json for Section<'_> {
  fn to_json_in(&self, buf: &mut JsonBuf) {
    buf.begin_obj("Section");
    if !self.meta.is_empty() {
      buf.add_member("meta", &self.meta);
    }
    buf.add_member("level", &self.level);
    buf.add_option_member("id", self.id.as_ref());
    buf.add_member("heading", &self.heading);
    buf.add_member("blocks", &self.blocks);
    buf.finish_obj();
  }
}

impl Json for Anchor<'_> {
  fn to_json_in(&self, buf: &mut JsonBuf) {
    buf.begin_obj("Anchor");
    buf.add_option_member("reftext", self.reftext.as_ref());
    buf.add_member("title", &self.title);
    buf.finish_obj();
  }
}

impl Json for Document<'_> {
  fn to_json_in(&self, buf: &mut JsonBuf) {
    buf.begin_obj("Document");
    buf.add_member("subtitle", &self.subtitle);
    buf.add_member("content", &self.content);
    buf.add_option_member("toc", self.toc.as_ref());
    buf.finish_obj();
  }

  fn size_hint(&self) -> usize {
    // json is verbose compared to the source len
    // multiplying by 16 is an explicit choice to accept possible
    // extra unused memory for fewer/zero realloc + copy
    self
      .content
      .last_loc()
      .map_or(1024, |loc| loc.end as usize * 16)
  }
}
