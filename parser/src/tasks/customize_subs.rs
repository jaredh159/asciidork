use crate::internal::*;

// https://docs.asciidoctor.org/asciidoc/latest/subs/apply-subs-to-blocks/
pub fn from_meta(current: Substitutions, attrs: &Option<AttrList>) -> Substitutions {
  let Some(subs) = attrs.as_ref().and_then(|a| a.named.get("subs")) else {
    return current;
  };

  let mut next = current;
  for part in subs.split(',') {
    let len = part.len();
    if len < 4 {
      // TODO: error? warning?
      continue;
    }

    let bytes = part.as_bytes();
    let (strategy, name) = match (bytes[0], bytes[len - 1]) {
      (b'+', _) => (Strategy::Append, &bytes[1..len]),
      (_, b'+') => (Strategy::Prepend, &bytes[0..len - 1]),
      (b'-', _) => (Strategy::Remove, &bytes[1..len]),
      _ => (Strategy::Replace, bytes),
    };

    let Some(step_or_group) = StepOrGroup::from(name) else {
      // TODO: error? warning?
      continue;
    };

    match strategy {
      Strategy::Replace => match step_or_group {
        StepOrGroup::None => return Substitutions::none(),
        StepOrGroup::Normal => return Substitutions::normal(),
        StepOrGroup::Verbatim => todo!(),
        StepOrGroup::SpecialChars => todo!(),
        StepOrGroup::Callouts => todo!(),
        StepOrGroup::Quotes => todo!(),
        StepOrGroup::Attributes => todo!(),
        StepOrGroup::Replacements => todo!(),
        StepOrGroup::Macros => todo!(),
        StepOrGroup::PostReplacements => todo!(),
      },
      Strategy::Append | Strategy::Prepend | Strategy::Remove => {
        let modify = match strategy {
          Strategy::Append => Substitutions::insert,
          Strategy::Prepend => Substitutions::prepend,
          Strategy::Remove => Substitutions::remove,
          _ => unreachable!(),
        };
        match step_or_group {
          StepOrGroup::None => todo!(),
          StepOrGroup::Normal => todo!(),
          StepOrGroup::Verbatim => todo!(),
          StepOrGroup::SpecialChars => {
            modify(&mut next, Subs::SpecialChars);
          }
          StepOrGroup::Callouts => todo!(),
          StepOrGroup::Quotes => {
            modify(&mut next, Subs::InlineFormatting);
          }
          StepOrGroup::Attributes => {
            modify(&mut next, Subs::AttrRefs);
          }
          StepOrGroup::Replacements => todo!(),
          StepOrGroup::Macros => {
            modify(&mut next, Subs::Macros);
          }
          StepOrGroup::PostReplacements => {
            modify(&mut next, Subs::PostReplacement);
          }
        }
      }
    }
  }
  next
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Strategy {
  Replace,
  Append,
  Prepend,
  Remove,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StepOrGroup {
  None,
  Normal,
  Verbatim,
  SpecialChars,
  Callouts,
  Quotes,
  Attributes,
  Replacements,
  Macros,
  PostReplacements,
}

impl StepOrGroup {
  const fn from(s: &[u8]) -> Option<Self> {
    match s {
      b"none" => Some(Self::None),
      b"normal" => Some(Self::Normal),
      b"verbatim" => Some(Self::Verbatim),
      b"specialchars" => Some(Self::SpecialChars),
      b"callouts" => Some(Self::Callouts),
      b"quotes" => Some(Self::Quotes),
      b"attributes" => Some(Self::Attributes),
      b"replacements" => Some(Self::Replacements),
      b"macros" => Some(Self::Macros),
      b"post_replacements" => Some(Self::PostReplacements),
      _ => None,
    }
  }
}

// tests

#[cfg(test)]
mod tests {
  use super::*;
  use test_utils::{assert_eq, parse_block};

  fn exactly(subs: &[Subs]) -> Substitutions {
    let mut s = Substitutions::none();
    subs.iter().for_each(|sub| s.insert(*sub));
    s
  }

  fn all_except(subs: &[Subs]) -> Substitutions {
    let mut s = Substitutions::all();
    subs.iter().for_each(|sub| s.remove(*sub));
    s
  }

  #[test]
  fn test_customize_subs_from_meta() {
    let cases = [
      ("[subs=none]", Substitutions::all(), Substitutions::none()),
      (
        "[subs=\"none\"]",
        Substitutions::all(),
        Substitutions::none(),
      ),
      ("[subs=normal]", Substitutions::none(), Substitutions::all()),
      (
        "[subs=-specialchars]",
        Substitutions::all(),
        all_except(&[Subs::SpecialChars]),
      ),
      (
        "[subs=macros+]",
        exactly(&[Subs::SpecialChars]),
        exactly(&[Subs::Macros, Subs::SpecialChars]),
      ),
      (
        "[subs=+macros]",
        exactly(&[Subs::SpecialChars]),
        exactly(&[Subs::SpecialChars, Subs::Macros]),
      ),
      (
        r#"[subs="attributes+,+quotes,-macros"]"#,
        exactly(&[Subs::SpecialChars, Subs::Macros]),
        exactly(&[Subs::AttrRefs, Subs::SpecialChars, Subs::InlineFormatting]),
      ),
    ];

    for (attrs, current, expected) in cases {
      let input = format!("{attrs}\nfoo");
      parse_block!(&input, block, b);
      let attrs = block.meta.attrs;
      let next = from_meta(current, &attrs);
      assert_eq!(next, expected);
    }
  }
}
