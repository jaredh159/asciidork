use std::collections::HashMap;

use crate::internal::*;

// https://docs.asciidoctor.org/asciidoc/latest/document/header/
#[derive(Debug, PartialEq, Eq)]
pub struct DocHeader<'bmp> {
  pub title: Option<DocTitle<'bmp>>,
  pub authors: Vec<'bmp, Author<'bmp>>,
  pub revision: Option<Revision<'bmp>>,
  // 👍 thurs jared: make non optional up at the doc level, maybe with
  // an empty entries that gets handed out if doc header not present
  pub attrs: AttrEntries,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttrEntry {
  String(StdString),
  Bool(bool),
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AttrEntries(HashMap<StdString, AttrEntry>);

impl AttrEntries {
  pub fn new() -> Self {
    Self(HashMap::new())
  }

  pub fn insert(&mut self, key: StdString, value: AttrEntry) {
    self.0.insert(key, value);
  }

  pub fn get(&self, key: &str) -> Option<&AttrEntry> {
    self.0.get(key)
  }

  pub fn is_set(&self, key: &str) -> bool {
    matches!(self.get(key), Some(AttrEntry::Bool(true)))
  }

  pub fn is_unset(&self, key: &str) -> bool {
    matches!(self.get(key), Some(AttrEntry::Bool(false)))
  }
}

#[derive(Debug, PartialEq, Eq)]
pub struct DocTitle<'bmp> {
  pub heading: Vec<'bmp, InlineNode<'bmp>>,
  pub subtitle: Option<Vec<'bmp, InlineNode<'bmp>>>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Revision<'bmp> {
  pub version: String<'bmp>,
  pub date: Option<String<'bmp>>,
  pub remark: Option<String<'bmp>>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Author<'bmp> {
  pub first_name: String<'bmp>,
  pub middle_name: Option<String<'bmp>>,
  pub last_name: String<'bmp>,
  pub email: Option<String<'bmp>>,
}
