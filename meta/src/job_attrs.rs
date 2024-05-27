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

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct JobAttrs(HashMap<String, JobAttr>);

impl JobAttrs {
  pub fn empty() -> Self {
    Self(HashMap::new())
  }

  pub fn insert(&mut self, key: impl Into<String>, job_attr: JobAttr) -> Result<(), String> {
    let key: String = key.into();
    validate::attr(&key, &job_attr.value)?;
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
}

impl AsRef<HashMap<String, JobAttr>> for JobAttrs {
  fn as_ref(&self) -> &HashMap<String, JobAttr> {
    &self.0
  }
}
