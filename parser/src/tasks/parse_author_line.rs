use bumpalo::collections::{String, Vec};
use regex::Regex;

use crate::ast::Author;
use crate::line::Line;
use crate::token::TokenKind::*;
use crate::{Parser, Result};

impl<'alloc, 'src> Parser<'alloc, 'src> {
  /// if this function is called, the following invaraints hold:
  /// - the line is not empty
  /// - the line starts with a word
  /// - we are considering the line _directly_ below the doc title
  ///
  /// Therefore, it would be an error for this line to not be an author line
  pub(super) fn parse_author_line(
    &self,
    line: Line<'alloc, 'src>,
    authors: &mut Vec<'alloc, Author<'alloc>>,
  ) -> Result<()> {
    debug_assert!(!line.is_empty());
    debug_assert!(line.starts(Word));

    // https://regexr.com/7m8ni
    let pattern =
      r"([^\s<]+\b)(\s+([^<;]+\b))*(\s*([^\s<;]+))(?:\s+<([^\s>@]+@[^\s>]+)>)?(\s*;\s*)?";
    let re = Regex::new(pattern).unwrap();

    let num_bytes = line.src.bytes().len();
    let mut first_start = usize::MAX;
    let mut last_end = 0;
    for captures in re.captures_iter(line.src) {
      if let Some(m) = captures.get(0) {
        if m.start() < first_start {
          first_start = m.start();
        }
        last_end = m.end();
      }
      authors.push(self.author_from(captures));
    }

    if first_start == usize::MAX {
      self.err("invalid author line", line.current_token())
    } else if first_start > 0 {
      let start = line.current_token().unwrap().loc.start;
      self.err_at("invalid author line", start, start + first_start)
    } else if last_end < num_bytes {
      let start = line.current_token().unwrap().loc.start;
      self.err_at("invalid author line", start + last_end, start + num_bytes)
    } else {
      Ok(())
    }
  }

  pub(crate) fn author_from(&self, captures: regex::Captures<'alloc>) -> Author<'alloc> {
    println!("captures: {:?}", captures);
    let first_name = captures.get(1).unwrap().as_str();
    let middle_name = captures.get(3).map(|m| m.as_str().trim_end());
    let last_name = captures.get(5).unwrap().as_str();
    let email = captures.get(6).map(|m| m.as_str());
    return Author {
      first_name: String::from_str_in(first_name, self.allocator),
      middle_name: middle_name.map(|m| String::from_str_in(m, self.allocator)),
      last_name: String::from_str_in(last_name, self.allocator),
      email: email.map(|e| String::from_str_in(e, self.allocator)),
    };
  }
}

#[cfg(test)]
mod tests {
  use crate::ast::Author;
  use bumpalo::collections::String;

  macro_rules! s {
    (in $bump:expr;$s:expr) => {
      String::from_str_in($s, $bump)
    };
  }

  #[test]
  fn test_parse_author_lines() {
    #[allow(clippy::type_complexity)]
    let cases: Vec<(&str, Vec<(&str, Option<&str>, &str, Option<&str>)>)> = vec![
      ("Bob L. Foo", vec![("Bob", Some("L."), "Foo", None)]),
      ("Bob Foo", vec![("Bob", None, "Foo", None)]),
      (
        "Bob L. Foo <bob@foo.com>",
        vec![("Bob", Some("L."), "Foo", Some("bob@foo.com"))],
      ),
      (
        "Bob Foo <bob@foo.com>",
        vec![("Bob", None, "Foo", Some("bob@foo.com"))],
      ),
      (
        "Bob Foo; Bob Baz",
        vec![("Bob", None, "Foo", None), ("Bob", None, "Baz", None)],
      ),
      (
        "Bob Foo <bob@foo.com>; Bob Thomas Baz",
        vec![
          ("Bob", None, "Foo", Some("bob@foo.com")),
          ("Bob", Some("Thomas"), "Baz", None),
        ],
      ),
    ];

    let b = &bumpalo::Bump::new();
    for (input, authors) in cases {
      let mut parser = crate::Parser::new(b, input);
      let line = parser.read_line().unwrap();

      let expected_authors = authors
        .iter()
        .map(|(first, middle, last, email)| Author {
          first_name: s!(in b; first),
          middle_name: middle.map(|m| s!(in b; m)),
          last_name: s!(in b; last),
          email: email.map(|e| s!(in b; e)),
        })
        .collect::<Vec<Author>>();
      let mut authors = bumpalo::collections::Vec::new_in(b);
      parser.parse_author_line(line, &mut authors).unwrap();
      assert_eq!(authors.to_vec(), expected_authors);
    }
  }
}
