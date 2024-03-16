use crate::internal::*;

// https://docs.asciidoctor.org/asciidoc/latest/key-concepts/#document
#[derive(Debug, PartialEq, Eq)]
pub struct Document<'bmp> {
  pub header: Option<DocHeader<'bmp>>,
  pub content: DocContent<'bmp>,
}

impl<'bmp> Document<'bmp> {
  pub fn new(bump: &'bmp bumpalo::Bump) -> Self {
    Self {
      header: None,
      content: DocContent::Blocks(bumpalo::vec![in bump]),
    }
  }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Section<'bmp> {
  pub level: u8,
  pub heading: InlineNodes<'bmp>,
  pub blocks: BumpVec<'bmp, Block<'bmp>>,
}

impl<'bmp> Section<'bmp> {
  pub fn new_in(bump: &'bmp bumpalo::Bump) -> Self {
    Self {
      level: 1,
      heading: InlineNodes::new(bump),
      blocks: BumpVec::new_in(bump),
    }
  }
}
