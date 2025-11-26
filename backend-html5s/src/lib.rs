use std::error::Error;

extern crate asciidork_ast as ast;
extern crate asciidork_backend as backend;
extern crate asciidork_eval as eval;

mod css;
mod html5s;

pub use crate::html5s::Html5s;

pub fn convert(document: ast::Document) -> Result<String, Box<dyn Error>> {
  Ok(eval::eval(&document, Html5s::new())?)
}

mod internal {
  pub use std::mem;

  pub use asciidork_core::*;
  pub use ast::prelude::*;
  pub use backend::html::backend::*;
  pub use backend::html::{AltHtmlBuf, HtmlBuf, OpenTag};
  pub use backend::prelude::*;
  pub use backend::utils;
}
