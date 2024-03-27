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
}
