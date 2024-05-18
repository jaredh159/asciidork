use std::collections::HashMap;

use crate::internal::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskAttr {
  pub readonly: bool,
  pub value: AttrValue,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TaskAttrs(HashMap<String, TaskAttr>);

impl TaskAttrs {
  pub fn insert(&mut self, key: impl Into<String>, value: TaskAttr) {
    self.0.insert(key.into(), value);
  }

  pub fn get(&self, key: &str) -> Option<&TaskAttr> {
    self.0.get(key)
  }
}
