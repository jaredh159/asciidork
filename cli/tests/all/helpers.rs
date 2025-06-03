use std::io::Write;
use std::process::{Child, Command, Stdio};

pub fn run_file(args: &[&str], filepath: &str) -> String {
  let child = cmd_from_file(args, filepath);
  let output = child.wait_with_output().unwrap();
  let stdout = String::from_utf8_lossy(&output.stdout);

  if !output.status.success() {
    let stderr = String::from_utf8_lossy(&output.stderr);
    println!("{stderr}");
    panic!("\nCommand failed: {:?}", output.status);
  }

  stdout.to_string()
}

pub fn run_input(args: &[&str], input: &str) -> String {
  let mut child = cmd_for_stdin(args);
  let stdin = child.stdin.as_mut().unwrap();
  stdin.write_all(input.as_bytes()).unwrap();
  let output = child.wait_with_output().unwrap();
  let stdout = String::from_utf8_lossy(&output.stdout);

  if !output.status.success() {
    let stderr = String::from_utf8_lossy(&output.stderr);
    println!("{stderr}");
    panic!("\nCommand failed: {:?}", output.status);
  }

  stdout.to_string()
}

pub fn run_expecting_err(args: &[&str], filepath: &str) -> String {
  let child = cmd_from_file(args, filepath);
  let output = child.wait_with_output().unwrap();
  let stderr = String::from_utf8_lossy(&output.stderr);

  if output.status.success() {
    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("{stdout}");
    panic!("\nExpected error, but got none");
  }

  stderr.to_string()
}

pub fn run_input_expecting_err(args: &[&str], input: &str) -> String {
  let mut child = cmd_for_stdin(args);
  let stdin = child.stdin.as_mut().unwrap();
  stdin.write_all(input.as_bytes()).unwrap();
  let output = child.wait_with_output().unwrap();
  let stderr = String::from_utf8_lossy(&output.stderr);

  if output.status.success() {
    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("{stdout}");
    panic!("\nExpected error, but got none");
  }

  stderr.to_string()
}

fn cmd_from_file(args: &[&str], input: &str) -> Child {
  Command::new("cargo")
    .arg("run")
    .args(["--quiet", "--"])
    .args(["--input", input])
    .args(args)
    .stdin(Stdio::piped())
    .stderr(Stdio::piped())
    .stdout(Stdio::piped())
    .spawn()
    .unwrap()
}

fn cmd_for_stdin(args: &[&str]) -> Child {
  Command::new("cargo")
    .current_dir(format!("{}/tests/all/fixtures", cwd()))
    .arg("run")
    .args(["--quiet", "--"])
    .args(args)
    .stdin(Stdio::piped())
    .stderr(Stdio::piped())
    .stdout(Stdio::piped())
    .spawn()
    .unwrap()
}

pub fn cwd() -> String {
  std::env::current_dir()
    .unwrap()
    .to_string_lossy()
    .to_string()
}
