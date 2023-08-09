use std::collections::HashMap;

use super::Inline;

// https://docs.asciidoctor.org/asciidoc/latest/document/header/
#[derive(Debug, PartialEq, Eq)]
pub struct DocHeader {
  pub title: Option<DocTitle>,
  pub authors: Vec<Author>,
  pub revision: Option<Revision>,
  pub attrs: HashMap<String, String>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct DocTitle {
  pub heading: Vec<Inline>,
  pub subtitle: Option<Vec<Inline>>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Revision {
  pub version: String,
  pub date: Option<String>,
  pub remark: Option<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Author {
  pub first_name: String,
  pub middle_name: Option<String>,
  pub last_name: String,
  pub email: Option<String>,
}
