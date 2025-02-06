use std::process::{Command, Stdio};

use test_utils::*;

#[cfg(unix)]
#[test]
fn test_cli_app_single_include() {
  let stdout = run_cli(
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

#[cfg(unix)]
#[test]
fn test_relative_includes() {
  let stdout = run_cli(
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

#[cfg(unix)]
#[test]
fn test_remote_relative_includes() {
  let stdout = run_cli(
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
  let stdout = run_cli(
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

#[cfg(unix)]
#[test]
fn test_url_includes() {
  let stdout = run_cli(
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
  let stdout = run_cli(
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
  let stdout = run_cli(
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

fn run_cli(args: &[&str], input: &str) -> String {
  let child = Command::new("cargo")
    .arg("run")
    .args(["--quiet", "--"])
    .args(["--input", input])
    .args(args)
    .stdin(Stdio::piped())
    .stderr(Stdio::piped())
    .stdout(Stdio::piped())
    .spawn()
    .unwrap();

  let output = child.wait_with_output().unwrap();
  let stdout = String::from_utf8_lossy(&output.stdout);

  if !output.status.success() {
    let stderr = String::from_utf8_lossy(&output.stderr);
    println!("{stderr}");
    panic!("\nCommand failed: {:?}", output.status);
  }
  stdout.to_string()
}

fn cwd() -> String {
  std::env::current_dir()
    .unwrap()
    .to_string_lossy()
    .to_string()
}
