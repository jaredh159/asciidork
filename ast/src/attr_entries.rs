use std::collections::HashMap;

use crate::internal::*;

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

  pub fn str(&self, key: &str) -> Option<&str> {
    match self.get(key) {
      Some(AttrEntry::String(s)) => Some(s),
      _ => None,
    }
  }

  pub fn str_or(&self, key: &str, default: &'static str) -> &str {
    match self.get(key) {
      Some(AttrEntry::String(s)) => s,
      _ => default,
    }
  }
}
