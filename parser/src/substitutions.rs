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
pub struct Substitutions {
  order: [Option<Subs>; 6],
  test: u8,
}

impl Substitutions {
  pub const fn contains(&self, sub: Subs) -> bool {
    self.test & sub.bitflag_pos() != 0
  }

  pub fn insert(&mut self, sub: Subs) {
    if self.contains(sub) {
      return;
    }
    self.test |= sub.bitflag_pos();
    for i in 0..5 {
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
    self.test &= !sub.bitflag_pos();
    let mut next = [None; 6];
    let mut j = 0;
    for i in 0..6 {
      if self.order[i] == Some(sub) {
        continue;
      }
      next[j] = self.order[i];
      j += 1;
    }
    self.order = next;
  }

  pub const fn normal() -> Self {
    Self::all()
  }

  pub const fn all() -> Self {
    Self {
      // TODO: what's the definitive order to start?
      order: [
        Some(Subs::SpecialChars),
        Some(Subs::InlineFormatting),
        Some(Subs::AttrRefs),
        Some(Subs::CharReplacement),
        Some(Subs::Macros),
        Some(Subs::PostReplacement),
      ],
      test: 0xFF,
    }
  }

  pub const fn none() -> Self {
    Self { order: [None; 6], test: 0x00 }
  }

  pub const fn special_chars(&self) -> bool {
    self.test & Subs::SPECIAL_CHARS != 0
  }

  pub const fn inline_formatting(&self) -> bool {
    self.test & Subs::INLINE_FORMATTING != 0
  }

  pub const fn attr_refs(&self) -> bool {
    self.test & Subs::ATTR_REFS != 0
  }

  pub const fn char_replacement(&self) -> bool {
    self.test & Subs::CHAR_REPLACEMENT != 0
  }

  pub const fn macros(&self) -> bool {
    self.test & Subs::MACROS != 0
  }

  pub const fn post_replacement(&self) -> bool {
    self.test & Subs::POST_REPLACEMENT != 0
  }

  fn prepend(&mut self, sub: Subs) {
    let mut next = [None; 6];
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
  }

  fn append(&mut self, sub: Subs) {
    let mut next = [None; 6];
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
  }
}

impl Subs {
  const SPECIAL_CHARS: u8 = 0x01;
  const INLINE_FORMATTING: u8 = 0x02;
  const ATTR_REFS: u8 = 0x04;
  const CHAR_REPLACEMENT: u8 = 0x08;
  const MACROS: u8 = 0x10;
  const POST_REPLACEMENT: u8 = 0x20;

  const fn bitflag_pos(&self) -> u8 {
    match self {
      Subs::SpecialChars => Self::SPECIAL_CHARS,
      Subs::InlineFormatting => Self::INLINE_FORMATTING,
      Subs::AttrRefs => Self::ATTR_REFS,
      Subs::CharReplacement => Self::CHAR_REPLACEMENT,
      Subs::Macros => Self::MACROS,
      Subs::PostReplacement => Self::POST_REPLACEMENT,
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
      let mut expected_order = [None; 6];
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
      let mut expected_order = [None; 6];
      for i in 0..5 {
        if i < expected.len() {
          expected_order[i] = Some(expected[i]);
        }
      }
      assert_eq!(substitutions.order, expected_order);
    }
  }

  #[test]
  fn test_jared_can_do_basic_bit_manipulation() {
    let cases: Vec<(&[Subs], u8, &str)> = vec![
      (&[SpecialChars], 0x01, "00000001"),
      (&[Macros, InlineFormatting], 0x12, "00010010"),
      (&[InlineFormatting], 0x02, "00000010"),
      (&[AttrRefs], 0x04, "00000100"),
      (&[CharReplacement], 0x08, "00001000"),
      (&[Macros], 0x10, "00010000"),
      (&[PostReplacement], 0x20, "00100000"),
      (&[SpecialChars, InlineFormatting], 0x03, "00000011"),
    ];

    for (sub, bitflags, binary) in cases {
      let mut subs = Substitutions::none();
      for s in sub {
        subs.insert(*s);
      }
      assert_eq!(format!("{:08b}", subs.test), binary);
      assert_eq!(subs.test, bitflags);
    }
  }
}
