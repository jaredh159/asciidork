use crate::helpers::*;

#[test]
fn test_cli_docinfo_dr_html() {
  let stdout = run_file(
    &["--safe-mode", "unsafe"],
    "tests/all/fixtures/docinfo/basic.adoc",
  );
  assert!(stdout.contains(r#"<meta name="robots" content="index,follow">"#));
  assert!(stdout.contains(r#"<script src="modernizr.js"></script>"#));
  assert!(stdout.contains(r#"<nav class="navbar">Docs</nav>"#));
  assert!(stdout.contains(r##"<a id="top" href="#">Back to top</a>"##));
  assert!(stdout.contains(r#"<script src="plusone.js"></script>"#));
}

#[test]
fn test_cli_docinfo_html5() {
  let stdout = run_file(
    &["--safe-mode", "unsafe", "--format", "html5"],
    "tests/all/fixtures/docinfo/basic.adoc",
  );
  assert!(stdout.contains(r#"<meta name="robots" content="index,follow">"#));
  assert!(stdout.contains(r#"<body class="article"><nav class="navbar">Docs</nav>"#));
  assert!(stdout.contains("<header>"));
  assert!(stdout.contains(r##"<a id="top" href="#">Back to top</a>"##));
}

#[test]
fn test_cli_docinfo_custom_dir_and_subs() {
  let stdout = run_file(
    &["--safe-mode", "unsafe"],
    "tests/all/fixtures/docinfo/subs.adoc",
  );
  assert!(stdout.contains(r#"<meta name="copyright" content="&#169; OpenDevise">"#));
  assert!(stdout.contains(r#"<script src="bootstrap.3.2.0.js"></script>"#));
  assert!(stdout.contains(r##"<a id="top" href="#">Back to top</a>"##));
}

#[test]
fn test_cli_docinfo_secure_mode_ignores_files() {
  let stdout = run_file(
    &["--safe-mode", "secure"],
    "tests/all/fixtures/docinfo/basic.adoc",
  );
  assert!(!stdout.contains("modernizr.js"));
  assert!(!stdout.contains(r#"name="robots""#));
  assert!(!stdout.contains(r#"<nav class="navbar">Docs</nav>"#));
}

#[test]
fn test_cli_docinfo_embedded_ignores_files() {
  let stdout = run_file(
    &["--embedded", "--safe-mode", "unsafe"],
    "tests/all/fixtures/docinfo/basic.adoc",
  );
  assert!(!stdout.contains("modernizr.js"));
  assert!(!stdout.contains(r#"name="robots""#));
  assert!(!stdout.contains(r#"<nav class="navbar">Docs</nav>"#));
  assert!(!stdout.contains("plusone.js"));
  assert!(!stdout.contains("Back to top"));
}
