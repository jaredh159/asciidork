use crate::helpers::*;
use test_utils::*;

#[test]
fn no_mods_includes_default_css() {
  let stdout = run_input(&[], "hello world");
  assert!(stdout.contains("<style>\n/*! Asciidoctor default stylesheet"));
}

#[cfg(unix)]
#[test]
fn override_stylesheet() {
  let adoc = adoc! {r#"
    :stylesheet: custom.css

    hello world
  "#};
  let stdout = run_input(&["--strict"], adoc);
  assert!(stdout.contains("<style>/* custom styles */\n</style>"));
}

#[cfg(target_os = "windows")]
#[test]
fn override_stylesheet_windows() {
  let adoc = adoc! {r#"
    :stylesheet: custom.css

    hello world
  "#};
  let stdout = run_input(&["--strict"], adoc);
  assert!(stdout.contains("<style>/* custom styles */\r\n</style>"));
}

#[cfg(unix)]
#[test]
fn override_stylesheet_w_dir() {
  let adoc = adoc! {r#"
    :stylesheet: sub-custom.css
    :stylesdir: gen/sub

    hello world
  "#};
  let stdout = run_input(&["--strict"], adoc);
  assert!(stdout.contains("<style>/* sub custom styles */\n</style>"));
}

#[cfg(target_os = "windows")]
#[test]
fn override_stylesheet_w_dir_windows() {
  let adoc = adoc! {r#"
    :stylesheet: sub-custom.css
    :stylesdir: gen\sub

    hello world
  "#};
  let stdout = run_input(&[], adoc);
  assert!(stdout.contains("<style>/* sub custom styles */\r\n</style>"));
}

#[cfg(unix)]
#[test]
fn err_404_stylesheet() {
  let adoc = adoc! {r#"
    :stylesheet: nope.css

    hello world
  "#};
  let stderr = run_input_expecting_err(&["--strict"], adoc);
  expect_eq!(stderr.trim(), "ERROR: stylesheet `./nope.css` not found");
}

#[cfg(target_os = "windows")]
#[test]
fn err_404_stylesheet_windows() {
  let adoc = adoc! {r#"
    :stylesheet: nope.css

    hello world
  "#};
  let stderr = run_input_expecting_err(&["--strict"], adoc);
  expect_eq!(stderr.trim(), "ERROR: stylesheet `.\\nope.css` not found");
}

#[cfg(unix)]
#[test]
fn err_out_of_cwd() {
  let adoc = adoc! {r#"
    :stylesheet: css.rs
    :stylesdir: ..

    hello world
  "#};
  let stderr = run_input_expecting_err(&["--strict"], adoc);
  expect_eq!(
    stderr.trim(),
    "ERROR: stylesheet path `../css.rs` not permitted outside cwd except in unsafe mode"
  );
}

#[cfg(target_os = "windows")]
#[test]
fn err_out_of_cwd_windows() {
  let adoc = adoc! {r#"
    :stylesheet: css.rs
    :stylesdir: ..

    hello world
  "#};
  let stderr = run_input_expecting_err(&["--strict"], adoc);
  expect_eq!(
    stderr.trim(),
    "ERROR: stylesheet path `..\\css.rs` not permitted outside cwd except in unsafe mode"
  );
}
