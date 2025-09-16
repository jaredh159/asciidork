pub use asciidork_backend::test::html::*;

pub fn test_backend_factory() -> asciidork_dr_html_backend::AsciidoctorHtml {
  asciidork_dr_html_backend::AsciidoctorHtml::new()
}
