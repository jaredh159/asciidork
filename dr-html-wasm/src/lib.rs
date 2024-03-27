mod utils;

use asciidork_parser::prelude::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn convert(adoc: &str) -> String {
  let bump = &Bump::new();
  let parser = Parser::new(bump, adoc);
  let doc = parser.parse().unwrap().document;
  asciidork_dr_html_backend::convert_embedded_article(doc).unwrap()
}
