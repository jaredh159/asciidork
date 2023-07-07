use std::io::BufRead;

use super::line::Line;
use super::{ast::*, Result};
use crate::err::ParseErr;
use crate::parse::Parser;
use crate::token::{Token, TokenType::*};
use smallvec::SmallVec;

impl<R: BufRead> Parser<R> {
  pub(super) fn parse_author_line(&self, mut line: Line, authors: &mut Vec<Author>) -> Result<()> {
    line.remove_all(Whitespace);
    while let Some(author) = self.parse_single_author(&mut line)? {
      authors.push(author);
    }
    Ok(())
  }

  fn parse_single_author(&self, line: &mut Line) -> Result<Option<Author>> {
    if line.is_empty() {
      return Ok(None);
    }

    let mut name_parts = SmallVec::<[Token; 3]>::new();
    while let Some(name) = line.consume_if(Word) {
      name_parts.push(name);
    }

    if name_parts.len() < 2 {
      return Err(ParseErr::Error(
        "author name must have at least first and last name".to_string(),
        name_parts
          .first()
          .map(|token| token.clone())
          .or_else(|| line.current_token().cloned()),
      ));
    }

    let first_name = self.lexeme_string(&name_parts[0]);
    let last_name = self.lexeme_string(name_parts.last().unwrap());
    let middle_name = if name_parts.len() > 2 {
      let mut middle = self.lexeme_string(&name_parts[1]);
      for i in 2..name_parts.len() - 1 {
        middle.push(' ');
        middle.push_str(&self.lexeme_string(&name_parts[i]));
      }
      Some(middle)
    } else {
      None
    };

    let mut author = Author {
      first_name,
      middle_name,
      last_name,
      email: None,
    };

    if line.starts_with_seq(&[LessThan, Word, GreaterThan]) {
      line.consume_current();
      author.email = Some(self.lexeme_string(&line.consume_current().unwrap()));
      line.consume_current();
    }

    if line.starts_with_one_of(&[SemiColon, Newlines]) {
      line.consume_current();
    }

    Ok(Some(author))
  }
}

#[cfg(test)]
mod tests {
  use crate::parse::ast::Author;
  use crate::t::*;

  #[test]
  fn test_parse_author_lines() {
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
        .map(|(first, middle, last, email)| Author {
          first_name: s(first),
          middle_name: middle.map(s),
          last_name: s(last),
          email: email.map(s),
        })
        .collect::<Vec<Author>>();
      let mut authors = Vec::new();
      parser.parse_author_line(line, &mut authors).unwrap();
      assert_eq!(authors, expected_authors);
    }
  }
}
