use crate::err::SourceLocation;
use crate::parse::{Parser, Result};
use crate::tok::Token;

#[derive(Debug, Eq, PartialEq)]
pub struct Diagnostic {
  pub line_num: usize,
  pub line: String,
  pub message: String,
  pub underline_start: usize,
  pub underline_width: usize,
}

impl Parser {
  pub(crate) fn err_at(&self, message: &'static str, start: usize, end: usize) -> Result<()> {
    let (line_num, offset) = self.lexer.line_number_with_offset(start);
    self.handle_err(Diagnostic {
      line_num,
      line: self.lexer.line_of(start).to_string(),
      message: message.into(),
      underline_start: offset + 1,
      underline_width: end - start,
    })
  }

  pub(crate) fn err_token_start(&self, message: &'static str, token: &Token) -> Result<()> {
    let (line_num, offset) = self.lexer.line_number_with_offset(token.start);
    self.handle_err(Diagnostic {
      line_num,
      line: self.lexer.line_of(token.start).to_string(),
      message: message.into(),
      underline_start: offset + 1,
      underline_width: 1,
    })
  }

  pub(crate) fn err_token_end(&self, message: &'static str, token: &Token) -> Result<()> {
    let (line_num, offset) = self.lexer.line_number_with_offset(token.start);
    self.handle_err(Diagnostic {
      line_num,
      line: self.lexer.line_of(token.start).to_string(),
      message: message.into(),
      underline_start: offset + 1 + token.len(),
      underline_width: 1,
    })
  }

  pub(crate) fn err_token_end_opt(
    &self,
    message: &'static str,
    token: Option<&Token>,
  ) -> Result<()> {
    let location = token.map_or_else(|| self.lexer.current_location(), SourceLocation::from);
    let (line_num, offset) = self.lexer.line_number_with_offset(location.start);
    self.handle_err(Diagnostic {
      line_num,
      line: self.lexer.line_of(location.start).to_string(),
      message: message.into(),
      underline_start: offset + 1 + token.map_or(0, Token::len),
      underline_width: 1,
    })
  }

  pub(crate) fn err(&self, message: impl Into<String>, token: Option<&Token>) -> Result<()> {
    let location = token.map_or_else(|| self.lexer.current_location(), SourceLocation::from);
    let (line_num, offset) = self.lexer.line_number_with_offset(location.start);
    self.handle_err(Diagnostic {
      line_num,
      line: self.lexer.line_of(location.start).to_string(),
      message: message.into(),
      underline_start: offset + 1,
      underline_width: 1,
    })
  }

  pub(crate) fn err_expected_token(&self, token: Option<&Token>, detail: &str) -> Result<()> {
    self.err(format!("Expected {}", detail), token)
  }

  fn handle_err(&self, err: Diagnostic) -> Result<()> {
    if self.bail {
      Err(err)
    } else {
      self.errors.borrow_mut().push(err);
      Ok(())
    }
  }
}
