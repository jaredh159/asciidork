use std::collections::HashMap;

use asciidork_ast::prelude::*;
use asciidork_core::{JobAttr, JobSettings};
use asciidork_parser::includes::*;
use asciidork_parser::prelude::*;
use test_utils::*;

#[derive(Clone, Default)]
struct MapResolver(HashMap<String, Vec<u8>>);

impl IncludeResolver for MapResolver {
  fn resolve(
    &mut self,
    target: IncludeTarget,
    buffer: &mut dyn IncludeBuffer,
    _: asciidork_core::SafeMode,
  ) -> std::result::Result<usize, ResolveError> {
    let Some(bytes) = self.0.get(&target.path().to_string()) else {
      return Err(ResolveError::NotFound);
    };
    buffer.initialize(bytes.len());
    buffer.as_bytes_mut().copy_from_slice(bytes);
    Ok(bytes.len())
  }

  fn clone_box(&self) -> Box<dyn IncludeResolver> {
    Box::new(self.clone())
  }
}

#[test]
fn parses_docinfo_as_semantic_fragment() {
  let mut parser = test_parser!(adoc! {r#"
    = Doc
    :docinfo1:
    :docinfosubs: attributes,replacements
    :copyright-owner: OpenDevise
    :bootstrap-version: 3.2.0
  "#});
  parser.apply_job_settings(JobSettings::r#unsafe());
  parser.set_resolver(Box::new(MapResolver(HashMap::from([(
    "docinfo.html".into(),
    br#"<meta name="copyright" content="(C) {copyright-owner}">
<script src="bootstrap.{bootstrap-version}.js"></script>
"#
    .to_vec(),
  )]))));
  let document = parser.parse().unwrap().document;
  expect_eq!(
    document.docinfo.head,
    Some(nodes![
      node!(
        Inline::Text(bstr!(r#"<meta name="copyright" content=""#)),
        0..0
      ),
      node!(Inline::Symbol(SymbolKind::Copyright), 0..0),
      node!(Inline::Text(bstr!(" OpenDevise\">")), 0..0),
      node!(
        Inline::Text(bstr!("<script src=\"bootstrap.3.2.0.js\"></script>")),
        0..0
      ),
    ])
  );
}

#[test]
fn missing_attr_can_drop_docinfo_line() {
  let mut parser = test_parser!(adoc! {r#"
    = Doc
    :docinfo1:
    :attribute-missing: drop-line
  "#});
  parser.apply_job_settings(JobSettings::r#unsafe());
  parser.set_resolver(Box::new(MapResolver(HashMap::from([(
    "docinfo.html".into(),
    br#"<script src="{missing}.js"></script>
<meta name="robots" content="index,follow">
"#
    .to_vec(),
  )]))));
  let document = parser.parse().unwrap().document;
  expect_eq!(
    document.docinfo.head,
    Some(nodes![node!(
      Inline::Text(bstr!("<meta name=\"robots\" content=\"index,follow\">")),
      0..0
    )])
  );
}

#[test]
fn unsupported_docinfosubs_are_ignored() {
  let mut parser = test_parser!(adoc! {r#"
    = Doc
    :docinfo1:
    :docinfosubs: attributes,macros
    :site-url: https://example.com
  "#});
  parser.apply_job_settings(JobSettings::r#unsafe());
  parser.set_resolver(Box::new(MapResolver(HashMap::from([(
    "docinfo.html".into(),
    br#"link:{site-url}[Docs]
"#
    .to_vec(),
  )]))));
  let document = parser.parse().unwrap().document;
  expect_eq!(
    document.docinfo.head,
    Some(nodes![node!(
      Inline::Text(bstr!("link:https://example.com[Docs]")),
      0..0
    )])
  );
}

#[test]
fn docinfo_uses_outfilesuffix_for_lookup() {
  let mut parser = test_parser!(adoc! {r#"
    = Doc
    :docinfo1:
  "#});
  let mut settings = JobSettings::r#unsafe();
  settings
    .job_attrs
    .insert_unchecked("outfilesuffix", JobAttr::readonly(".xml"));
  parser.apply_job_settings(settings);
  parser.set_resolver(Box::new(MapResolver(HashMap::from([(
    "docinfo.xml".into(),
    br#"<meta name="robots" content="index,follow">
"#
    .to_vec(),
  )]))));
  let document = parser.parse().unwrap().document;
  expect_eq!(
    document.docinfo.head,
    Some(nodes![node!(
      Inline::Text(bstr!("<meta name=\"robots\" content=\"index,follow\">")),
      0..0
    )])
  );
}
