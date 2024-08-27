use std::collections::HashMap;

use crate::internal::*;
use crate::validate;

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

#[derive(Clone, PartialEq, Eq, Default)]
pub struct JobAttrs(HashMap<String, JobAttr>);

impl JobAttrs {
  pub fn empty() -> Self {
    Self(HashMap::new())
  }

  pub fn insert(&mut self, key: impl Into<String>, job_attr: JobAttr) -> Result<(), String> {
    let key: String = key.into();
    validate::attr(self, &key, &job_attr.value)?;
    self.0.insert(key, job_attr);
    Ok(())
  }

  pub fn insert_unchecked(&mut self, key: impl Into<String>, job_attr: JobAttr) {
    let key: String = key.into();
    self.0.insert(key, job_attr);
  }

  pub fn get(&self, key: &str) -> Option<&JobAttr> {
    self.0.get(key)
  }

  pub fn remove(&mut self, key: &str) {
    self.0.remove(key);
  }
}

impl std::fmt::Debug for JobAttrs {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "JobAttrs(<num attrs: {}>)", self.0.len())
  }
}

impl RemoveAttr for JobAttrs {
  fn remove(&mut self, key: &str) {
    self.0.remove(key);
  }
}

impl AsRef<HashMap<String, JobAttr>> for JobAttrs {
  fn as_ref(&self) -> &HashMap<String, JobAttr> {
    &self.0
  }
}
