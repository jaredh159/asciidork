use std::ops::{Deref, DerefMut};

use crate::internal::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct MultiAttrList<'arena>(BumpVec<'arena, AttrList<'arena>>);

impl<'arena> MultiAttrList<'arena> {
  pub fn new_in(bump: &'arena Bump) -> Self {
    Self(BumpVec::new_in(bump))
  }

  pub fn push(&mut self, attr: AttrList<'arena>) {
    self.0.push(attr);
  }
}

impl AttrData for MultiAttrList<'_> {
  fn is_empty(&self) -> bool {
    self.0.is_empty() || self.0.iter().all(AttrList::is_empty)
  }

  fn roles(&self) -> impl Iterator<Item = &SourceString> {
    self.0.iter().flat_map(AttrList::roles)
  }

  fn id(&self) -> Option<&SourceString> {
    self.0.iter().find_map(AttrList::id)
  }

  fn str_positional_at(&self, index: usize) -> Option<&str> {
    self.0.iter().find_map(|attr| attr.str_positional_at(index))
  }

  fn has_option(&self, option: &str) -> bool {
    self.0.iter().any(|attr| attr.has_option(option))
  }

  fn has_str_positional(&self, positional: &str) -> bool {
    self.0.iter().any(|a| a.has_str_positional(positional))
  }

  fn is_source(&self) -> bool {
    self.0.iter().any(AttrList::is_source)
  }

  fn source_language(&self) -> Option<&str> {
    self.0.iter().find_map(AttrList::source_language)
  }

  fn has_role(&self, role: &str) -> bool {
    self.0.iter().any(|attr| attr.has_role(role))
  }

  fn named(&self, key: &str) -> Option<&str> {
    self.0.iter().find_map(|attr| attr.named(key))
  }

  fn named_with_loc(&self, key: &str) -> Option<(&str, SourceLocation)> {
    self.0.iter().find_map(|attr| attr.named_with_loc(key))
  }

  fn ordered_list_custom_number_style(&self) -> Option<&'static str> {
    self
      .0
      .iter()
      .find_map(AttrList::ordered_list_custom_number_style)
  }

  fn unordered_list_custom_marker_style(&self) -> Option<&'static str> {
    self
      .0
      .iter()
      .find_map(AttrList::unordered_list_custom_marker_style)
  }

  fn block_style(&self) -> Option<BlockContext> {
    self.0.iter().find_map(AttrList::block_style)
  }
}

impl<'arena> Deref for MultiAttrList<'arena> {
  type Target = BumpVec<'arena, AttrList<'arena>>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl DerefMut for MultiAttrList<'_> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

impl<'arena> From<BumpVec<'arena, AttrList<'arena>>> for MultiAttrList<'arena> {
  fn from(bumpvec: BumpVec<'arena, AttrList<'arena>>) -> Self {
    Self(bumpvec)
  }
}

pub struct NoAttrs;

impl AttrData for NoAttrs {
  fn is_empty(&self) -> bool {
    true
  }

  fn str_positional_at(&self, _index: usize) -> Option<&str> {
    None
  }

  fn has_option(&self, _option: &str) -> bool {
    false
  }

  fn has_str_positional(&self, _positional: &str) -> bool {
    false
  }

  fn is_source(&self) -> bool {
    false
  }

  fn source_language(&self) -> Option<&str> {
    None
  }

  fn has_role(&self, _role: &str) -> bool {
    false
  }

  fn named(&self, _key: &str) -> Option<&str> {
    None
  }

  fn named_with_loc(&self, _key: &str) -> Option<(&str, SourceLocation)> {
    None
  }

  fn ordered_list_custom_number_style(&self) -> Option<&'static str> {
    None
  }

  fn unordered_list_custom_marker_style(&self) -> Option<&'static str> {
    None
  }

  fn block_style(&self) -> Option<BlockContext> {
    None
  }

  fn id(&self) -> Option<&SourceString> {
    None
  }

  fn roles(&self) -> impl Iterator<Item = &SourceString> {
    [].iter()
  }
}
