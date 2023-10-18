use bumpalo::collections::{String, Vec};
use bumpalo::Bump;

// https://docs.asciidoctor.org/asciidoc/latest/attributes/positional-and-named-attributes/
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AttrList<'alloc> {
  pub positional: Vec<'alloc, String<'alloc>>,
  pub named: Named<'alloc>,
  pub id: Option<String<'alloc>>,
  pub roles: Vec<'alloc, String<'alloc>>,
  pub options: Vec<'alloc, String<'alloc>>,
}

impl<'alloc> AttrList<'alloc> {
  pub fn new_in(allocator: &'alloc Bump) -> Self {
    AttrList {
      positional: Vec::new_in(allocator),
      named: Named::new_in(allocator),
      id: None,
      roles: Vec::new_in(allocator),
      options: Vec::new_in(allocator),
    }
  }

  pub fn role(role: &'static str, allocator: &'alloc Bump) -> AttrList<'alloc> {
    let mut attr_list = AttrList::new_in(allocator);
    let mut string = String::with_capacity_in(role.len(), allocator);
    string.push_str(role);
    attr_list.roles.push(string);
    attr_list
  }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Named<'alloc>(Vec<'alloc, (String<'alloc>, String<'alloc>)>);

impl<'alloc> Named<'alloc> {
  pub fn new_in(allocator: &'alloc Bump) -> Self {
    Named(Vec::new_in(allocator))
  }

  pub fn from(vec: Vec<'alloc, (String<'alloc>, String<'alloc>)>) -> Self {
    Named(vec)
  }

  pub fn insert(&mut self, key: String<'alloc>, value: String<'alloc>) {
    self.0.push((key, value));
  }

  pub fn get(&self, key: &str) -> Option<&String<'alloc>> {
    self
      .0
      .iter()
      .find_map(|(k, v)| if k == key { Some(v) } else { None })
  }
}
