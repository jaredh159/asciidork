use std::fmt::{Debug, Formatter, Result};

use crate::internal::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ListItem<'bmp> {
  pub marker: ListMarker,
  pub marker_src: SourceString<'bmp>,
  pub principle: InlineNodes<'bmp>,
  pub type_meta: ListItemTypeMeta<'bmp>,
  pub blocks: BumpVec<'bmp, Block<'bmp>>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ListItemTypeMeta<'bmp> {
  Checklist(bool, SourceString<'bmp>),
  Callout(SmallVec<[Callout; 4]>),
  None,
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

  pub const fn is_checklist(&self) -> bool {
    matches!(self.type_meta, ListItemTypeMeta::Checklist(_, _))
  }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ListVariant {
  Ordered,
  Unordered,
  Description,
  Callout,
}

impl ListVariant {
  pub const fn to_context(&self) -> BlockContext {
    match self {
      ListVariant::Ordered => BlockContext::OrderedList,
      ListVariant::Unordered => BlockContext::UnorderedList,
      ListVariant::Description => BlockContext::DescriptionList,
      ListVariant::Callout => BlockContext::CalloutList,
    }
  }
}

#[derive(Clone, Copy, Eq, PartialEq)]
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
  // callout
  Callout(Option<u8>),
}

impl Debug for ListMarker {
  fn fmt(&self, f: &mut Formatter) -> Result {
    match self {
      ListMarker::Dot(num) => write!(f, "Dot({})", num),
      ListMarker::Digits(num) => write!(f, "Digits({})", num),
      ListMarker::Dash => write!(f, "Dash"),
      ListMarker::Star(num) => write!(f, "Star({})", num),
      ListMarker::Colons(num) => write!(f, "Colons({})", num),
      ListMarker::SemiColons => write!(f, "SemiColons"),
      ListMarker::Callout(Some(num)) => write!(f, "Callout({})", num),
      ListMarker::Callout(None) => write!(f, "Callout(None)"),
    }
  }
}

impl ListMarker {
  pub const fn is_description(&self) -> bool {
    matches!(self, ListMarker::Colons(_) | ListMarker::SemiColons)
  }

  pub const fn callout_num(&self) -> Option<u8> {
    match self {
      ListMarker::Callout(Some(num)) => Some(*num),
      _ => None,
    }
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
      ListMarker::Callout(_) => ListVariant::Callout,
    }
  }
}
