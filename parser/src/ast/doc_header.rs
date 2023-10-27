use std::collections::HashMap;

use bumpalo::collections::{String, Vec};

use super::Inline;

// https://docs.asciidoctor.org/asciidoc/latest/document/header/
#[derive(Debug, PartialEq, Eq)]
pub struct DocHeader<'alloc> {
  pub title: Option<DocTitle<'alloc>>,
  pub authors: Vec<'alloc, Author<'alloc>>,
  pub revision: Option<Revision<'alloc>>,
  pub attrs: HashMap<String<'alloc>, String<'alloc>>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct DocTitle<'alloc> {
  pub heading: Vec<'alloc, Inline<'alloc>>,
  pub subtitle: Option<Vec<'alloc, Inline<'alloc>>>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Revision<'alloc> {
  pub version: String<'alloc>,
  pub date: Option<String<'alloc>>,
  pub remark: Option<String<'alloc>>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Author<'alloc> {
  pub first_name: String<'alloc>,
  pub middle_name: Option<String<'alloc>>,
  pub last_name: String<'alloc>,
  pub email: Option<String<'alloc>>,
}
