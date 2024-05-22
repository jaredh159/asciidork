use std::collections::HashMap;

use crate::internal::*;

// https://docs.asciidoctor.org/asciidoc/latest/key-concepts/#document
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Document<'bmp> {
  pub meta: DocumentMeta,
  pub title: Option<InlineNodes<'bmp>>,
  pub subtitle: Option<InlineNodes<'bmp>>,
  pub content: DocContent<'bmp>,
  pub toc: Option<TableOfContents<'bmp>>,
  pub anchors: HashMap<BumpString<'bmp>, Anchor<'bmp>>,
}

impl<'bmp> Document<'bmp> {
  pub fn new(bump: &'bmp Bump) -> Self {
    Document::from_content(DocContent::Blocks(bvec![in bump]))
  }

  pub fn from_content(content: DocContent<'bmp>) -> Self {
    Self {
      title: None,
      subtitle: None,
      content,
      toc: None,
      anchors: HashMap::new(),
      meta: DocumentMeta::default(),
    }
  }
}
