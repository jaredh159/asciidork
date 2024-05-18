use std::collections::HashMap;

use crate::internal::*;

// https://docs.asciidoctor.org/asciidoc/latest/key-concepts/#document
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Document<'bmp> {
  _type: DocType,
  pub header: Option<DocHeader<'bmp>>,
  pub content: DocContent<'bmp>,
  pub toc: Option<TableOfContents<'bmp>>,
  pub anchors: HashMap<BumpString<'bmp>, Anchor<'bmp>>,
  pub attrs: AttrEntries,
}

impl<'bmp> Document<'bmp> {
  pub fn new(bump: &'bmp Bump) -> Self {
    Document::from_content(DocContent::Blocks(bvec![in bump]))
  }

  pub fn from_content(content: DocContent<'bmp>) -> Self {
    let mut document = Self {
      _type: DocType::default(),
      header: None,
      content,
      toc: None,
      anchors: HashMap::new(),
      attrs: AttrEntries::default(),
    };
    document.set_type(DocType::default());
    document
  }

  pub const fn get_type(&self) -> DocType {
    self._type
  }

  pub fn set_type(&mut self, kind: DocType) {
    self._type = kind;
    self.attrs.insert(
      "doctype",
      AttrEntry {
        readonly: false,
        value: AttrValue::String(kind.to_str().to_string()),
      },
    )
  }
}
