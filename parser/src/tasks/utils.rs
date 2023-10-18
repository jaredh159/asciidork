use bumpalo::collections::String;
use bumpalo::Bump;

use crate::token::Token;

#[derive(Debug)]
pub(super) struct Text<'alloc> {
  allocator: &'alloc Bump,
  string: Option<String<'alloc>>,
}

impl<'alloc> Text<'alloc> {
  pub fn new_in(allocator: &'alloc Bump) -> Self {
    Text {
      string: Some(String::new_in(allocator)),
      allocator,
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

  pub fn take(&mut self) -> String<'alloc> {
    self.string.replace(String::new_in(self.allocator)).unwrap()
  }

  pub fn replace(&mut self, s: String<'alloc>) -> String {
    self.string.replace(s).unwrap()
  }
}
