use std::collections::HashMap;

// https://docs.asciidoctor.org/asciidoc/latest/attributes/positional-and-named-attributes/
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AttrList {
  pub positional: Vec<String>,
  pub named: HashMap<String, String>,
  pub id: Option<String>,
  pub roles: Vec<String>,
  pub options: Vec<String>,
}

impl AttrList {
  pub fn new() -> AttrList {
    AttrList {
      positional: vec![],
      named: HashMap::new(),
      id: None,
      roles: vec![],
      options: vec![],
    }
  }

  pub fn role(role: &'static str) -> AttrList {
    AttrList {
      roles: vec![role.to_string()],
      ..AttrList::default()
    }
  }
}

impl Default for AttrList {
  fn default() -> Self {
    Self::new()
  }
}
