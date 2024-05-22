use std::collections::HashMap;

use crate::internal::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JobAttr {
  pub readonly: bool,
  pub value: AttrValue,
}

impl JobAttr {
  pub fn modifiable(value: impl Into<AttrValue>) -> Self {
    Self { readonly: false, value: value.into() }
  }

  pub fn readonly(value: impl Into<AttrValue>) -> Self {
    Self { readonly: true, value: value.into() }
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct JobAttrs(HashMap<String, JobAttr>);

impl JobAttrs {
  pub fn empty() -> Self {
    Self(HashMap::new())
  }

  pub fn insert(&mut self, key: impl Into<String>, value: JobAttr) {
    self.0.insert(key.into(), value);
  }

  pub fn get(&self, key: &str) -> Option<&JobAttr> {
    self.0.get(key)
  }
}

impl AsRef<HashMap<String, JobAttr>> for JobAttrs {
  fn as_ref(&self) -> &HashMap<String, JobAttr> {
    &self.0
  }
}
