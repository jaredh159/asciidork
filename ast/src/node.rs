use std::fmt::{Debug, Formatter, Result};

use crate::internal::*;

// https://docs.asciidoctor.org/asciidoc/latest/key-concepts/#document
#[derive(Debug, PartialEq, Eq)]
pub struct Document<'bmp> {
  pub header: Option<DocHeader<'bmp>>,
  pub content: DocContent<'bmp>,
  pub toc: Option<TableOfContents<'bmp>>,
}

impl<'bmp> Document<'bmp> {
  pub fn new(bump: &'bmp Bump) -> Self {
    Self {
      header: None,
      content: DocContent::Blocks(bvec![in bump]),
      toc: None,
    }
  }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Section<'bmp> {
  pub meta: ChunkMeta<'bmp>,
  pub level: u8,
  pub id: Option<BumpString<'bmp>>,
  pub heading: InlineNodes<'bmp>,
  pub blocks: BumpVec<'bmp, Block<'bmp>>,
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

impl Json for Document<'_> {
  fn to_json_in(&self, buf: &mut JsonBuf) {
    buf.begin_obj("Document");
    buf.add_option_member("header", self.header.as_ref());
    buf.add_member("content", &self.content);
    buf.add_option_member("toc", self.toc.as_ref());
    buf.finish_obj();
  }

  fn size_hint(&self) -> usize {
    // json is verbose compared to the source len
    // multiplying by 16 is an explicit choice to accept possible
    // extra unused memory for fewer/zero realloc + copy
    self.content.last_loc().map_or(1024, |loc| loc.end * 16)
  }
}
