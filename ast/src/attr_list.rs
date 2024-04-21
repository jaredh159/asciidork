use crate::internal::*;

// https://docs.asciidoctor.org/asciidoc/latest/attributes/positional-and-named-attributes/
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AttrList<'bmp> {
  pub positional: BumpVec<'bmp, Option<InlineNodes<'bmp>>>,
  pub named: Named<'bmp>,
  pub id: Option<SourceString<'bmp>>,
  pub roles: BumpVec<'bmp, SourceString<'bmp>>,
  pub options: BumpVec<'bmp, SourceString<'bmp>>,
  pub loc: SourceLocation,
}

impl<'bmp> AttrList<'bmp> {
  pub fn new(loc: SourceLocation, bump: &'bmp Bump) -> Self {
    AttrList {
      positional: BumpVec::new_in(bump),
      named: Named::new_in(bump),
      id: None,
      roles: BumpVec::new_in(bump),
      options: BumpVec::new_in(bump),
      loc,
    }
  }

  /// https://docs.asciidoctor.org/asciidoc/latest/blocks/#block-style
  pub fn block_style(&self) -> Option<BlockContext> {
    if let Some(first_positional) = self.str_positional_at(0) {
      BlockContext::derive(first_positional)
    } else {
      None
    }
  }

  // https://docs.asciidoctor.org/asciidoc/latest/lists/unordered/#custom-markers
  pub fn unordered_list_custom_marker_style(&self) -> Option<&'static str> {
    // documented to support these, but seems like in practice
    // they actually pass through ANY first positional attr
    match self.str_positional_at(0) {
      Some("square") => Some("square"),
      Some("circle") => Some("circle"),
      Some("disc") => Some("disc"),
      Some("none") => Some("none"),
      Some("no-bullet") => Some("no-bullet"),
      Some("unstyled") => Some("unstyled"),
      _ => None,
    }
  }

  // https://docs.asciidoctor.org/asciidoc/latest/lists/ordered/#styles
  pub fn ordered_list_custom_number_style(&self) -> Option<&'static str> {
    match self.str_positional_at(0) {
      Some("arabic") => Some("arabic"),
      Some("decimal") => Some("decimal"), // html only
      Some("loweralpha") => Some("loweralpha"),
      Some("upperalpha") => Some("upperalpha"),
      Some("lowerroman") => Some("lowerroman"),
      Some("upperroman") => Some("upperroman"),
      Some("lowergreek") => Some("lowergreek"), // html only
      _ => None,
    }
  }

  pub fn named(&self, key: &str) -> Option<&str> {
    self.named.get(key).and_then(|s| s.single_text())
  }

  pub fn str_positional_at(&self, index: usize) -> Option<&str> {
    let Some(Some(nodes)) = self.positional.get(index) else {
      return None;
    };
    nodes.single_text()
  }

  pub fn has_option(&self, option: &str) -> bool {
    self.options.iter().any(|s| s.src == option)
  }

  pub fn has_str_positional(&self, positional: &str) -> bool {
    self
      .positional
      .iter()
      .enumerate()
      .any(|(i, _)| self.str_positional_at(i) == Some(positional))
  }

  pub fn is_source(&self) -> bool {
    self.source_language().is_some()
  }

  // TODO: this is incorrect, see https://github.com/jaredh159/asciidork/issues/4
  pub fn source_language(&self) -> Option<&str> {
    match (self.str_positional_at(0), self.str_positional_at(1)) {
      (None | Some("source"), Some(lang)) => Some(lang),
      _ => None,
    }
  }

  pub fn is_empty(&self) -> bool {
    self.positional.is_empty()
      && self.named.0.is_empty()
      && self.id.is_none()
      && self.roles.is_empty()
      && self.options.is_empty()
  }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Named<'bmp>(BumpVec<'bmp, (SourceString<'bmp>, InlineNodes<'bmp>)>);

impl<'bmp> Named<'bmp> {
  pub fn new_in(bump: &'bmp Bump) -> Self {
    Named(BumpVec::new_in(bump))
  }

  pub fn from(vec: BumpVec<'bmp, (SourceString<'bmp>, InlineNodes<'bmp>)>) -> Self {
    Named(vec)
  }

  pub fn insert(&mut self, key: SourceString<'bmp>, value: InlineNodes<'bmp>) {
    self.0.push((key, value));
  }

  pub fn get(&self, key: &str) -> Option<&InlineNodes<'bmp>> {
    self
      .0
      .iter()
      .find_map(|(k, v)| if k == key { Some(v) } else { None })
  }
}

impl Json for Named<'_> {
  fn to_json_in(&self, buf: &mut JsonBuf) {
    buf.push('{');
    for (i, (key, value)) in self.0.iter().enumerate() {
      if i > 0 {
        buf.push(',');
      }
      key.src.to_json_in(buf);
      buf.push(':');
      if let Some(text) = value.single_text() {
        text.to_json_in(buf);
      } else {
        value.to_json_in(buf);
      }
    }
    buf.finish_obj();
  }
}

impl Json for AttrList<'_> {
  fn to_json_in(&self, buf: &mut JsonBuf) {
    buf.begin_obj("AttrList");
    if !self.positional.is_empty() {
      buf.push_str(r#","positional":"#);
      buf.push('[');
      for (i, location) in self.positional.iter().enumerate() {
        if i > 0 {
          buf.push(',');
        }
        match location {
          Some(nodes) => {
            if let Some(text) = nodes.single_text() {
              text.to_json_in(buf);
            } else {
              nodes.to_json_in(buf);
            }
          }
          None => location.to_json_in(buf),
        }
      }
      buf.push(']');
    }
    if !self.named.0.is_empty() {
      buf.add_member("named", &self.named);
    }
    let id = self.id.as_ref().map(|s| &s.src);
    buf.add_option_member("id", id);
    if !self.roles.is_empty() {
      buf.add_member("roles", &self.roles);
    }
    if !self.options.is_empty() {
      buf.add_member("options", &self.options);
    }
    buf.finish_obj();
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::assert_json;
  use test_utils::*;

  #[test]
  fn test_attrlist_to_json() {
    let cases = [
      (
        AttrList {
          positional: vecb![Some(just!("pos1", 0..0)), None],
          named: Named(vecb![(src!("key", 0..0), just!("value", 0..0),)]),
          id: Some(src!("foo", 0..0)),
          roles: vecb![src!("role1", 0..0)],
          options: vecb![src!("option1", 0..0)],
          loc: SourceLocation::new(0, 0),
        },
        r#"{
          "type": "AttrList",
          "positional": ["pos1", null],
          "named": {"key":"value"},
          "id": "foo",
          "roles": ["role1"],
          "options": ["option1"]
        }"#,
      ),
      (
        AttrList {
          positional: vecb![],
          named: Named(vecb![]),
          id: None,
          roles: vecb![],
          options: vecb![],
          loc: SourceLocation::new(0, 0),
        },
        r#"{"type": "AttrList"}"#,
      ),
    ];
    for (input, expected) in cases {
      assert_json!(input, expected);
    }
  }
}
