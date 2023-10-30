use bumpalo::collections::String;
use bumpalo::Bump;

use crate::token::Token;

#[derive(Debug)]
pub(super) struct Text<'bmp> {
  bump: &'bmp Bump,
  string: Option<String<'bmp>>,
}

impl<'bmp> Text<'bmp> {
  pub fn new_in(bump: &'bmp Bump) -> Self {
    Text {
      string: Some(String::new_in(bump)),
      bump,
    }
  }

  pub fn push_str(&mut self, s: &str) {
    self.string.as_mut().unwrap().push_str(s);
  }

  pub fn push_token(&mut self, token: &Token) {
    self.push_str(token.lexeme);
  }

  pub fn trim_end(&mut self) {
    if self.string.as_ref().unwrap().ends_with(' ') {
      self.string.as_mut().unwrap().pop();
    }
  }

  pub fn is_empty(&self) -> bool {
    self.string.as_ref().unwrap().len() == 0
  }

  pub fn take(&mut self) -> String<'bmp> {
    self.string.replace(String::new_in(self.bump)).unwrap()
  }

  pub fn replace(&mut self, s: String<'bmp>) -> String {
    self.string.replace(s).unwrap()
  }
}
