use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttrValue {
  Str(&'static str),
  String(String),
  Bool(bool),
}

pub trait ReadAttr {
  fn get(&self, key: &str) -> Option<&AttrValue>;

  fn is_true(&self, key: &str) -> bool {
    self.get(key).map_or(false, |attr| attr.is_true())
  }

  fn is_false(&self, key: &str) -> bool {
    self.get(key).map_or(false, |attr| attr.is_false())
  }

  fn is_set(&self, key: &str) -> bool {
    self.get(key).is_some()
  }

  fn str(&self, key: &str) -> Option<&str> {
    self.get(key).and_then(|v| v.str())
  }

  fn u8(&self, key: &str) -> Option<u8> {
    match self.get(key) {
      Some(AttrValue::String(s)) => s.parse().ok(),
      Some(AttrValue::Str(s)) => s.parse().ok(),
      _ => None,
    }
  }

  fn isize(&self, key: &str) -> Option<isize> {
    match self.get(key) {
      Some(AttrValue::String(s)) => s.parse().ok(),
      _ => None,
    }
  }

  fn str_or(&self, key: &str, default: &'static str) -> &str {
    self.str(key).unwrap_or(default)
  }

  fn u8_or(&self, key: &str, default: u8) -> u8 {
    match self.get(key) {
      Some(AttrValue::String(s)) => s.parse().unwrap_or(default),
      _ => default,
    }
  }

  fn true_if(&self, predicate: bool) -> Option<&AttrValue> {
    if predicate {
      Some(&AttrValue::Bool(true))
    } else {
      None
    }
  }
}

impl AttrValue {
  pub const fn is_true(&self) -> bool {
    matches!(self, AttrValue::Bool(true))
  }

  pub const fn is_false(&self) -> bool {
    matches!(self, AttrValue::Bool(false))
  }

  pub fn str(&self) -> Option<&str> {
    match self {
      AttrValue::Str(s) => Some(s),
      AttrValue::String(s) => Some(s),
      _ => None,
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Attrs(HashMap<String, AttrValue>);

impl Attrs {
  pub fn insert(&mut self, key: impl Into<String>, value: AttrValue) {
    self.0.insert(key.into(), value);
  }
}

impl ReadAttr for Attrs {
  fn get(&self, key: &str) -> Option<&AttrValue> {
    self.0.get(key)
  }
}
