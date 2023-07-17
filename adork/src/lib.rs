#![allow(dead_code)]

mod either;
pub mod err;
mod lexer;
pub mod parse;
mod token;

#[cfg(test)]
mod t {
  use crate::lexer::Lexer;
  use crate::parse::line::Line;
  use crate::parse::Parser;

  pub fn parser_of(input: &str) -> Parser<&[u8]> {
    let lexer = Lexer::<&[u8]>::new_from(input);
    Parser::new(lexer)
  }

  pub fn line_test(input: &str) -> (Line, Parser<&[u8]>) {
    let mut parser = parser_of(input);
    let line = parser.read_line().unwrap();
    (line, parser)
  }

  /// test helper, converts &str -> String
  pub fn s(input: &str) -> String {
    input.to_string()
  }
}
