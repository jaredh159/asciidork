use std::error::Error;

extern crate asciidork_ast as ast;
extern crate asciidork_backend as backend;
extern crate asciidork_eval as eval;

mod asciidoctor_html;
pub mod css;
pub mod section;

pub use asciidoctor_html::AsciidoctorHtml;
pub use backend::Backend;

pub fn convert(document: ast::Document) -> Result<String, Box<dyn Error>> {
  Ok(eval::eval(&document, AsciidoctorHtml::new())?)
}

mod internal {
  pub use std::borrow::Cow;
  pub use std::convert::Infallible;
  pub use std::mem;

  pub use lazy_static::lazy_static;
  pub use regex::Regex;

  pub use crate::section;
  pub use asciidork_core::*;
  pub use ast::prelude::*;
  pub use backend::html::htmlbuf::*;
  pub use backend::html::*;
  pub use backend::prelude::*;
  pub use backend::utils;
  pub use eval::helpers;
}
