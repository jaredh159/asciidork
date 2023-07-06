#![allow(dead_code)]

mod either;
mod err;
mod lexer;
mod parse;
mod token;

#[cfg(test)]
mod t {
  use crate::lexer::Lexer;
  use crate::parse::line::Line;
  use crate::parse::Parser;

  pub fn line_test(input: &str) -> (Line, Parser<&[u8]>) {
    let lexer = Lexer::<&[u8]>::new_from(input);
    let mut parser = Parser::new(lexer);
    (parser.read_line().unwrap(), parser)
  }
}
