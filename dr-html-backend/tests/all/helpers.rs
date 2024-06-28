#![macro_use]

#[macro_export]
macro_rules! assert_html {
  ($name:ident, $input:expr, $expected:expr) => {
    assert_html!($name, |_| {}, $input, $expected);
  };
  ($name:ident, $mod_settings:expr, $input:expr, $expected:expr) => {
    #[test]
    fn $name() {
      let bump = &::asciidork_parser::prelude::Bump::new();
      let mut settings = ::asciidork_meta::JobSettings::embedded();
      #[allow(clippy::redundant_closure_call)]
      $mod_settings(&mut settings);
      let mut parser = ::asciidork_parser::Parser::from_str($input, bump);
      parser.apply_job_settings(settings);
      let document = parser.parse().unwrap().document;
      let actual = ::asciidork_eval::eval(
        &document,
        ::asciidork_dr_html_backend::AsciidoctorHtml::new()).unwrap();
      ::test_utils::assert_eq!(actual, $expected.to_string(), from: $input);
    }
  };
  ($name:ident, $input:expr, contains: $($expected:expr),+$(,)?) => {
    assert_html!($name, |_| {}, $input, contains: $($expected),+);
  };
  ($name:ident, $mod_settings:expr, $input:expr, contains: $($expected:expr),+$(,)?) => {
    #[test]
    fn $name() {
      let bump = &::asciidork_parser::prelude::Bump::new();
      let mut settings = ::asciidork_meta::JobSettings::embedded();
      #[allow(clippy::redundant_closure_call)]
      $mod_settings(&mut settings);
      let mut parser = ::asciidork_parser::Parser::from_str($input, bump);
      parser.apply_job_settings(settings);
      let document = parser.parse().unwrap().document;
      let actual = ::asciidork_eval::eval(
        &document,
        ::asciidork_dr_html_backend::AsciidoctorHtml::new()).unwrap();
      $(assert!(
        actual.contains($expected),
        "\n`{}` was NOT found when expected\n\n\x1b[2m```adoc\x1b[0m\n{}\n\x1b[2m```\x1b[0m\n\n\x1b[2m```html\x1b[0m\n{}\n\x1b[2m```\x1b[0m",
        $expected,
        $input.trim(),
        actual.replace('>', ">\n").trim()
      );)+
    }
  };
}

#[macro_export]
macro_rules! assert_inline_html {
  ($name:ident, $input:expr, $expected:expr) => {
    assert_html!(
      $name,
      |settings: &mut ::asciidork_meta::JobSettings| {
        settings.embedded = true;
        settings.doctype = Some(::asciidork_meta::DocType::Inline);
      },
      $input,
      $expected
    );
  };
}

#[macro_export]
macro_rules! assert_standalone_body {
  ($name:ident, $input:expr, $expected:expr) => {
    assert_standalone_body!($name, |_| {}, $input, $expected);
  };
  ($name:ident, $mod_settings:expr, $input:expr, $expected:expr) => {
    #[test]
    fn $name() {
      let bump = &::asciidork_parser::prelude::Bump::new();
      let mut settings = ::asciidork_meta::JobSettings::default();
      #[allow(clippy::redundant_closure_call)]
      $mod_settings(&mut settings);
      let mut parser = ::asciidork_parser::Parser::from_str($input, bump);
      parser.apply_job_settings(settings);
      let document = parser.parse().unwrap().document;
      let actual = ::asciidork_eval::eval(
        &document,
        ::asciidork_dr_html_backend::AsciidoctorHtml::new()).unwrap();
      let mut body = actual.splitn(2, "<body").nth(1).unwrap();
      body = body.splitn(2, "</body>").nth(0).unwrap();
      let body = format!("<body{}</body>", body);
      ::test_utils::assert_eq!(body, $expected.to_string(), from: $input);
    }
  };
}

#[macro_export]
macro_rules! test_non_embedded_contains {
  ($name:ident, $input:expr, $needles:expr$(,)?) => {
    #[test]
    fn $name() {
      let bump = &::asciidork_parser::prelude::Bump::new();
      let parser = ::asciidork_parser::Parser::from_str($input, bump);
      let document = parser.parse().unwrap().document;
      let actual = ::asciidork_eval::eval(
        &document,
        ::asciidork_dr_html_backend::AsciidoctorHtml::new()).unwrap();
      for needle in &$needles {
        ::test_utils::assert_html_contains!(actual, needle.to_string(), from: $input);
      }
    }
  };
}
