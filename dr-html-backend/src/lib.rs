use std::error::Error;

extern crate asciidork_ast as ast;
extern crate asciidork_backend as backend;
extern crate asciidork_eval as eval;
extern crate asciidork_meta as meta;

mod asciidoctor_html;
mod htmlbuf;
mod open_tag;
pub mod section;
mod table;

pub use asciidoctor_html::AsciidoctorHtml;

pub fn convert(document: ast::Document) -> Result<String, Box<dyn Error>> {
  Ok(eval::eval(&document, AsciidoctorHtml::new())?)
}

mod internal {
  pub use std::borrow::Cow;
  pub use std::convert::Infallible;
  pub use std::mem;

  pub use lazy_static::lazy_static;
  pub use regex::Regex;

  pub use crate::htmlbuf::*;
  pub use crate::open_tag::*;
  pub use crate::section;
  pub use crate::AsciidoctorHtml;
  pub use ast::prelude::*;
  pub use backend::prelude::*;
  pub use eval::helpers;
  pub use meta::*;
}

mod str_util {
  /// NB: does not return the `.`
  pub fn file_ext(input: &str) -> Option<&str> {
    if let Some(idx) = input.rfind('.') {
      Some(&input[idx + 1..])
    } else {
      None
    }
  }
  pub fn basename(input: &str) -> &str {
    input.split(&['/', '\\']).last().unwrap_or(input)
  }
  pub fn filestem(input: &str) -> &str {
    basename(input).split('.').next().unwrap_or(input)
  }
  pub fn remove_uri_scheme(input: &str) -> &str {
    let mut split = input.splitn(2, "://");
    let first = split.next().unwrap_or("");
    let Some(rest) = split.next() else {
      return input;
    };
    if rest.is_empty() {
      input
    } else if matches!(first, "http" | "https" | "ftp" | "mailto" | "irc" | "file") {
      rest
    } else {
      input
    }
  }
}
