use std::collections::HashMap;
use std::fmt::{Debug, Write};
use std::ops::{Deref, DerefMut};

use crate::internal::*;

pub trait Json {
  fn to_json_in(&self, buf: &mut JsonBuf);

  fn to_json(&self) -> String {
    let mut buf = JsonBuf(String::with_capacity(self.size_hint()));
    self.to_json_in(&mut buf);
    buf.0
  }

  fn size_hint(&self) -> usize {
    256
  }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct JsonBuf(String);

impl Deref for JsonBuf {
  type Target = String;
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl DerefMut for JsonBuf {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

impl JsonBuf {
  pub fn begin_obj(&mut self, ty: &str) {
    self.push_str(r#"{"type":""#);
    self.push_str(ty);
    self.push('"');
  }

  pub fn finish_obj(&mut self) {
    self.push('}');
  }

  pub fn start_obj_enum_type(&mut self, ty: &str) {
    self.begin_obj(ty);
    self.push_str(r#","variant":""#);
  }

  pub fn push_obj_enum_type<V: Debug>(&mut self, ty: &str, value: V) {
    self.start_obj_enum_type(ty);
    self.push_simple_variant(value);
    self.push_str("\"}");
  }

  pub fn push_simple_variant<V: Debug>(&mut self, variant: V) {
    debug_assert!(format!("{:?}", variant)
      .chars()
      .all(|c| c.is_ascii_alphanumeric()));
    write!(self, "{:?}", variant).unwrap();
  }

  pub fn add_member<T: Json>(&mut self, name: &str, value: &T) {
    self.push_str(",\"");
    self.push_str(name);
    self.push_str("\":");
    value.to_json_in(self);
  }

  pub fn add_option_member<T: Json>(&mut self, name: &str, value: Option<&T>) {
    if let Some(value) = value {
      self.add_member(name, value);
    }
  }
}

impl Json for str {
  fn to_json_in(&self, buf: &mut JsonBuf) {
    buf.push('"');
    for c in self.chars() {
      match c {
        '"' => buf.push_str(r#"\""#),
        '\n' => buf.push_str(r#"\\n"#),
        _ => buf.push(c),
      }
    }
    buf.push('"');
  }
}

impl Json for &str {
  fn to_json_in(&self, buf: &mut JsonBuf) {
    (*self).to_json_in(buf);
  }
}

impl<'bmp> Json for BumpString<'bmp> {
  fn to_json_in(&self, buf: &mut JsonBuf) {
    self.as_str().to_json_in(buf);
  }
}

impl Json for usize {
  fn to_json_in(&self, buf: &mut JsonBuf) {
    write!(buf, "{}", self).unwrap();
  }
}

impl Json for u8 {
  fn to_json_in(&self, buf: &mut JsonBuf) {
    write!(buf, "{}", self).unwrap();
  }
}

impl Json for u16 {
  fn to_json_in(&self, buf: &mut JsonBuf) {
    write!(buf, "{}", self).unwrap();
  }
}

impl Json for bool {
  fn to_json_in(&self, buf: &mut JsonBuf) {
    buf.push_str(if *self { "true" } else { "false" });
  }
}

impl<T: Json> Json for Option<T> {
  fn to_json_in(&self, buf: &mut JsonBuf) {
    match self {
      Some(value) => value.to_json_in(buf),
      None => buf.push_str("null"),
    }
  }
}

impl<T: Json> Json for [T] {
  fn to_json_in(&self, buf: &mut JsonBuf) {
    if self.is_empty() {
      buf.push_str("[]");
      return;
    }
    buf.push('[');
    for item in self.iter() {
      item.to_json_in(buf);
      buf.push(',');
    }
    buf.pop();
    buf.push(']');
  }
}

impl<T: Json> Json for &[T] {
  fn to_json_in(&self, buf: &mut JsonBuf) {
    (*self).to_json_in(buf);
  }
}

impl Json for String {
  fn to_json_in(&self, buf: &mut JsonBuf) {
    self.as_str().to_json_in(buf);
  }
}

impl<K: Json, V: Json> Json for HashMap<K, V> {
  fn to_json_in(&self, buf: &mut JsonBuf) {
    buf.push('{');
    let mut first = true;
    for (key, value) in self {
      if first {
        first = false;
      } else {
        buf.push(',');
      }
      key.to_json_in(buf);
      buf.push(':');
      value.to_json_in(buf);
    }
    buf.push('}');
  }
}

impl<T: Json> Json for BumpVec<'_, T> {
  fn to_json_in(&self, buf: &mut JsonBuf) {
    self.as_slice().to_json_in(buf);
  }
}

#[cfg(test)]
#[macro_export]
macro_rules! assert_json {
  ($input:expr, $expected:expr$(,)?) => {{
    let json = $input.to_json();
    test_utils::assert_eq!(json, jsonxf::minimize($expected).unwrap());
    // assert that the JSON is valid
    assert!(serde_json::from_str::<serde_json::Value>(&json).is_ok());
  }};
}

#[cfg(test)]
mod tests {
  use super::*;
  use test_utils::*;

  #[test]
  fn test_inline_to_json() {
    let cases = [
      (
        Inline::Discarded,
        r#"{
          "type": "Inline",
          "variant": "Discarded"
        }"#,
      ),
      (
        Inline::AttributeReference(bstr!("foo\"bar\"")),
        r#"{
          "type": "Inline",
          "variant": "AttributeReference",
          "name": "foo\"bar\""
        }"#,
      ),
      (
        Inline::CurlyQuote(CurlyKind::LeftDouble),
        r#"{
          "type": "Inline",
          "variant": "CurlyQuote",
          "kind": {
            "type": "CurlyKind",
            "variant": "LeftDouble"
          }
        }"#,
      ),
      (
        Inline::Bold(nodes![node!(Inline::Discarded, 0..1)]),
        r#"{
          "type": "Inline",
          "variant": "Bold",
          "children": [
            {
              "type": "InlineNode",
              "content": {
                "type": "Inline",
                "variant": "Discarded"
              }
            }
          ]
        }"#,
      ),
      (
        Inline::Macro(MacroNode::Image {
          flow: Flow::Block,
          target: src!("cat.jpg", 0..0),
          attrs: AttrList::new(SourceLocation::new(0, 0), leaked_bump()),
        }),
        r#"{
          "type": "Inline",
          "variant": "Macro",
          "macro": {
            "type": "MacroNode",
            "variant": "Image",
            "flow": {
              "type": "Flow",
              "variant": "Block"
            },
            "target": "cat.jpg"
          }
        }"#,
      ),
    ];
    for (input, expected) in cases.iter() {
      assert_json!(input, expected);
    }
  }
}
