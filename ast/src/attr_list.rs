use crate::internal::*;

// https://docs.asciidoctor.org/asciidoc/latest/attributes/positional-and-named-attributes/
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AttrList<'bmp> {
  pub positional: BumpVec<'bmp, Option<BumpVec<'bmp, InlineNode<'bmp>>>>,
  pub named: Named<'bmp>,
  pub id: Option<SourceString<'bmp>>,
  pub roles: BumpVec<'bmp, SourceString<'bmp>>,
  pub options: BumpVec<'bmp, SourceString<'bmp>>,
  pub loc: SourceLocation,
}

impl<'bmp> AttrList<'bmp> {
  pub fn new(loc: SourceLocation, bump: &'bmp Bump) -> Self {
    AttrList {
      positional: BumpVec::new_in(bump),
      named: Named::new_in(bump),
      id: None,
      roles: BumpVec::new_in(bump),
      options: BumpVec::new_in(bump),
      loc,
    }
  }

  /// https://docs.asciidoctor.org/asciidoc/latest/blocks/#block-style
  pub fn block_style(&self) -> Option<BlockContext> {
    if let Some(first_positional) = self.str_positional_at(0) {
      BlockContext::derive(first_positional)
    } else {
      None
    }
  }

  pub fn named(&self, key: &str) -> Option<&str> {
    self.named.get(key).map(|s| s.src.as_str())
  }

  pub fn str_positional_at(&self, index: usize) -> Option<&str> {
    let Some(Some(nodes)) = self.positional.get(index) else {
      return None;
    };
    if nodes.len() != 1 {
      return None;
    }
    let Inline::Text(positional) = &nodes[0].content else {
      return None;
    };
    Some(positional.as_str())
  }

  // todo: rename, make test or something...
  pub fn positional(
    positional: &'static str,
    loc: SourceLocation,
    bump: &'bmp Bump,
  ) -> AttrList<'bmp> {
    AttrList {
      positional: bvec![in bump; Some(bvec![in bump;
        InlineNode::new(
          Inline::Text(BumpString::from_str_in(positional, bump)),
          SourceLocation::new(loc.start, loc.end),
        )
      ])],
      ..AttrList::new(SourceLocation::new(loc.start - 1, loc.end + 1), bump)
    }
  }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Named<'bmp>(BumpVec<'bmp, (SourceString<'bmp>, SourceString<'bmp>)>);

impl<'bmp> Named<'bmp> {
  pub fn new_in(bump: &'bmp Bump) -> Self {
    Named(BumpVec::new_in(bump))
  }

  pub fn from(vec: BumpVec<'bmp, (SourceString<'bmp>, SourceString<'bmp>)>) -> Self {
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
