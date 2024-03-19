use lazy_static::lazy_static;
use regex::Regex;

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
macro_rules! assert_eq {
  ($left:expr, $right:expr$(,)?) => {{
    ::pretty_assertions::assert_eq!(@ $left, $right, "", "");
  }};
  ($left:expr, $right:expr, from: $adoc:expr) => {{
    ::pretty_assertions::assert_eq!(
      $left,
      $right,
      "input was:\n\n```\n{}{}```\n",
      $adoc,
      if $adoc.ends_with('\n') { "" } else { "\n" }
    );
  }};
}

#[macro_export]
macro_rules! parse_block {
  ($input:expr, $block:ident, $bump:ident) => {
    let $bump = &Bump::new();
    let mut parser = Parser::new($bump, $input);
    let $block = parser.parse_block().unwrap().unwrap();
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
