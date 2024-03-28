use crate::internal::*;

pub trait BumpTestHelpers<'bmp> {
  fn vec<const N: usize, T: Clone>(&'bmp self, nodes: [T; N]) -> BumpVec<'bmp, T>;
  fn s(&'bmp self, s: &'static str) -> BumpString<'bmp>;
  fn src(&'bmp self, s: &'static str, loc: SourceLocation) -> SourceString<'bmp>;
  fn inodes<const N: usize>(&'bmp self, nodes: [InlineNode<'bmp>; N]) -> InlineNodes<'bmp>;
  fn empty_block(&'bmp self, start: usize, end: usize) -> Block<'bmp>;
  fn positional_attrs(&'bmp self, positional: &'static str, loc: SourceLocation) -> AttrList<'bmp>;
}

impl<'bmp> BumpTestHelpers<'bmp> for &bumpalo::Bump {
  fn vec<const N: usize, T: Clone>(&'bmp self, nodes: [T; N]) -> BumpVec<'bmp, T> {
    let mut vec = BumpVec::new_in(self);
    for node in nodes.iter() {
      vec.push(node.clone());
    }
    vec
  }

  fn empty_block(&'bmp self, start: usize, end: usize) -> Block<'bmp> {
    Block {
      meta: ChunkMeta::empty(start),
      context: BlockContext::Paragraph,
      content: BlockContent::Simple(InlineNodes::new(self)),
      loc: SourceLocation::new(start, end),
    }
  }

  fn positional_attrs(&'bmp self, positional: &'static str, loc: SourceLocation) -> AttrList<'bmp> {
    AttrList {
      positional: bvec![in self; Some(bvec![in self;
        InlineNode::new(
          Inline::Text(BumpString::from_str_in(positional, self)),
          SourceLocation::new(loc.start, loc.end),
        )
      ].into())],
      ..AttrList::new(SourceLocation::new(loc.start - 1, loc.end + 1), self)
    }
  }

  fn s(&'bmp self, s: &'static str) -> BumpString<'bmp> {
    BumpString::from_str_in(s, self)
  }

  fn src(&'bmp self, s: &'static str, loc: SourceLocation) -> SourceString<'bmp> {
    SourceString::new(self.s(s), loc)
  }

  fn inodes<const N: usize>(&'bmp self, nodes: [InlineNode<'bmp>; N]) -> InlineNodes<'bmp> {
    let mut vec = BumpVec::new_in(self);
    for node in nodes.iter() {
      vec.push(node.clone());
    }
    vec.into()
  }
}

pub fn l(start: usize, end: usize) -> SourceLocation {
  SourceLocation::new(start, end)
}

pub fn inode(content: Inline, loc: SourceLocation) -> InlineNode {
  InlineNode::new(content, loc)
}

pub fn n(content: Inline, loc: SourceLocation) -> InlineNode {
  InlineNode::new(content, loc)
}

pub fn n_text<'bmp>(
  s: &'static str,
  start: usize,
  end: usize,
  bump: &'bmp Bump,
) -> InlineNode<'bmp> {
  InlineNode::new(
    Inline::Text(BumpString::from_str_in(s, bump)),
    SourceLocation::new(start, end),
  )
}

macro_rules! s {
  (in $bump:expr; $s:expr) => {
    bumpalo::collections::String::from_str_in($s, $bump)
  };
}

pub(crate) use s;
