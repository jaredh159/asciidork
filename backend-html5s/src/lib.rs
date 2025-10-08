#![allow(unused_imports)]

extern crate asciidork_ast as ast;
extern crate asciidork_backend as backend;
extern crate asciidork_eval as eval;

mod html5s;

pub use crate::html5s::Html5s;

mod internal {
  pub use std::borrow::Cow;
  pub use std::mem;

  // pub use lazy_static::lazy_static;
  pub use regex::Regex;

  pub use crate::Html5s;
  pub use asciidork_core::*;
  pub use ast::prelude::*;
  pub use backend::html::backend::*;
  pub use backend::html::{AltHtmlBuf, HtmlBuf, OpenTag};
  pub use backend::prelude::*;
  pub use backend::utils;
  pub use eval::helpers;
}
