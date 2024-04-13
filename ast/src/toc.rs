use crate::internal::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TableOfContents<'bmp> {
  pub title: BumpString<'bmp>,
  pub nodes: BumpVec<'bmp, TocNode<'bmp>>,
  pub position: TocPosition,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TocNode<'bmp> {
  pub level: u8,
  pub title: InlineNodes<'bmp>,
  pub id: Option<BumpString<'bmp>>,
  pub children: BumpVec<'bmp, TocNode<'bmp>>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TocPosition {
  Left,
  Right,
  Preamble,
  Macro,
  Auto,
}
