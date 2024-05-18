use std::collections::HashMap;

use crate::internal::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttrValue {
  String(String),
  Bool(bool),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttrEntry {
  pub readonly: bool,
  pub value: AttrValue,
}

impl AttrEntry {
  pub const fn new(value: AttrValue) -> Self {
    AttrEntry { readonly: false, value }
  }
}

impl AttrValue {
  pub const fn is_true(&self) -> bool {
    matches!(self, AttrValue::Bool(true))
  }

  pub const fn is_false(&self) -> bool {
    matches!(self, AttrValue::Bool(false))
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AttrEntries(HashMap<String, AttrEntry>);

impl AttrEntries {
  pub fn insert(&mut self, key: impl Into<String>, value: AttrEntry) {
    self.0.insert(key.into(), value);
  }

  pub fn remove(&mut self, key: &str) -> Option<AttrValue> {
    self.0.remove(key).map(|entry| entry.value)
  }

  pub fn get(&self, key: &str) -> Option<&AttrValue> {
    self.0.get(key).map(|entry| &entry.value)
  }

  pub fn is_true(&self, key: &str) -> bool {
    self.get(key).map_or(false, |entry| entry.is_true())
  }

  pub fn is_false(&self, key: &str) -> bool {
    self.get(key).map_or(false, |entry| entry.is_false())
  }

  pub fn is_set(&self, key: &str) -> bool {
    self.0.contains_key(key)
  }

  pub fn str(&self, key: &str) -> Option<&str> {
    match self.get(key) {
      Some(AttrValue::String(s)) => Some(s),
      _ => None,
    }
  }

  pub fn u8(&self, key: &str) -> Option<u8> {
    match self.get(key) {
      Some(AttrValue::String(s)) => s.parse().ok(),
      _ => None,
    }
  }

  pub fn isize(&self, key: &str) -> Option<isize> {
    match self.get(key) {
      Some(AttrValue::String(s)) => s.parse().ok(),
      _ => None,
    }
  }

  pub fn str_or(&self, key: &str, default: &'static str) -> &str {
    match self.get(key) {
      Some(AttrValue::String(s)) => s,
      _ => default,
    }
  }

  pub fn u8_or(&self, key: &str, default: u8) -> u8 {
    match self.get(key) {
      Some(AttrValue::String(s)) => s.parse().unwrap_or(default),
      _ => default,
    }
  }

  // https://docs.asciidoctor.org/asciidoc/latest/attributes/character-replacement-ref/
  pub fn builtin(&self, key: &str) -> Option<&'static str> {
    match key {
      "blank" | "empty" => Some(""),
      "sp" => Some(" "),
      "nbsp" => Some("&#160;"),
      "zwsp" => Some("&#8203;"),
      "wj" => Some("&#8288;"),
      "apos" => Some("&#39;"),
      "quot" => Some("&#34;"),
      "lsquo" => Some("&#8216;"),
      "rsquo" => Some("&#8217;"),
      "ldquo" => Some("&#8220;"),
      "rdquo" => Some("&#8221;"),
      "deg" => Some("&#176;"),
      "plus" => Some("&#43;"),
      "brvbar" => Some("&#166;"),
      "vbar" => Some("|"),
      "amp" => Some("&"),
      "lt" => Some("<"),
      "gt" => Some(">"),
      "startsb" => Some("["),
      "endsb" => Some("]"),
      "caret" => Some("^"),
      "asterisk" => Some("*"),
      "tilde" => Some("~"),
      "backslash" => Some("\\"),
      "backtick" => Some("`"),
      "two-colons" => Some("::"),
      "two-semicolons" => Some(";;"),
      "cpp" => Some("C++"),
      "pp" => Some("&#43;&#43;"),
      _ => None,
    }
  }
}

impl Json for AttrValue {
  fn to_json_in(&self, buf: &mut JsonBuf) {
    match self {
      AttrValue::String(s) => s.to_json_in(buf),
      AttrValue::Bool(b) => b.to_json_in(buf),
    }
  }
}

impl Json for AttrEntry {
  fn to_json_in(&self, buf: &mut JsonBuf) {
    buf.begin_obj("AttrEntry");
    buf.add_member("readonly", &self.readonly);
    buf.add_member("value", &self.value);
    buf.finish_obj();
  }
}

impl Json for AttrEntries {
  fn to_json_in(&self, buf: &mut JsonBuf) {
    self.0.to_json_in(buf);
  }
}
