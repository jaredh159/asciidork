use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttrValue {
  String(String),
  Bool(bool),
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
      AttrValue::String(s) => Some(s),
      AttrValue::Bool(true) => Some(""),
      AttrValue::Bool(false) => None,
    }
  }
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

  fn is_unset(&self, key: &str) -> bool {
    self.get(key).is_none()
  }

  fn str(&self, key: &str) -> Option<&str> {
    self.get(key).and_then(|v| v.str())
  }

  fn string(&self, key: &str) -> Option<String> {
    self.get(key).and_then(|v| v.str()).map(ToOwned::to_owned)
  }

  fn u8(&self, key: &str) -> Option<u8> {
    match self.get(key) {
      Some(AttrValue::String(s)) => s.parse().ok(),
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

  fn string_or(&self, key: &str, default: &'static str) -> String {
    self.str(key).map_or(default.to_owned(), ToOwned::to_owned)
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

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Attrs(HashMap<String, AttrValue>);

impl AsRef<HashMap<String, AttrValue>> for Attrs {
  fn as_ref(&self) -> &HashMap<String, AttrValue> {
    &self.0
  }
}

impl ReadAttr for Attrs {
  fn get(&self, key: &str) -> Option<&AttrValue> {
    self.0.get(key)
  }
}

impl Attrs {
  pub fn empty() -> Self {
    Self(HashMap::new())
  }

  pub fn defaults() -> Self {
    let attrs = Self(HashMap::from_iter(
      [
        ("empty", ""),
        ("blank", ""),
        ("attribute-missing", "skip"),
        ("attribute-undefined", "drop-line"),
        ("appendix-caption", "Appendix"),
        ("appendix-refsig", "Appendix"),
        ("caution-caption", "Caution"),
        ("chapter-refsig", "Chapter"),
        ("example-caption", "Example"),
        ("figure-caption", "Figure"),
        ("important-caption", "Important"),
        ("last-update-label", "Last updated"),
        ("note-caption", "Note"),
        ("part-refsig", "Part"),
        ("section-refsig", "Section"),
        ("table-caption", "Table"),
        ("tip-caption", "Tip"),
        ("toc-title", "Table of Contents"),
        ("untitled-label", "Untitled"),
        ("version-label", "Version"),
        ("warning-caption", "Warning"),
        ("sp", " "),
        ("nbsp", "&#160;"),
        ("zwsp", "&#8203;"),
        ("wj", "&#8288;"),
        ("apos", "&#39;"),
        ("quot", "&#34;"),
        ("lsquo", "&#8216;"),
        ("rsquo", "&#8217;"),
        ("ldquo", "&#8220;"),
        ("rdquo", "&#8221;"),
        ("deg", "&#176;"),
        ("plus", "&#43;"),
        ("brvbar", "&#166;"),
        ("vbar", "|"),
        ("amp", "&"),
        ("lt", "<"),
        ("gt", ">"),
        ("startsb", "["),
        ("endsb", "]"),
        ("caret", "^"),
        ("asterisk", "*"),
        ("tilde", "~"),
        ("backslash", "\\"),
        ("backtick", "`"),
        ("two-colons", "::"),
        ("two-semicolons", ";;"),
        ("cpp", "C++"),
        ("pp", "&#43;&#43;"),
      ]
      .iter()
      .map(|(k, v)| (k.to_string(), (*v).into())),
    ));
    attrs
  }

  pub fn insert(&mut self, key: impl Into<String>, value: AttrValue) {
    self.0.insert(key.into(), value);
  }
}

impl From<&str> for AttrValue {
  fn from(s: &str) -> Self {
    AttrValue::String(s.to_owned())
  }
}

impl From<bool> for AttrValue {
  fn from(b: bool) -> Self {
    AttrValue::Bool(b)
  }
}

impl From<String> for AttrValue {
  fn from(s: String) -> Self {
    AttrValue::String(s)
  }
}
