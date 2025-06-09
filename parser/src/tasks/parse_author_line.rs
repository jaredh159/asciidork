use regex::Regex;

use crate::internal::*;
use crate::variants::token::*;

impl<'arena> Parser<'arena> {
  /// if this function is called, the following invaraints hold:
  /// - the line is not empty
  /// - the line starts with a word
  /// - we are considering the line _directly_ below the doc title
  ///
  /// Therefore, it would be an error for this line to not be an author line
  pub(super) fn parse_author_line(&mut self, line: Line<'arena>) -> Result<()> {
    debug_assert!(!line.is_empty());
    debug_assert!(line.starts(Word));

    // https://regexr.com/7m8ni
    let pattern =
      r"([^\s<]+\b)(\s+([^<;]+\b))*(\s*([^\s<;]+))(?:\s+<([^\s>@]+@[^\s>]+)>)?(\s*;\s*)?";
    let re = Regex::new(pattern).unwrap();

    let mut first_start = usize::MAX;
    let mut last_end = 0;
    let src = line.reassemble_src();
    let src = src.trim_end();
    for captures in re.captures_iter(src) {
      if let Some(m) = captures.get(0) {
        if m.start() < first_start {
          first_start = m.start();
        }
        last_end = m.end();
      }
      self.document.meta.add_author(self.author_from(captures));
    }

    let num_bytes = src.len();
    if first_start == usize::MAX {
      self.err_token("invalid author line", line.current_token())
    } else if first_start > 0 {
      let loc = line.current_token().unwrap().loc;
      self.err_at("invalid author line", loc.adding_to_end(first_start as u32))
    } else if last_end < num_bytes {
      let mut loc = line.current_token().unwrap().loc;
      loc.start += last_end as u32;
      loc.end += num_bytes as u32;
      self.err_at("invalid author line", loc)
    } else {
      Ok(())
    }
  }

  fn author_from(&self, captures: regex::Captures) -> Author {
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
  use test_utils::*;

  #[test]
  fn test_parse_author_lines() {
    #[allow(clippy::type_complexity)]
    let cases: Vec<(&str, Vec<(&str, Option<&str>, &str, Option<&str>)>)> = vec![
      ("Bob L. Foo", vec![("Bob", Some("L."), "Foo", None)]),
      ("Bob Foo", vec![("Bob", None, "Foo", None)]),
      ("Bob Foo ", vec![("Bob", None, "Foo", None)]), // trailing whitespace
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
      let mut parser = test_parser!(input);
      let line = parser.read_line().unwrap().unwrap();

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
