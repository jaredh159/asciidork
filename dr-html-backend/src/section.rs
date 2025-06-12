use crate::internal::*;

pub fn number_prefix(level: u8, sect_nums: &mut [u16; 5], appendix: bool) -> String {
  debug_assert!(level > 0 && level < 6);
  let level_idx = (level - 1) as usize;
  sect_nums[level_idx] += 1;
  sect_nums
    .iter_mut()
    .skip(level_idx + 1)
    .for_each(|n| *n = 0);
  let mut out = String::with_capacity(10);
  let mut idx = 0;
  while idx <= level_idx {
    if appendix && idx == 0 {
      out.push((b'A' + sect_nums[idx] as u8) as char);
    } else if appendix && idx == 1 {
      out.push_str(&(sect_nums[idx]).to_string());
    } else {
      out.push_str(&sect_nums[idx].to_string());
    }
    out.push('.');
    idx += 1;
  }
  out.push(' ');
  out
}

pub fn class(section: &Section) -> &'static str {
  match section.level {
    0 => "sect0",
    1 => "sect1",
    2 => "sect2",
    3 => "sect3",
    4 => "sect4",
    5 => "sect5",
    6 => "sect6",
    _ => unreachable!("section::class() level={}", section.level),
  }
}

impl AsciidoctorHtml {
  pub(super) fn push_section_heading_prefix(
    &mut self,
    level: u8,
    special_sect: Option<SpecialSection>,
  ) {
    if self.should_number_section(level, special_sect) {
      if level == 1 && self.doc_meta.get_doctype() == DocType::Book {
        if let Some(chapter_signifier) = self.doc_meta.string("chapter-signifier") {
          self.push([&chapter_signifier, " "]);
        }
      }
      let prefix = section::number_prefix(
        level,
        &mut self.section_nums,
        self.state.contains(&EphemeralState::InAppendix),
      );
      self.push_str(&prefix);
    }
  }

  pub(super) fn should_number_section(
    &self,
    level: u8,
    special_sect: Option<SpecialSection>,
  ) -> bool {
    let Some(sectnums) = self.doc_meta.get("sectnums") else {
      return false;
    };
    if self.section_num_levels < level as isize {
      return false;
    }
    match sectnums {
      AttrValue::String(val) if val == "all" => true,
      AttrValue::Bool(true) => {
        if let Some(special_sect) = special_sect {
          self
            .doc_meta
            .get_doctype()
            .supports_special_section(special_sect)
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
      (1, [0, 0, 0, 0, 0], "1. ", [1, 0, 0, 0, 0], false),
      (1, [1, 0, 0, 0, 0], "2. ", [2, 0, 0, 0, 0], false),
      (2, [1, 0, 0, 0, 0], "1.1. ", [1, 1, 0, 0, 0], false),
      (2, [1, 1, 0, 0, 0], "1.2. ", [1, 2, 0, 0, 0], false),
      (1, [1, 1, 0, 0, 0], "2. ", [2, 0, 0, 0, 0], false),
      (3, [2, 4, 0, 0, 0], "2.4.1. ", [2, 4, 1, 0, 0], false),
      (2, [1, 1, 0, 0, 0], "B.2. ", [1, 2, 0, 0, 0], true),
      (3, [1, 2, 0, 0, 0], "B.2.1. ", [1, 2, 1, 0, 0], true),
    ];
    for (level, mut sect_nums, expected, after_mutation, apndx) in cases {
      expect_eq!(
        number_prefix(level, &mut sect_nums, apndx),
        expected.to_string()
      );
      expect_eq!(sect_nums, after_mutation);
    }
  }
}
