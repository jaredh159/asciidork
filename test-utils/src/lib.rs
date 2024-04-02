use lazy_static::lazy_static;
use regex::Regex;

pub mod ast_helpers;
pub use ast_helpers::*;

lazy_static! {
  pub static ref NEWLINES_RE: Regex = Regex::new(r"(?m)\n\s*").unwrap();
}

#[macro_export]
macro_rules! html {
  ($s:expr) => {{
    let expected = ::indoc::indoc!($s);
    test_utils::NEWLINES_RE
      .replace_all(expected, "")
      .to_string()
  }};
}

#[macro_export]
macro_rules! adoc {
  ($s:expr) => {
    ::indoc::indoc!($s)
  };
}

#[macro_export]
macro_rules! raw_html {
  ($s:expr) => {
    ::indoc::indoc!($s)
  };
}

#[macro_export]
macro_rules! assert_eq {
  ($left:expr, $right:expr$(,)?) => {{
    ::pretty_assertions::assert_eq!(@ $left, $right, "", "");
  }};
  ($left:expr, $right:expr, from: $adoc:expr) => {{
    ::pretty_assertions::assert_eq!(
      $left,
      $right,
      "input was:\n\n\x1b[2m```adoc\x1b[0m\n{}{}\x1b[2m```\x1b[0m\n",
      $adoc,
      if $adoc.ends_with('\n') { "" } else { "\n" }
    );
  }};
}

#[macro_export]
macro_rules! parse_block {
  ($input:expr, $block:ident, $bump:ident) => {
    let $bump = &bumpalo::Bump::new();
    let mut parser = Parser::new($bump, $input);
    let doc_content = parser.parse().unwrap().document.content;
    let $block = match doc_content {
      ::asciidork_ast::DocContent::Blocks(mut blocks) => {
        if blocks.len() != 1 {
          panic!("expected one block, found {}", blocks.len());
        }
        blocks.remove(0)
      }
      _ => panic!("expected block content"),
    };
  };
}

#[macro_export]
macro_rules! parse_list {
  ($input:expr, $list:ident, $bump:ident) => {
    let $bump = &Bump::new();
    let mut parser = Parser::new($bump, $input);
    let lines = parser.read_lines().unwrap();
    let $list = parser.parse_list(lines, None).unwrap();
  };
}

#[macro_export]
macro_rules! s {
  (in $bump:expr; $s:expr) => {
    bumpalo::collections::String::from_str_in($s, $bump)
  };
}
