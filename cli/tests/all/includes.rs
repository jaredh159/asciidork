use std::process::{Command, Stdio};

use test_utils::*;

#[test]
fn test_cli_app_single_include() {
  let stdout = run_cli(
    &["--embedded", "--strict", "--safe-mode", "unsafe"],
    "tests/all/fixtures/a.adoc",
  );
  expect_eq!(
    stdout.trim(),
    html! {r#"
      <div class="paragraph">
        <p>docdir: {cwd}/tests/all/fixtures</p>
      </div>
      <div class="paragraph">
        <p>f: <em>fixtures/a.adoc</em></p>
      </div>
      <div class="paragraph">
        <p>docdir: {cwd}/tests/all/fixtures</p>
      </div>
      <div class="paragraph">
        <p>f: <em>fixtures/b.adoc</em></p>
      </div>
    "#}
    .replace("{cwd}", &cwd())
  );
}

#[test]
fn test_cli_app_doc_attrs() {
  let stdout = run_cli(
    &["--embedded", "--strict", "--safe-mode", "unsafe"],
    "tests/all/fixtures/attrs.adoc",
  );
  expect_eq!(
    stdout.trim(),
    html! {r#"
      <div class="paragraph">
        <p>f: <em>fixtures/attrs.adoc</em></p>
      </div>
      <div class="paragraph">
        <p>docdir: {cwd}/tests/all/fixtures</p>
      </div>
      <div class="paragraph">
        <p>docfile: {cwd}/tests/all/fixtures/attrs.adoc</p>
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
    println!("{}", stderr);
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