use ast::CellContentStyle;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Subs {
  SpecialChars,
  InlineFormatting,
  AttrRefs,
  CharReplacement,
  Macros,
  PostReplacement,
  Callouts,
}

#[derive(Debug, Clone, Eq, PartialEq, Copy)]
pub struct Substitutions {
  order: [Option<Subs>; 7],
  flags: u8,
}

impl Default for Substitutions {
  fn default() -> Self {
    Self::normal()
  }
}

impl Substitutions {
  pub const fn contains(&self, sub: Subs) -> bool {
    self.flags & sub.bitflag_pos() != 0
  }

  pub const fn normal() -> Self {
    Self {
      // TODO: what's the definitive order to start?
      order: [
        Some(Subs::SpecialChars),
        Some(Subs::InlineFormatting),
        Some(Subs::AttrRefs),
        Some(Subs::CharReplacement),
        Some(Subs::Macros),
        Some(Subs::PostReplacement),
        None,
      ],
      flags: 0b_00111111,
    }
  }

  pub const fn attr_value() -> Self {
    Self {
      order: [
        Some(Subs::SpecialChars),
        Some(Subs::InlineFormatting),
        None,
        Some(Subs::CharReplacement),
        None,
        None,
        None,
      ],
      flags: 0b_00001011,
    }
  }

  pub const fn verbatim() -> Self {
    Self {
      order: [
        Some(Subs::SpecialChars),
        Some(Subs::Callouts),
        None,
        None,
        None,
        None,
        None,
      ],
      flags: 0b_01000001,
    }
  }

  pub const fn only_special_chars() -> Self {
    Self {
      order: [Some(Subs::SpecialChars), None, None, None, None, None, None],
      flags: 0b_00000001,
    }
  }

  pub const fn all() -> Self {
    Self {
      order: [
        Some(Subs::SpecialChars),
        Some(Subs::InlineFormatting),
        Some(Subs::AttrRefs),
        Some(Subs::CharReplacement),
        Some(Subs::Macros),
        Some(Subs::PostReplacement),
        Some(Subs::Callouts),
      ],
      flags: 0b_01111111,
    }
  }

  pub const fn none() -> Self {
    Self { order: [None; 7], flags: 0x00 }
  }

  pub const fn special_chars(&self) -> bool {
    self.flags & Subs::SPECIAL_CHARS != 0
  }

  pub const fn inline_formatting(&self) -> bool {
    self.flags & Subs::INLINE_FORMATTING != 0
  }

  pub const fn attr_refs(&self) -> bool {
    self.flags & Subs::ATTR_REFS != 0
  }

  pub const fn char_replacement(&self) -> bool {
    self.flags & Subs::CHAR_REPLACEMENT != 0
  }

  pub const fn macros(&self) -> bool {
    self.flags & Subs::MACROS != 0
  }

  pub const fn post_replacement(&self) -> bool {
    self.flags & Subs::POST_REPLACEMENT != 0
  }

  pub const fn callouts(&self) -> bool {
    self.flags & Subs::CALLOUTS != 0
  }

  pub fn insert(&mut self, sub: Subs) {
    if self.contains(sub) {
      return;
    }
    self.flags |= sub.bitflag_pos();
    for i in 0..6 {
      if self.order[i].is_none() {
        self.order[i] = Some(sub);
        return;
      }
    }
    debug_assert!(false);
  }

  pub fn remove(&mut self, sub: Subs) {
    if !self.contains(sub) {
      return;
    }
    self.flags &= !sub.bitflag_pos();
    let mut next = [None; 7];
    let mut j = 0;
    for i in 0..7 {
      if self.order[i] == Some(sub) {
        continue;
      }
      next[j] = self.order[i];
      j += 1;
    }
    self.order = next;
  }

  pub fn prepend(&mut self, sub: Subs) {
    let mut next = [None; 7];
    next[0] = Some(sub);
    let mut j = 1;
    for existing in self.order {
      let Some(s) = existing else {
        continue;
      };
      if s != sub {
        next[j] = Some(s);
        j += 1;
      }
    }
    self.order = next;
    self.flags |= sub.bitflag_pos();
  }

  pub fn append(&mut self, sub: Subs) {
    let mut next = [None; 7];
    let mut j = 0;
    for existing in self.order {
      let Some(s) = existing else {
        continue;
      };
      next[j] = Some(s);
      j += 1;
    }
    if j == 0 || next[j - 1] != Some(sub) {
      next[j] = Some(sub);
    }
    self.order = next;
    self.flags |= sub.bitflag_pos();
  }
}

#[rustfmt::skip]
impl Subs {
  const SPECIAL_CHARS:     u8 = 0b_00000001;
  const INLINE_FORMATTING: u8 = 0b_00000010;
  const ATTR_REFS:         u8 = 0b_00000100;
  const CHAR_REPLACEMENT:  u8 = 0b_00001000;
  const MACROS:            u8 = 0b_00010000;
  const POST_REPLACEMENT:  u8 = 0b_00100000;
  const CALLOUTS:          u8 = 0b_01000000;

  const fn bitflag_pos(&self) -> u8 {
    match self {
      Subs::SpecialChars => Self::SPECIAL_CHARS,
      Subs::InlineFormatting => Self::INLINE_FORMATTING,
      Subs::AttrRefs => Self::ATTR_REFS,
      Subs::CharReplacement => Self::CHAR_REPLACEMENT,
      Subs::Macros => Self::MACROS,
      Subs::PostReplacement => Self::POST_REPLACEMENT,
      Subs::Callouts => Self::CALLOUTS,
    }
  }
}

impl From<CellContentStyle> for Substitutions {
  fn from(style: CellContentStyle) -> Self {
    match style {
      CellContentStyle::AsciiDoc => Substitutions::normal(),
      CellContentStyle::Default => Substitutions::normal(),
      CellContentStyle::Emphasis => Substitutions::normal(),
      CellContentStyle::Header => Substitutions::normal(),
      CellContentStyle::Literal => Substitutions::verbatim(),
      CellContentStyle::Monospace => Substitutions::normal(),
      CellContentStyle::Strong => Substitutions::normal(),
    }
  }
}

// tests

#[cfg(test)]
mod tests {
  use super::*;
  use Subs::*;

  #[test]
  fn test_append() {
    let cases: Vec<(&[Subs], Subs, &[Subs])> = vec![
      (&[], Macros, &[Macros]),
      (&[SpecialChars], SpecialChars, &[SpecialChars]),
      (&[SpecialChars], Macros, &[SpecialChars, Macros]),
      (
        &[Macros, SpecialChars],
        SpecialChars,
        &[Macros, SpecialChars],
      ),
      (
        &[SpecialChars, InlineFormatting],
        Macros,
        &[SpecialChars, InlineFormatting, Macros],
      ),
    ];

    for (initial, sub, expected) in cases {
      let mut substitutions = Substitutions::none();
      initial.iter().for_each(|s| substitutions.insert(*s));
      substitutions.append(sub);
      let mut expected_order = [None; 7];
      for i in 0..5 {
        if i < expected.len() {
          expected_order[i] = Some(expected[i]);
        }
      }
      assert_eq!(substitutions.order, expected_order);
    }
  }

  #[test]
  fn test_prepend() {
    let cases: Vec<(&[Subs], Subs, &[Subs])> = vec![
      (&[], Macros, &[Macros]),
      (&[SpecialChars], SpecialChars, &[SpecialChars]),
      (&[SpecialChars], Macros, &[Macros, SpecialChars]),
      (
        &[Macros, SpecialChars],
        SpecialChars,
        &[SpecialChars, Macros],
      ),
      (
        &[SpecialChars, InlineFormatting],
        Macros,
        &[Macros, SpecialChars, InlineFormatting],
      ),
    ];

    for (initial, sub, expected) in cases {
      let mut substitutions = Substitutions::none();
      initial.iter().for_each(|s| substitutions.insert(*s));
      substitutions.prepend(sub);
      let mut expected_order = [None; 7];
      for i in 0..5 {
        if i < expected.len() {
          expected_order[i] = Some(expected[i]);
        }
      }
      assert_eq!(substitutions.order, expected_order);
    }
  }
}
