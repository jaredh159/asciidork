use crate::parse::Parser;
use crate::tok::Token;

#[derive(Debug)]
pub(super) struct Text(Option<String>);

impl Text {
  pub fn new() -> Text {
    Text(Some(String::new()))
  }

  pub fn push_str(&mut self, s: &str) {
    self.0.as_mut().unwrap().push_str(s);
  }

  pub fn push_token(&mut self, token: &Token, parser: &Parser) {
    self.push_str(parser.lexeme_str(token));
  }

  pub fn trim_end(&mut self) {
    if self.0.as_ref().unwrap().ends_with(' ') {
      self.0.as_mut().unwrap().pop();
    }
  }

  pub fn is_empty(&self) -> bool {
    self.0.as_ref().unwrap().len() == 0
  }

  pub fn take(&mut self) -> String {
    self.0.replace(String::new()).unwrap()
  }

  pub fn replace(&mut self, s: String) -> String {
    self.0.replace(s).unwrap()
  }
}
