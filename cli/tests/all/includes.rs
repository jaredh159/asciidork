use crate::helpers::*;
use test_utils::*;

#[cfg(unix)]
#[test]
fn test_cli_app_single_include() {
  let stdout = run_file(
    &["--embedded", "--strict", "--safe-mode", "unsafe"],
    "tests/all/fixtures/gen/a.adoc",
  );
  expect_eq!(
    stdout.trim(),
    html! {r#"
      <div class="paragraph">
        <p>docdir: {cwd}/tests/all/fixtures/gen</p>
      </div>
      <div class="paragraph">
        <p>f: <em>fixtures/gen/a.adoc</em></p>
      </div>
      <div class="paragraph">
        <p>docdir: {cwd}/tests/all/fixtures/gen</p>
      </div>
      <div class="paragraph">
        <p>f: <em>fixtures/gen/b.adoc</em></p>
      </div>
    "#}
    .replace("{cwd}", &cwd())
  );
}

#[test]
fn test_cli_include_case_fail_strict() {
  let stderr = run_expecting_err(
    &["--embedded", "--strict", "--safe-mode", "unsafe"],
    "tests/all/fixtures/gen/case-fail.adoc",
  );

  #[cfg(any(target_os = "windows", target_os = "macos"))]
  expect_eq!(
    stderr.trim(),
    adoc! {r#"
      --> case-fail.adoc:1:10
        |
      1 | include::sub/inNER.adoc[]
        |          ^^^^^^^^^^^^^^ Include error: Case mismatch in file path. Maybe you meant to include `inner.adoc`?

      Error: "Parse error""#}
  );

  #[cfg(target_os = "linux")]
  expect_eq!(
    stderr.trim(),
    adoc! {r#"
      --> case-fail.adoc:1:10
        |
      1 | include::sub/inNER.adoc[]
        |          ^^^^^^^^^^^^^^ Include error: File not found

      Error: "Parse error""#}
  );
}

#[cfg(unix)]
#[test]
fn test_relative_includes() {
  let stdout = run_file(
    &["--embedded", "--strict", "--safe-mode", "unsafe"],
    "tests/all/fixtures/gen/parent-include.adoc",
  );
  expect_eq!(
    stdout.trim(),
    html! {r#"
      <div class="paragraph"><p>first line of parent</p></div>
      <div class="paragraph"><p>first line of child</p></div>
      <div class="paragraph"><p>first line of grandchild</p></div>
      <div class="paragraph"><p>last line of grandchild</p></div>
      <div class="paragraph"><p>last line of child</p></div>
      <div class="paragraph"><p>last line of parent</p></div>
    "#}
  );
}

// run on linux (CI) only for speed in local dev
#[cfg(target_os = "linux")]
#[test]
fn test_remote_relative_includes() {
  let stdout = run_file(
    &[
      "--embedded",
      "--strict",
      "--safe-mode",
      "unsafe",
      "--attribute",
      "allow-uri-read",
    ],
    "tests/all/fixtures/remote-rel.adoc",
  );
  expect_eq!(
    stdout.trim(),
    html! {r#"
      <div class="paragraph"><p>first line of parent</p></div>
      <div class="paragraph"><p>first line of child</p></div>
      <div class="paragraph"><p>first line of grandchild</p></div>
      <div class="paragraph"><p>last line of grandchild</p></div>
      <div class="paragraph"><p>last line of child</p></div>
      <div class="paragraph"><p>last line of parent</p></div>
    "#}
  );
}

#[cfg(unix)]
#[test]
fn test_relative_nested_includes() {
  let stdout = run_file(
    &["--embedded", "--strict", "--safe-mode", "unsafe"],
    "tests/all/fixtures/gen/relative-include.adoc",
  );
  expect_eq!(
    stdout.trim(),
    html! {r#"
      <div class="paragraph"><p>first line of outer</p></div>
      <div class="paragraph"><p>first line of middle</p></div>
      <div class="paragraph"><p>only line of inner</p></div>
      <div class="paragraph"><p>last line of middle</p></div>
      <div class="paragraph"><p>last line of outer</p></div>
    "#}
  );
}

// run on linux (CI) only for speed in local dev
#[cfg(target_os = "linux")]
#[test]
fn test_url_includes() {
  let stdout = run_file(
    &[
      "--embedded",
      "--strict",
      "--safe-mode",
      "unsafe",
      "--attribute",
      "allow-uri-read",
    ],
    "tests/all/fixtures/remote.adoc",
  );
  expect_eq!(
    stdout.trim(),
    html! {r#"
      <div class="paragraph"><p>line 1</p></div>
      <div class="paragraph"><p>from <em>github</em></p></div>
    "#}
  );
}

#[cfg(unix)]
#[test]
fn test_cli_app_doc_attrs() {
  let stdout = run_file(
    &["--embedded", "--strict", "--safe-mode", "unsafe"],
    "tests/all/fixtures/gen/attrs.adoc",
  );
  expect_eq!(
    stdout.trim(),
    html! {r#"
      <div class="paragraph">
        <p>f: <em>fixtures/gen/attrs.adoc</em></p>
      </div>
      <div class="paragraph">
        <p>docdir: {cwd}/tests/all/fixtures/gen</p>
      </div>
      <div class="paragraph">
        <p>docfile: {cwd}/tests/all/fixtures/gen/attrs.adoc</p>
      </div>
      <div class="paragraph">
        <p>docfilesuffix: .adoc</p>
      </div>
      <div class="paragraph">
        <p>docname: attrs</p>
      </div>
    "#}
    .replace("{cwd}", &cwd())
  );
}

#[test]
fn test_cli_runs_on_windows() {
  let stdout = run_file(
    &["--embedded", "--strict", "--safe-mode", "unsafe"],
    "tests/all/fixtures/gen/gchild-include.adoc",
  );
  expect_eq!(
    stdout.trim(),
    html! {r#"
      <div class="paragraph">
        <p>first line of grandchild</p>
      </div>
      <div class="paragraph">
        <p>last line of grandchild</p>
      </div>
    "#}
    .replace("{cwd}", &cwd())
  );
}

#[test]
fn test_cli_doctype() {
  let stdout = run_file(&[], "tests/all/fixtures/book.adoc");
  assert!(stdout.contains("doctype: book"));
}
