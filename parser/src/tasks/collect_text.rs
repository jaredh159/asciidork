use crate::internal::*;

pub struct CollectText<'bmp> {
  bump: &'bmp bumpalo::Bump,
  string: Option<BumpString<'bmp>>,
  pub loc: SourceLocation,
}

impl<'bmp> CollectText<'bmp> {
  pub fn new_in(loc: SourceLocation, bump: &'bmp bumpalo::Bump) -> Self {
    CollectText {
      bump,
      string: Some(BumpString::new_in(bump)),
      loc,
    }
  }

  pub fn push_token(&mut self, token: &Token<'_>) {
    self.string.as_mut().unwrap().push_str(&token.lexeme);
    self.loc.extend(token.loc);
  }

  pub fn push_str(&mut self, s: &str) {
    self.string.as_mut().unwrap().push_str(s);
    self.loc.end += s.len();
  }

  pub fn str(&self) -> &str {
    self.string.as_ref().unwrap()
  }

  pub fn trim_end(&mut self) {
    let string = self.string.as_mut().unwrap();
    if !string.ends_with(' ') {
      return;
    }
    let trimmed = string.trim_end();
    let mut delta = string.len() - trimmed.len();
    self.loc.end -= delta;
    while delta > 0 {
      string.pop();
      delta -= 1;
    }
  }

  pub fn drop_last(&mut self, n: usize) {
    debug_assert!(n <= self.string.as_ref().unwrap().len());
    let string = self.string.as_mut().unwrap();
    for _ in 0..n {
      string.pop();
    }
    self.loc.end -= n;
  }

  pub fn take(&mut self) -> BumpString<'bmp> {
    self.loc = self.loc.clamp_end();
    self.string.replace(BumpString::new_in(self.bump)).unwrap()
  }

  pub fn take_src(&mut self) -> SourceString<'bmp> {
    let src_loc = self.loc;
    self.loc = self.loc.clamp_end();
    SourceString::new(self.take(), src_loc)
  }

  pub fn commit_inlines(&mut self, inlines: &mut InlineNodes<'bmp>) {
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

impl<'bmp> std::fmt::Debug for CollectText<'bmp> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "TextSpan {{ string: {}, loc: {:?} }}",
      self
        .string
        .as_ref()
        .map_or("None".to_string(), |s| format!("Some({:?})", s)),
      self.loc
    )
  }
}
