use std::collections::HashMap;
use std::{cell::RefCell, rc::Rc};

use crate::internal::*;

// https://docs.asciidoctor.org/asciidoc/latest/key-concepts/#document
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Document<'arena> {
  pub meta: DocumentMeta,
  pub header: Option<DocHeader<'arena>>,
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
      header: None,
      content,
      toc: None,
      anchors: Rc::new(RefCell::new(HashMap::new())),
      meta: DocumentMeta::default(),
      source_filenames: Vec::new(),
    }
  }

  pub fn title(&self) -> Option<&DocTitle<'arena>> {
    self
      .header
      .as_ref()
      .and_then(|header| header.title.as_ref())
  }

  pub fn last_loc(&self) -> Option<SourceLocation> {
    match &self.content {
      DocContent::Parts(book) => book.last_loc(),
      DocContent::Sections(sectioned) => sectioned.last_loc(),
      DocContent::Blocks(blocks) => blocks.last().and_then(|b| b.content.last_loc()),
    }
  }
}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct DocHeader<'arena> {
  pub title: Option<DocTitle<'arena>>,
  pub loc: SourceLocation,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct DocTitle<'arena> {
  pub attrs: MultiAttrList<'arena>,
  pub main: InlineNodes<'arena>,
  pub subtitle: Option<InlineNodes<'arena>>,
}
