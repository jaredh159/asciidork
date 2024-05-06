#[macro_export]
macro_rules! test_eval {
  ($name:ident, $input:expr, $expected:expr) => {
    #[test]
    fn $name() {
      let bump = &::asciidork_parser::prelude::Bump::new();
      let parser = ::asciidork_parser::Parser::new(bump, $input);
      let document = parser.parse().unwrap().document;
      let actual = ::asciidork_eval::eval(
        document,
        ::asciidork_eval::Opts::embedded(),
        ::asciidork_dr_html_backend::AsciidoctorHtml::new()).unwrap();
      ::test_utils::assert_eq!(actual, $expected.to_string(), from: $input);
    }
  };
  ($name:ident, $input:expr, html_contains: $expected:expr) => {
    #[test]
    fn $name() {
      let bump = &::asciidork_parser::prelude::Bump::new();
      let parser = ::asciidork_parser::Parser::new(bump, $input);
      let document = parser.parse().unwrap().document;
      let actual = ::asciidork_eval::eval(
        document,
        ::asciidork_eval::Opts::embedded(),
        ::asciidork_dr_html_backend::AsciidoctorHtml::new()).unwrap();
      // ::test_utils::assert_eq!(actual, $expected.to_string(), from: $input);
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
      let mut opts = ::asciidork_eval::Opts::embedded();
      opts.strict = false;
      let bump = &::asciidork_parser::prelude::Bump::new();
      let parser = ::asciidork_parser::Parser::new_opts(bump, $input, opts);
      let document = parser.parse().unwrap().document;
      let actual = ::asciidork_eval::eval(
        document,
        opts,
        ::asciidork_dr_html_backend::AsciidoctorHtml::new()).unwrap();
      ::test_utils::assert_eq!(actual, $expected.to_string(), from: $input);
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
        document,
        ::asciidork_eval::Opts::default(),
        ::asciidork_dr_html_backend::AsciidoctorHtml::new()).unwrap();
      for needle in &$needles {
        ::test_utils::assert_html_contains!(actual, needle.to_string(), from: $input);
      }
    }
  };
}
