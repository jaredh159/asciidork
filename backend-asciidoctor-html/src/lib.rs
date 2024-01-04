extern crate asciidork_ast as ast;
extern crate asciidork_backend as backend;

mod asciidoctor_html;

pub use asciidoctor_html::AsciidoctorHtml;

mod internal {
  pub use std::borrow::Cow;
  pub use std::convert::Infallible;

  pub use lazy_static::lazy_static;
  pub use regex::Regex;

  pub use ast::prelude::*;
  pub use ast::short::block::*;
  pub use ast::variants::inline::*;
  pub use backend::prelude::*;
}
