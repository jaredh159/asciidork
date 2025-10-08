use ast::Section;

pub fn section_class(section: &Section) -> &'static str {
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

// tests

// todo, need a test backend
// #[cfg(test)]
// mod tests {
//   use super::*;
//
//   // struct TestBackend
//
//   #[test]
//   fn test_number_prefix() {
//     let cases = vec![
//       (1, [0, 0, 0, 0, 0], "1. ", [1, 0, 0, 0, 0], false),
//       (1, [1, 0, 0, 0, 0], "2. ", [2, 0, 0, 0, 0], false),
//       (2, [1, 0, 0, 0, 0], "1.1. ", [1, 1, 0, 0, 0], false),
//       (2, [1, 1, 0, 0, 0], "1.2. ", [1, 2, 0, 0, 0], false),
//       (1, [1, 1, 0, 0, 0], "2. ", [2, 0, 0, 0, 0], false),
//       (3, [2, 4, 0, 0, 0], "2.4.1. ", [2, 4, 1, 0, 0], false),
//       (2, [1, 1, 0, 0, 0], "B.2. ", [1, 2, 0, 0, 0], true),
//       (3, [1, 2, 0, 0, 0], "B.2.1. ", [1, 2, 1, 0, 0], true),
//     ];
//     for (level, mut sect_nums, expected, after_mutation, apndx) in cases {
//       assert_eq!(
//         section_number_prefix(level, &mut sect_nums, apndx),
//         expected.to_string()
//       );
//       assert_eq!(sect_nums, after_mutation);
//     }
//   }
// }
