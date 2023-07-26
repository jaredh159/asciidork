use crate::err::SourceLocation;
use crate::parse::{Parser, Result};
use crate::token::{Token, TokenType};

#[derive(Debug, Eq, PartialEq)]
pub struct Diagnostic {
  pub line_num: usize,
  pub line: String,
  pub message: String,
  pub message_offset: usize,
  pub source_start: usize,
  pub source_end: usize,
  pub token_type: Option<TokenType>,
}

impl Parser {
  pub(crate) fn err_expected_token(&mut self, token: Option<&Token>, detail: &str) -> Result<()> {
    let location = token.map_or(self.lexer.current_location(), SourceLocation::from);
    let (line_num, message_offset) = self.lexer.line_number_with_offset(location.start);
    let error = Diagnostic {
      line_num,
      line: self.lexer.line_of(location.start).to_string(),
      message: format!("Expected {}", detail),
      message_offset: message_offset + token.map_or(0, Token::len),
      source_start: location.start,
      source_end: location.end,
      token_type: location.token_type,
    };
    if self.bail {
      Err(error)
    } else {
      self.errors.push(error);
      Ok(())
    }
  }
}
