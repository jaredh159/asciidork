use crate::ast::*;
use crate::utils::bump::*;

pub use bumpalo::Bump;

pub trait BumpTestHelpers<'bmp> {
  fn vec<const N: usize, T: Clone>(&'bmp self, nodes: [T; N]) -> Vec<'bmp, T>;
  fn s(&'bmp self, s: &'static str) -> String<'bmp>;
  fn src(&'bmp self, s: &'static str, loc: SourceLocation) -> SourceString<'bmp>;
}

impl<'bmp> BumpTestHelpers<'bmp> for &bumpalo::Bump {
  fn vec<const N: usize, T: Clone>(&'bmp self, nodes: [T; N]) -> Vec<'bmp, T> {
    let mut vec = Vec::new_in(self);
    for node in nodes.iter() {
      vec.push(node.clone());
    }
    vec
  }

  fn s(&'bmp self, s: &'static str) -> String<'bmp> {
    String::from_str_in(s, self)
  }

  fn src(&'bmp self, s: &'static str, loc: SourceLocation) -> SourceString<'bmp> {
    SourceString::new(self.s(s), loc)
  }
}

pub fn l(start: usize, end: usize) -> SourceLocation {
  SourceLocation::new(start, end)
}

pub fn inode(content: Inline, loc: SourceLocation) -> InlineNode {
  InlineNode::new(content, loc)
}

macro_rules! s {
  (in $bump:expr; $s:expr) => {
    bumpalo::collections::String::from_str_in($s, $bump)
  };
}

pub(crate) use s;