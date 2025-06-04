#![macro_use]

#[macro_export]
macro_rules! assert_html {
  ($name:ident, $input:expr, $expected:expr) => {
    assert_html!($name, |_| {}, $input, $expected);
  };
  ($name:ident, strict: false, $input:expr, $expected:expr) => {
    assert_html!($name, |s: &mut asciidork_core::JobSettings| {s.strict = false}, $input, $expected);
  };
  ($name:ident, $mod_settings:expr, $input:expr, $expected:expr) => {
    #[test]
    fn $name() {
      let actual = _html!($input, $mod_settings, None);
      ::test_utils::expect_eq!(actual, $expected.to_string(), from: $input);
    }
  };
  ($name:ident, resolving: $bytes:expr, $input:expr, $expected:expr$(,)?) => {
    #[test]
    fn $name() {
      let actual = _html!($input, |_| {}, Some(const_resolver!($bytes)));
      ::test_utils::expect_eq!(actual, $expected.to_string(), from: $input);
    }
  };
  ($name:ident, resolving: $bytes:expr, $mod_settings:expr, $input:expr, $expected:expr$(,)?) => {
    #[test]
    fn $name() {
      let actual = _html!($input, $mod_settings, Some(const_resolver!($bytes)));
      ::test_utils::expect_eq!(actual, $expected.to_string(), from: $input);
    }
  };
  ($name:ident, resolving_err: $err:expr, $input:expr, $expected:expr$(,)?) => {
    #[test]
    fn $name() {
      let actual = _html!($input, |s: &mut JobSettings| {s.strict = false}, Some(error_resolver!($err)));
      ::test_utils::expect_eq!(actual, $expected.to_string(), from: $input);
    }
  };
  ($name:ident, resolving: $bytes:expr, $input:expr, contains: $($expected:expr),+$(,)?) => {
    #[test]
    fn $name() {
      let actual = _html!($input, |_| {}, Some(const_resolver!($bytes)));
      $(assert!(
        actual.contains($expected),
        "\n`{}` was NOT found when expected\n\n\x1b[2m```adoc\x1b[0m\n{}\n\x1b[2m```\x1b[0m\n\n\x1b[2m```html\x1b[0m\n{}\n\x1b[2m```\x1b[0m",
        $expected,
        $input.trim(),
        actual.replace('>', ">\n").trim()
      );)+
    }
  };
  ($name:ident, $input:expr, contains: $($expected:expr),+$(,)?) => {
    assert_html!($name, |_| {}, $input, contains: $($expected),+);
  };
  ($name:ident, strict: false, $input:expr, contains: $($expected:expr),+$(,)?) => {
    assert_html!($name, |s: &mut asciidork_core::JobSettings| {s.strict = false}, $input, contains: $($expected),+);
  };
  ($name:ident, $mod_settings:expr, $input:expr, contains: $($expected:expr),+$(,)?) => {
    #[test]
    fn $name() {
      let actual = _html!($input, $mod_settings, None);
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
      |settings: &mut ::asciidork_core::JobSettings| {
        settings.embedded = true;
        settings.doctype = Some(::asciidork_core::DocType::Inline);
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
      let mut settings = ::asciidork_core::JobSettings::default();
      #[allow(clippy::redundant_closure_call)]
      $mod_settings(&mut settings);
      let mut parser = ::test_utils::test_parser!($input);
      parser.apply_job_settings(settings);
      let document = parser.parse().unwrap().document;
      let actual = ::asciidork_eval::eval(
        &document,
        ::asciidork_dr_html_backend::AsciidoctorHtml::new()).unwrap();
      let mut body = actual.splitn(2, "<body").nth(1).unwrap();
      body = body.splitn(2, "</body>").nth(0).unwrap();
      let body = format!("<body{}</body>", body);
      ::test_utils::expect_eq!(body, $expected.to_string(), from: $input);
    }
  };
}

#[macro_export]
macro_rules! test_non_embedded_contains {
  ($name:ident, $input:expr, $needles:expr$(,)?) => {
    #[test]
    fn $name() {
      let bump = &::asciidork_parser::prelude::Bump::new();
      let parser = ::asciidork_parser::Parser::from_str(
        $input,
        ::asciidork_parser::prelude::SourceFile::Tmp,
        bump
      );
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

macro_rules! _html {
  ($input:expr, $mod_settings:expr, $resolver:expr) => {{
    let bump = &::asciidork_parser::prelude::Bump::new();
    let mut settings = ::asciidork_core::JobSettings::embedded();
    settings.safe_mode = ::asciidork_core::SafeMode::Unsafe;
    #[allow(clippy::redundant_closure_call)]
    $mod_settings(&mut settings);
    let path = ::asciidork_core::Path::new("test.adoc");
    let mut parser = ::asciidork_parser::Parser::from_str(
      $input,
      ::asciidork_parser::prelude::SourceFile::Path(path),
      bump,
    );
    parser.apply_job_settings(settings);
    if let Some(resolver) = $resolver {
      parser.set_resolver(resolver);
    }
    let document = parser.parse().unwrap().document;
    ::asciidork_eval::eval(
      &document,
      ::asciidork_dr_html_backend::AsciidoctorHtml::new(),
    )
    .unwrap()
  }};
}

pub mod source {
  pub fn wrap_listing(inner: &str) -> String {
    format!(
      r#"<div class="listingblock"><div class="content">{}</div></div>"#,
      inner.trim(),
    )
  }

  pub fn wrap_literal(inner: &str) -> String {
    format!(
      r#"<div class="literalblock"><div class="content">{}</div></div>"#,
      inner.trim(),
    )
  }

  pub fn wrap(lang: &str, inner: &str) -> String {
    wrap_listing(&format!(
      r#"<pre class="highlight"><code class="language-{lang}" data-lang="{lang}">{}</code></pre>"#,
      inner.trim(),
    ))
  }
}
