#[macro_export]
macro_rules! test_eval_inline {
  ($name:ident, $input:expr, $expected:expr) => {
    #[test]
    fn $name() {
      let bump = &::asciidork_parser::prelude::Bump::new();
      let mut settings = ::asciidork_meta::JobSettings::embedded();
      settings.doctype = Some(::asciidork_meta::DocType::Inline);
      let parser = ::asciidork_parser::Parser::new_settings(bump, $input, settings);
      let document = parser.parse().unwrap().document;
      let actual = ::asciidork_eval::eval(
        &document,
        ::asciidork_dr_html_backend::AsciidoctorHtml::new()).unwrap();
      ::test_utils::assert_eq!(actual, $expected.to_string(), from: $input);
    }
  };
}

#[macro_export]
macro_rules! test_eval {
  ($name:ident, $input:expr, $expected:expr) => {
    test_eval!($name, |_| {}, $input, $expected);
  };
  ($name:ident, $mod_settings:expr, $input:expr, $expected:expr) => {
    #[test]
    fn $name() {
      let bump = &::asciidork_parser::prelude::Bump::new();
      let mut settings = ::asciidork_meta::JobSettings::embedded();
      #[allow(clippy::redundant_closure_call)]
      $mod_settings(&mut settings.job_attrs);
      let parser = ::asciidork_parser::Parser::new_settings(bump, $input, settings);
      let document = parser.parse().unwrap().document;
      let actual = ::asciidork_eval::eval(
        &document,
        ::asciidork_dr_html_backend::AsciidoctorHtml::new()).unwrap();
      ::test_utils::assert_eq!(actual, $expected.to_string(), from: $input);
    }
  };
  ($name:ident, $input:expr, html_contains: $expected:expr) => {
    test_eval!($name, |_| {}, $input, html_contains: $expected);
  };
  ($name:ident, $mod_settings:expr, $input:expr, html_contains: $expected:expr) => {
    #[test]
    fn $name() {
      let bump = &::asciidork_parser::prelude::Bump::new();
      let mut settings = ::asciidork_meta::JobSettings::embedded();
      #[allow(clippy::redundant_closure_call)]
      $mod_settings(&mut settings.job_attrs);
      let parser = ::asciidork_parser::Parser::new_settings(bump, $input, settings);
      let document = parser.parse().unwrap().document;
      let actual = ::asciidork_eval::eval(
        &document,
        ::asciidork_dr_html_backend::AsciidoctorHtml::new()).unwrap();
      assert!(
        actual.contains($expected),
        "\n`{}` was NOT found when expected\n\n```adoc\n{}\n```\n\n```html\n{}\n```",
        $expected,
        $input.trim(),
        actual.replace('>', ">\n").trim()
      )
    }
  };
}

#[macro_export]
macro_rules! test_eval_loose {
  ($name:ident, $input:expr, $expected:expr) => {
    #[test]
    fn $name() {
      let mut settings = ::asciidork_meta::JobSettings::default();
      settings.strict = false;
      let bump = &::asciidork_parser::prelude::Bump::new();
      let parser = ::asciidork_parser::Parser::new_settings(bump, $input, settings);
      let document = parser.parse().unwrap().document;
      let actual = ::asciidork_eval::eval(
        &document,
        ::asciidork_dr_html_backend::AsciidoctorHtml::new()).unwrap();
      let mut body = actual.splitn(2, "<body class=\"article\">").nth(1).unwrap();
      body = body.splitn(2, "</body>").nth(0).unwrap();
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
      let parser = ::asciidork_parser::Parser::new(bump, $input);
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
