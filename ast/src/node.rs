use crate::internal::*;

// https://docs.asciidoctor.org/asciidoc/latest/key-concepts/#document
#[derive(Debug, PartialEq, Eq)]
pub struct Document<'bmp> {
  pub header: Option<DocHeader<'bmp>>,
  pub content: DocContent<'bmp>,
}

impl<'bmp> Document<'bmp> {
  pub fn new(bump: &'bmp Bump) -> Self {
    Self {
      header: None,
      content: DocContent::Blocks(bvec![in bump]),
    }
  }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Section<'bmp> {
  pub meta: ChunkMeta<'bmp>,
  pub level: u8,
  pub heading: InlineNodes<'bmp>,
  pub blocks: BumpVec<'bmp, Block<'bmp>>,
}

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub struct Callout {
  /// list index, e.g. `0` maps to `1` in dr id: `CO1-3`
  pub list_idx: u8,
  /// callout index w/in list, e.g. `2` maps to `3` in dr id: `CO1-3`
  pub callout_idx: u8,
  /// the reader-facing callout number, i.e. `1` in `<1>`
  pub num: u8,
}
