use regex::Regex;

use crate::internal::*;
use crate::variants::token::*;

impl<'bmp, 'src> Parser<'bmp, 'src> {
  /// if this function is called, the following invaraints hold:
  /// - the line is not empty
  /// - the line starts with a word
  /// - we are considering the line _directly_ below the doc title
  ///
  /// Therefore, it would be an error for this line to not be an author line
  pub(super) fn parse_author_line(&mut self, line: Line<'bmp, 'src>) -> Result<()> {
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
      self.document.meta.add_author(self.author_from(captures));
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

  fn author_from(&self, captures: regex::Captures<'bmp>) -> Author {
    let first_name = captures.get(1).unwrap().as_str().to_string();
    let middle_name = captures.get(3).map(|m| m.as_str().trim_end().to_string());
    let last_name = captures.get(5).unwrap().as_str().to_string();
    let email = captures.get(6).map(|m| m.as_str().to_string());
    Author {
      first_name,
      middle_name,
      last_name,
      email,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use test_utils::{assert_eq, *};

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

    for (input, authors) in cases {
      let mut parser = crate::Parser::new(leaked_bump(), input);
      let line = parser.read_line().unwrap();

      let expected_authors = authors
        .iter()
        .map(|(first, middle, last, email)| Author {
          first_name: first.to_string(),
          middle_name: middle.map(|m| m.to_string()),
          last_name: last.to_string(),
          email: email.map(|e| e.to_string()),
        })
        .collect::<Vec<Author>>();
      parser.parse_author_line(line).unwrap();
      assert_eq!(parser.document.meta.authors(), expected_authors);
    }
  }
}
