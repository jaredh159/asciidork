mod utils;

use asciidork_dr_html_backend::AsciidoctorHtml;
use asciidork_eval::{eval, Opts};
use asciidork_parser::prelude::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn convert(adoc: &str) -> String {
  let bump = &Bump::new();
  let parser = Parser::new(bump, adoc);
  let document = parser.parse().unwrap().document;
  eval(document, Opts::embedded(), AsciidoctorHtml::new()).unwrap()
}
