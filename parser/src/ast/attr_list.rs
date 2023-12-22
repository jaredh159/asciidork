use crate::prelude::*;

// https://docs.asciidoctor.org/asciidoc/latest/attributes/positional-and-named-attributes/
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AttrList<'bmp> {
  pub positional: Vec<'bmp, Option<Vec<'bmp, InlineNode<'bmp>>>>,
  pub named: Named<'bmp>,
  pub id: Option<SourceString<'bmp>>,
  pub roles: Vec<'bmp, SourceString<'bmp>>,
  pub options: Vec<'bmp, SourceString<'bmp>>,
  pub loc: SourceLocation,
}

impl<'bmp> AttrList<'bmp> {
  pub fn new(loc: SourceLocation, bump: &'bmp Bump) -> Self {
    AttrList {
      positional: Vec::new_in(bump),
      named: Named::new_in(bump),
      id: None,
      roles: Vec::new_in(bump),
      options: Vec::new_in(bump),
      loc,
    }
  }

  /// https://docs.asciidoctor.org/asciidoc/latest/blocks/#block-style
  pub fn block_style(&self) -> Option<BlockContext> {
    let Some(Some(nodes)) = self.positional.get(0) else {
      return None;
    };
    if nodes.len() != 1 {
      return None;
    }
    let Inline::Text(first_positional) = &nodes[0].content else {
      return None;
    };
    BlockContext::derive(first_positional.as_str())
  }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Named<'bmp>(Vec<'bmp, (SourceString<'bmp>, SourceString<'bmp>)>);

impl<'bmp> Named<'bmp> {
  pub fn new_in(bump: &'bmp Bump) -> Self {
    Named(Vec::new_in(bump))
  }

  pub fn from(vec: Vec<'bmp, (SourceString<'bmp>, SourceString<'bmp>)>) -> Self {
    Named(vec)
  }

  pub fn insert(&mut self, key: SourceString<'bmp>, value: SourceString<'bmp>) {
    self.0.push((key, value));
  }

  pub fn get(&self, key: &str) -> Option<&SourceString<'bmp>> {
    self
      .0
      .iter()
      .find_map(|(k, v)| if k == key { Some(v) } else { None })
  }
}
