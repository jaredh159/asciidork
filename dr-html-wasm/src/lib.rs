mod utils;

use asciidork_dr_html_backend as backend;
use asciidork_meta::JobSettings;
use asciidork_parser::{include_resolver::IncludeResolver, parser::ParseResult, prelude::*};
use wasm_bindgen::prelude::*;

fn get_include_from_js(_target: &str) -> Option<String> {
  Some(String::from("Hello, includes!"))
}

struct RoflResolver;

impl IncludeResolver for RoflResolver {
  fn resolve(
    &mut self,
    path: &str,
    buffer: &mut dyn asciidork_parser::include_resolver::IncludeBuffer,
  ) -> std::result::Result<usize, asciidork_parser::include_resolver::ResolveError> {
    let include = get_include_from_js(path).unwrap();
    buffer.initialize(include.len());
    buffer.as_bytes_mut().copy_from_slice(include.as_bytes());
    Ok(include.len())
  }
}

#[wasm_bindgen]
pub fn convert(adoc: &str) -> String {
  console_error_panic_hook::set_once();

  let bump = &Bump::new();
  let mut parser = Parser::from_str(adoc, bump);
  parser.apply_job_settings(JobSettings::embedded());
  parser.set_resolver(Box::new(RoflResolver));
  let result = parser.parse();
  match result {
    Ok(ParseResult { document, .. }) => {
      let html = backend::convert(document).unwrap();
      format!(
        r#"{{"success":true,"html":"{}"}}"#,
        html.replace('"', "\\\"").replace('\n', "\\n")
      )
    }
    Err(diagnostics) => format!(
      r#"{{"success":false,"errors":["{}"]}}"#,
      diagnostics
        .iter()
        .map(Diagnostic::plain_text)
        .collect::<Vec<_>>()
        .join(r#"",""#)
        .replace('"', "\\\"")
        .replace('\n', "\\n")
    ),
  }
}
