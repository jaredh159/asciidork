use crate::internal::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ListItem<'bmp> {
  pub marker: ListMarker,
  pub marker_src: SourceString<'bmp>,
  pub principle: InlineNodes<'bmp>,
  pub checklist: Option<(bool, SourceString<'bmp>)>,
  pub blocks: BumpVec<'bmp, Block<'bmp>>,
}

impl<'bmp> ListItem<'bmp> {
  pub const fn loc_start(&self) -> usize {
    self.marker_src.loc.start
  }

  pub fn last_loc(&self) -> Option<SourceLocation> {
    self
      .blocks
      .last()
      .map(|block| block.loc)
      .or_else(|| self.principle.last_loc())
  }

  pub fn last_loc_end(&self) -> Option<usize> {
    self.last_loc().map(|loc| loc.end)
  }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ListVariant {
  Ordered,
  Unordered,
  Description,
}

impl ListVariant {
  pub const fn to_context(&self) -> BlockContext {
    match self {
      ListVariant::Ordered => BlockContext::OrderedList,
      ListVariant::Unordered => BlockContext::UnorderedList,
      ListVariant::Description => BlockContext::DescriptionList,
    }
  }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ListMarker {
  // ordered
  Dot(u8),
  Digits(u16),
  // unordered
  Dash,
  Star(u8),
  // description
  Colons(u8),
  SemiColons,
}

impl ListMarker {
  pub const fn is_description(&self) -> bool {
    matches!(self, ListMarker::Colons(_) | ListMarker::SemiColons)
  }
}

impl From<ListMarker> for ListVariant {
  fn from(marker: ListMarker) -> Self {
    match marker {
      ListMarker::Dot(_) => ListVariant::Ordered,
      ListMarker::Digits(_) => ListVariant::Ordered,
      ListMarker::Dash => ListVariant::Unordered,
      ListMarker::Star(_) => ListVariant::Unordered,
      ListMarker::Colons(_) => ListVariant::Description,
      ListMarker::SemiColons => ListVariant::Description,
    }
  }
}
