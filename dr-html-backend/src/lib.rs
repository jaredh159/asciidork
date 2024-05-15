use std::error::Error;

extern crate asciidork_ast as ast;
extern crate asciidork_backend as backend;
extern crate asciidork_eval as eval;

mod asciidoctor_html;
pub mod section;
mod table;

pub use asciidoctor_html::AsciidoctorHtml;

pub fn convert(document: ast::Document, opts: eval::Opts) -> Result<String, Box<dyn Error>> {
  Ok(eval::eval(&document, opts, AsciidoctorHtml::new())?)
}

pub fn convert_embedded_article(document: ast::Document) -> Result<String, Box<dyn Error>> {
  convert(document, eval::Opts::embedded())
}

mod internal {
  pub use std::borrow::Cow;
  pub use std::convert::Infallible;
  pub use std::mem;

  pub use lazy_static::lazy_static;
  pub use regex::Regex;
  pub use smallvec::SmallVec;

  pub use crate::section;
  pub use crate::AsciidoctorHtml;
  pub use ast::prelude::*;
  pub use ast::DocHeader;
  pub use backend::prelude::*;
  pub use eval::helpers;
}
