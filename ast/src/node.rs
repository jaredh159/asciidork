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
