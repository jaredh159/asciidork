#![allow(dead_code)]

mod either;
pub mod err;
mod lexer;
pub mod parse;
mod reader;
mod token;

#[cfg(test)]
mod t {
  use crate::parse::line::Line;
  use crate::parse::Parser;

  pub fn line_test(input: &'static str) -> (Line, Parser) {
    let mut parser = Parser::from(input);
    let line = parser.read_line().unwrap();
    (line, parser)
  }

  /// test helper, converts &str -> String
  pub fn s(input: &str) -> String {
    input.to_string()
  }
}
