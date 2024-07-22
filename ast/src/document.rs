use std::collections::HashMap;
use std::{cell::RefCell, rc::Rc};

use crate::internal::*;

// https://docs.asciidoctor.org/asciidoc/latest/key-concepts/#document
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Document<'arena> {
  pub meta: DocumentMeta,
  pub title: Option<InlineNodes<'arena>>,
  pub subtitle: Option<InlineNodes<'arena>>,
  pub content: DocContent<'arena>,
  pub toc: Option<TableOfContents<'arena>>,
  pub anchors: Rc<RefCell<HashMap<BumpString<'arena>, Anchor<'arena>>>>,
  pub source_filenames: Vec<String>,
}

impl<'arena> Document<'arena> {
  pub fn new(bump: &'arena Bump) -> Self {
    Document::from_content(DocContent::Blocks(bvec![in bump]))
  }

  pub fn from_content(content: DocContent<'arena>) -> Self {
    Self {
      title: None,
      subtitle: None,
      content,
      toc: None,
      anchors: Rc::new(RefCell::new(HashMap::new())),
      meta: DocumentMeta::default(),
      source_filenames: Vec::new(),
    }
  }
}
