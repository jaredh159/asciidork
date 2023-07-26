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
  use crate::parse::line_block::LineBlock;
  use crate::parse::Parser;

  pub fn line_test(input: &'static str) -> (Line, Parser) {
    let mut parser = Parser::from(input);
    let line = parser.read_line().unwrap();
    (line, parser)
  }

  pub fn block_test(input: &'static str) -> (LineBlock, Parser) {
    let mut parser = Parser::from(input);
    let block = parser.read_block().unwrap();
    (block, parser)
  }

  /// test helper, converts &str -> String
  pub fn s(input: &str) -> String {
    input.to_string()
  }
}
