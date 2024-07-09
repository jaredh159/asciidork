use crate::internal::*;

pub fn number_prefix(level: u8, sect_nums: &mut [u16; 5]) -> String {
  debug_assert!(level > 0 && level < 6);
  let level_idx = (level - 1) as usize;
  sect_nums[level_idx] += 1;
  sect_nums
    .iter_mut()
    .take(5)
    .skip(level_idx + 1)
    .for_each(|n| *n = 0);
  let mut out = String::with_capacity(10);
  let mut idx = 0;
  while idx <= level_idx {
    out.push_str(&sect_nums[idx].to_string());
    out.push('.');
    idx += 1;
  }
  out.push(' ');
  out
}

pub fn class(section: &Section) -> &'static str {
  match section.level {
    1 => "sect1",
    2 => "sect2",
    3 => "sect3",
    4 => "sect4",
    5 => "sect5",
    6 => "sect6",
    _ => unreachable!("section::class()"),
  }
}

impl AsciidoctorHtml {
  pub(super) fn should_number_section(&self, section: &Section) -> bool {
    let Some(sectnums) = self.doc_meta.get("sectnums") else {
      return false;
    };
    if self.section_num_levels < section.level as isize {
      return false;
    }
    match sectnums {
      AttrValue::String(val) if val == "all" => true,
      AttrValue::Bool(true) => {
        if let Some(special) = section
          .meta
          .attrs
          .as_ref()
          .and_then(|a| a.str_positional_at(0))
        {
          self
            .doc_meta
            .get_doctype()
            .supports_special_section(special)
        } else {
          true
        }
      }
      _ => false,
    }
  }
}

// tests

#[cfg(test)]
mod tests {
  use super::*;
  use test_utils::*;

  #[test]
  fn test_number_prefix() {
    let cases = vec![
      (1, [0, 0, 0, 0, 0], "1. ", [1, 0, 0, 0, 0]),
      (1, [1, 0, 0, 0, 0], "2. ", [2, 0, 0, 0, 0]),
      (2, [1, 0, 0, 0, 0], "1.1. ", [1, 1, 0, 0, 0]),
      (2, [1, 1, 0, 0, 0], "1.2. ", [1, 2, 0, 0, 0]),
      (1, [1, 1, 0, 0, 0], "2. ", [2, 0, 0, 0, 0]),
      (3, [2, 4, 0, 0, 0], "2.4.1. ", [2, 4, 1, 0, 0]),
    ];
    for (level, mut sect_nums, expected, after_mutation) in cases {
      eq!(number_prefix(level, &mut sect_nums), expected.to_string());
      eq!(sect_nums, after_mutation);
    }
  }
}
