#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Subs {
  SpecialChars,
  InlineFormatting,
  AttrRefs,
  CharReplacement,
  Macros,
  PostReplacement,
}

#[derive(Debug, Clone, Eq, PartialEq, Copy)]
pub struct Substitutions([Option<Subs>; 6]);

impl Substitutions {
  pub fn insert(&mut self, sub: Subs) {
    if self.0.contains(&Some(sub)) {
      return;
    }
    for i in 0..5 {
      if self.0[i].is_none() {
        self.0[i] = Some(sub);
        return;
      }
    }
    debug_assert!(false);
  }

  pub fn remove(&mut self, sub: Subs) {
    let mut next = [None; 6];
    let mut j = 0;
    for i in 0..6 {
      if self.0[i] == Some(sub) {
        continue;
      }
      next[j] = self.0[i];
      j += 1;
    }
    self.0 = next;
  }

  pub fn normal() -> Self {
    Self::all()
  }

  pub fn all() -> Self {
    Self([
      // TODO: what's the definitive order to start?
      Some(Subs::SpecialChars),
      Some(Subs::InlineFormatting),
      Some(Subs::AttrRefs),
      Some(Subs::CharReplacement),
      Some(Subs::Macros),
      Some(Subs::PostReplacement),
    ])
  }

  pub fn none() -> Self {
    Self([None; 6])
  }

  pub fn special_chars(&self) -> bool {
    self.0.contains(&Some(Subs::SpecialChars))
  }

  pub fn inline_formatting(&self) -> bool {
    self.0.contains(&Some(Subs::InlineFormatting))
  }

  pub fn attr_refs(&self) -> bool {
    self.0.contains(&Some(Subs::AttrRefs))
  }

  pub fn char_replacement(&self) -> bool {
    self.0.contains(&Some(Subs::CharReplacement))
  }

  pub fn macros(&self) -> bool {
    self.0.contains(&Some(Subs::Macros))
  }

  pub fn post_replacement(&self) -> bool {
    self.0.contains(&Some(Subs::PostReplacement))
  }
}
