use crate::internal::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TableOfContents<'arena> {
  pub title: BumpString<'arena>,
  pub nodes: BumpVec<'arena, TocNode<'arena>>,
  pub position: TocPosition,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TocNode<'arena> {
  pub level: u8,
  pub title: InlineNodes<'arena>,
  pub id: Option<BumpString<'arena>>,
  pub children: BumpVec<'arena, TocNode<'arena>>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TocPosition {
  Left,
  Right,
  Preamble,
  Macro,
  Auto,
}
