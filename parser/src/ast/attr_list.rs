use bumpalo::collections::{String, Vec};
use bumpalo::Bump;

// https://docs.asciidoctor.org/asciidoc/latest/attributes/positional-and-named-attributes/
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AttrList<'bmp> {
  pub positional: Vec<'bmp, String<'bmp>>,
  pub named: Named<'bmp>,
  pub id: Option<String<'bmp>>,
  pub roles: Vec<'bmp, String<'bmp>>,
  pub options: Vec<'bmp, String<'bmp>>,
}

impl<'bmp> AttrList<'bmp> {
  pub fn new_in(bump: &'bmp Bump) -> Self {
    AttrList {
      positional: Vec::new_in(bump),
      named: Named::new_in(bump),
      id: None,
      roles: Vec::new_in(bump),
      options: Vec::new_in(bump),
    }
  }

  pub fn role(role: &'static str, bump: &'bmp Bump) -> AttrList<'bmp> {
    let mut attr_list = AttrList::new_in(bump);
    let mut string = String::with_capacity_in(role.len(), bump);
    string.push_str(role);
    attr_list.roles.push(string);
    attr_list
  }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Named<'bmp>(Vec<'bmp, (String<'bmp>, String<'bmp>)>);

impl<'bmp> Named<'bmp> {
  pub fn new_in(bump: &'bmp Bump) -> Self {
    Named(Vec::new_in(bump))
  }

  pub fn from(vec: Vec<'bmp, (String<'bmp>, String<'bmp>)>) -> Self {
    Named(vec)
  }

  pub fn insert(&mut self, key: String<'bmp>, value: String<'bmp>) {
    self.0.push((key, value));
  }

  pub fn get(&self, key: &str) -> Option<&String<'bmp>> {
    self
      .0
      .iter()
      .find_map(|(k, v)| if k == key { Some(v) } else { None })
  }
}
