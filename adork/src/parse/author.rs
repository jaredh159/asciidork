use smallvec::SmallVec;

use super::Result;
use crate::ast;
use crate::parse::Parser;
use crate::tok::{self, TokenType::*};

impl Parser {
  /// if this function is called, the following invaraints hold:
  /// - the line is not empty
  /// - the line starts with a word
  /// - we are considering the line _directly_ below the doc title
  ///
  /// Therefore, it would be an error for this line to not be an author line
  pub(super) fn parse_author_line(
    &self,
    mut line: tok::Line,
    authors: &mut Vec<ast::Author>,
  ) -> Result<()> {
    debug_assert!(!line.is_empty());
    debug_assert!(line.starts(Word));

    while let Some(author) = self.parse_single_author(&mut line)? {
      authors.push(author);
    }

    Ok(())
  }

  fn parse_single_author(&self, line: &mut tok::Line) -> Result<Option<ast::Author>> {
    if line.is_empty() {
      return Ok(None);
    }

    let clumps = line.clumps(self);
    let mut name_parts = SmallVec::<[&tok::Clump; 6]>::new();
    let mut remove_semicolon = false;
    for clump in &clumps {
      if clump.starts_with('<') {
        break;
      }
      name_parts.push(clump);
      if clump.ends_with(';') {
        remove_semicolon = true;
        break;
      }
    }

    if name_parts.len() < 2 {
      drop(name_parts);
      self.err(
        "author name must have at least first and last name",
        line.current_token(),
      )?;
      return Ok(None); // TODO: is this correct?
    }

    let first_name = name_parts[0].string();
    let last_name_part = name_parts.last().unwrap();
    let end_of_name = last_name_part.end;
    let mut last_name = last_name_part.string();
    if remove_semicolon {
      last_name.pop();
    }
    let middle_name = if name_parts.len() > 2 {
      let mut middle = name_parts[1].string();
      for i in 2..name_parts.len() - 1 {
        middle.push_str(name_parts[i].str);
      }
      Some(middle)
    } else {
      None
    };

    let mut author = ast::Author {
      first_name,
      middle_name,
      last_name,
      email: None,
    };

    drop(name_parts);
    line.discard(end_of_name);
    line.discard_leading_whitespace();

    if line.starts_with_seq(&[LessThan, Word]) && line.contains(GreaterThan) {
      line.discard(1); // `<`
      author.email = Some(line.consume_to_string_until(GreaterThan, self));
      line.discard(1); // `>`
    }

    line.print(self);
    if line.starts(SemiColon) {
      line.consume_current();
    }

    Ok(Some(author))
  }
}

#[cfg(test)]
mod tests {
  use crate::ast;
  use crate::t::*;

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

    for (input, authors) in cases {
      let (line, parser) = line_test(input);
      let expected_authors = authors
        .iter()
        .map(|(first, middle, last, email)| ast::Author {
          first_name: s(first),
          middle_name: middle.map(s),
          last_name: s(last),
          email: email.map(s),
        })
        .collect::<Vec<ast::Author>>();
      let mut authors = Vec::new();
      parser.parse_author_line(line, &mut authors).unwrap();
      assert_eq!(authors, expected_authors);
    }
  }
}
