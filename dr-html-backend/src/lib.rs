use std::error::Error;

extern crate asciidork_ast as ast;
extern crate asciidork_backend as backend;
extern crate asciidork_eval as eval;

mod asciidoctor_html;
mod htmlbuf;
mod open_tag;
pub mod section;
mod table;

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

  pub use crate::asciidoctor_html::EphemeralState;
  pub use crate::htmlbuf::*;
  pub use crate::open_tag::*;
  pub use crate::section;
  pub use crate::AsciidoctorHtml;
  pub use asciidork_core::*;
  pub use ast::prelude::*;
  pub use backend::prelude::*;
  pub use backend::utils;
  pub use eval::helpers;
}
