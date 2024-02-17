use crate::internal::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ListItem<'bmp> {
  pub marker: ListMarker,
  pub marker_src: SourceString<'bmp>,
  pub principle: InlineNodes<'bmp>,
  pub blocks: BumpVec<'bmp, Block<'bmp>>,
}

impl<'bmp> ListItem<'bmp> {
  pub fn loc_start(&self) -> usize {
    self.marker_src.loc.start
  }
  pub fn loc_end(&self) -> Option<usize> {
    self.principle.last_loc_end()
  }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ListVariant {
  Ordered,
  Unordered,
}

impl ListVariant {
  pub fn to_context(&self) -> BlockContext {
    match self {
      ListVariant::Ordered => BlockContext::OrderedList,
      ListVariant::Unordered => BlockContext::UnorderedList,
    }
  }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ListMarker {
  Dot(u8),
  Digits(u32),
  Dash,
  Star(u8),
}

impl From<ListMarker> for ListVariant {
  fn from(marker: ListMarker) -> Self {
    match marker {
      ListMarker::Dot(_) => ListVariant::Ordered,
      ListMarker::Digits(_) => ListVariant::Ordered,
      ListMarker::Dash => ListVariant::Unordered,
      ListMarker::Star(_) => ListVariant::Unordered,
    }
  }
}
