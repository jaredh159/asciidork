use std::io::BufRead;

use crate::err::{ParseErr, ParseErrLoc};
use crate::parse::Parser;

pub struct ParseErrData<'a> {
  pub line_num: usize,
  pub line: &'a str,
  pub message: String,
  pub message_offset: usize,
}

impl<R: BufRead> Parser<R> {
  pub fn display_err(&self, err: ParseErr) -> ParseErrData {
    match err {
      ParseErr::Error(_, _) => todo!(),
      ParseErr::ExpectedTokenNotFound(err_loc, msg) => self.expected_token_not_found(err_loc, msg),
    }
  }

  fn expected_token_not_found(&self, err_loc: ParseErrLoc, msg: String) -> ParseErrData {
    let line_num = 33; // = self.lexer.line_num(err_loc.start);
    let line = "lol"; // self.lexer.line(line_num);
    let message_offset = 3; // err_loc.start - self.lexer.line_start(line_num);
    ParseErrData {
      line_num,
      line,
      message: msg,
      message_offset,
    }
  }
}
