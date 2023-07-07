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
  use crate::token::{Token, TokenType};

  /// _NB:_ adds a newline token to the end of the line
  pub fn line_test(input: &str) -> (Line, Parser<&[u8]>) {
    let lexer = Lexer::<&[u8]>::new_from(input);
    let mut parser = Parser::new(lexer);
    let mut line = parser.read_line().unwrap();
    let last_end = line.last_token().map(|t| t.end).unwrap_or(0);
    line.tokens.push_back(Token {
      token_type: TokenType::Newlines,
      start: last_end,
      end: last_end + 1,
    });
    (line, parser)
  }

  /// test helper, converts &str -> String
  pub fn s(input: &str) -> String {
    input.to_string()
  }
}
