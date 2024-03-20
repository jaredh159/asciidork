extern crate asciidork_ast as ast;
extern crate asciidork_backend as backend;
extern crate asciidork_eval as eval;

mod asciidoctor_html;
pub mod section;

pub use asciidoctor_html::AsciidoctorHtml;

mod internal {
  pub use std::borrow::Cow;
  pub use std::convert::Infallible;
  pub use std::mem;

  pub use lazy_static::lazy_static;
  pub use regex::Regex;
  pub use smallvec::SmallVec;

  pub use crate::section;
  pub use ast::prelude::*;
  pub use ast::DocHeader;
  pub use backend::prelude::*;
  pub use eval::helpers;
}
