use crate::prelude::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Section<'bmp> {
  level: u8,
  heading: Heading<'bmp>,
  blocks: Vec<'bmp, Block<'bmp>>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Heading<'bmp> {
  inlines: Vec<'bmp, Inline<'bmp>>,
}

// https://docs.asciidoctor.org/asciidoc/latest/key-concepts/#document
#[derive(Debug, PartialEq, Eq)]
pub struct Document<'bmp> {
  pub header: Option<DocHeader<'bmp>>,
  pub content: DocContent<'bmp>,
}

impl<'bmp> Document<'bmp> {
  pub fn new_in(bump: &'bmp bumpalo::Bump) -> Self {
    Self {
      header: None,
      content: DocContent::Blocks(bumpalo::vec![in bump]),
    }
  }
}
