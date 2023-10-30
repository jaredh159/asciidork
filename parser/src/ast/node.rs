use bumpalo::collections::Vec;

use super::{block::Block, doc_content::DocContent, DocHeader, Inline};

#[derive(Debug, PartialEq, Eq)]
pub struct Section<'alloc> {
  level: u8,
  heading: Heading<'alloc>,
  blocks: Vec<'alloc, Block<'alloc>>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Heading<'alloc> {
  inlines: Vec<'alloc, Inline<'alloc>>,
}

// https://docs.asciidoctor.org/asciidoc/latest/key-concepts/#document
#[derive(Debug, PartialEq, Eq)]
pub struct Document<'alloc> {
  pub header: Option<DocHeader<'alloc>>,
  pub content: DocContent<'alloc>,
}

impl<'alloc> Document<'alloc> {
  pub fn new_in(allocator: &'alloc bumpalo::Bump) -> Self {
    Self {
      header: None,
      content: DocContent::Blocks(bumpalo::vec![in allocator]),
    }
  }
}
