use regex::Regex;

use crate::prelude::*;
use crate::variants::token::*;

impl<'bmp, 'src> Parser<'bmp, 'src> {
  /// if this function is called, the following invaraints hold:
  /// - the line is not empty
  /// - the line starts with a word
  /// - we are considering the line _directly_ below the doc title
  ///
  /// Therefore, it would be an error for this line to not be an author line
  pub(super) fn parse_author_line(
    &self,
    line: Line<'bmp, 'src>,
    authors: &mut Vec<'bmp, Author<'bmp>>,
  ) -> Result<()> {
    debug_assert!(!line.is_empty());
    debug_assert!(line.starts(Word));

    // https://regexr.com/7m8ni
    let pattern =
      r"([^\s<]+\b)(\s+([^<;]+\b))*(\s*([^\s<;]+))(?:\s+<([^\s>@]+@[^\s>]+)>)?(\s*;\s*)?";
    let re = Regex::new(pattern).unwrap();

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

    let num_bytes = line.src.bytes().len();
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

  pub(crate) fn author_from(&self, captures: regex::Captures<'bmp>) -> Author<'bmp> {
    let first_name = captures.get(1).unwrap().as_str();
    let middle_name = captures.get(3).map(|m| m.as_str().trim_end());
    let last_name = captures.get(5).unwrap().as_str();
    let email = captures.get(6).map(|m| m.as_str());
    return Author {
      first_name: String::from_str_in(first_name, self.bump),
      middle_name: middle_name.map(|m| String::from_str_in(m, self.bump)),
      last_name: String::from_str_in(last_name, self.bump),
      email: email.map(|e| String::from_str_in(e, self.bump)),
    };
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::test::*;

  #[test]
  fn test_parse_author_lines() {
    #[allow(clippy::type_complexity)]
    let cases: StdVec<(&str, StdVec<(&str, Option<&str>, &str, Option<&str>)>)> = vec![
      ("Bob L. Foo", vec![("Bob", Some("L."), "Foo", None)]),
      ("Bob Foo", vec![("Bob", None, "Foo", None)]),
      (
        "Bob L. Foo <bob@foo.com>",
        vec![("Bob", Some("L."), "Foo", Some("bob@foo.com"))],
      ),
      (
        "Kismet R. Lee <kismet@asciidoctor.org>",
        vec![("Kismet", Some("R."), "Lee", Some("kismet@asciidoctor.org"))],
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

    let b = &Bump::new();
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
        .collect::<StdVec<Author>>();
      let mut authors = bumpalo::collections::Vec::new_in(b);
      parser.parse_author_line(line, &mut authors).unwrap();
      assert_eq!(authors.to_vec(), expected_authors);
    }
  }
}
