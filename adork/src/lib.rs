#![allow(dead_code)]

mod ast;
mod either;
pub mod err;
mod lexer;
pub mod parse;
mod reader;
mod tok;

#[cfg(test)]
mod t {
  use crate::ast::Document;
  use crate::parse::Parser;
  use crate::tok;

  pub fn line_test(input: &'static str) -> (tok::Line, Parser) {
    let mut parser = Parser::from(input);
    let line = parser.read_line().unwrap();
    (line, parser)
  }

  pub fn block_test(input: &'static str) -> (tok::Block, Parser) {
    let mut parser = Parser::from(input);
    let block = parser.read_block().unwrap();
    (block, parser)
  }

  pub fn doc_test(input: &'static str) -> Document {
    Parser::from(input).parse().unwrap().document
  }

  /// test helper, converts &str -> String
  pub fn s(input: &str) -> String {
    input.to_string()
  }
}
