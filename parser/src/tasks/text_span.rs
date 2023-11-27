use crate::ast::*;
use crate::token::Token;
use crate::utils::bump::*;

#[derive(Debug)]
pub struct TextSpan<'bmp> {
  bump: &'bmp bumpalo::Bump,
  string: Option<String<'bmp>>,
  pub loc: SourceLocation,
}

impl<'bmp> TextSpan<'bmp> {
  pub fn new_in(loc: SourceLocation, bump: &'bmp bumpalo::Bump) -> Self {
    TextSpan {
      bump,
      string: Some(String::new_in(bump)),
      loc,
    }
  }

  pub fn push_token(&mut self, token: &Token<'_>) {
    self.string.as_mut().unwrap().push_str(token.lexeme);
    self.loc.extend(token.loc);
  }

  pub fn take(&mut self) -> String<'bmp> {
    self.loc = self.loc.clamp_end();
    self.string.replace(String::new_in(self.bump)).unwrap()
  }

  pub fn commit_inlines(&mut self, inlines: &mut Vec<'bmp, InlineNode<'bmp>>) {
    match (self.is_empty(), inlines.last_mut()) {
      (
        false,
        Some(InlineNode {
          content: Inline::Text(ref mut text),
          loc,
        }),
      ) => {
        text.push_str(&self.take());
        loc.extend(self.loc);
      }
      (false, _) => {
        inlines.push(InlineNode {
          loc: self.loc,
          content: Inline::Text(self.take()),
        });
        self.loc = self.loc.clamp_end();
      }
      _ => {}
    }
  }

  pub fn is_empty(&self) -> bool {
    self.string.as_ref().unwrap().len() == 0
  }

  pub fn ends_with(&self, predicate: impl FnMut(char) -> bool) -> bool {
    self.string.as_ref().unwrap().ends_with(predicate)
  }
}
