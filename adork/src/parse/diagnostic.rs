use crate::err::SourceLocation;
use crate::parse::Parser;
use crate::token::TokenType;

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
  fn err_expected_token(&self, location: SourceLocation, detail: &str) -> Diagnostic {
    let (line_num, message_offset) = self.lexer.line_number_with_offset(location.start);
    Diagnostic {
      line_num,
      line: self.lexer.line_of(location.start).to_string(),
      message: format!("Expected {}", detail),
      message_offset,
      source_start: location.start,
      source_end: location.end,
      token_type: location.token_type,
    }
  }
}
