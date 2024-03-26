use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttrEntry {
  String(String),
  Bool(bool),
}

impl AttrEntry {
  pub fn is_set(&self) -> bool {
    matches!(self, AttrEntry::Bool(true))
  }

  pub fn is_unset(&self) -> bool {
    matches!(self, AttrEntry::Bool(false))
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AttrEntries(HashMap<String, AttrEntry>);

impl AttrEntries {
  pub fn new() -> Self {
    Self(HashMap::new())
  }

  pub fn insert(&mut self, key: String, value: AttrEntry) {
    self.0.insert(key, value);
  }

  pub fn get(&self, key: &str) -> Option<&AttrEntry> {
    self.0.get(key)
  }

  pub fn is_set(&self, key: &str) -> bool {
    self.get(key).map_or(false, |entry| entry.is_set())
  }

  pub fn is_unset(&self, key: &str) -> bool {
    self.get(key).map_or(false, |entry| entry.is_unset())
  }

  pub fn str(&self, key: &str) -> Option<&str> {
    match self.get(key) {
      Some(AttrEntry::String(s)) => Some(s),
      _ => None,
    }
  }

  pub fn u8(&self, key: &str) -> Option<u8> {
    match self.get(key) {
      Some(AttrEntry::String(s)) => s.parse().ok(),
      _ => None,
    }
  }

  pub fn isize(&self, key: &str) -> Option<isize> {
    match self.get(key) {
      Some(AttrEntry::String(s)) => s.parse().ok(),
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
