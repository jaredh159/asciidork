use smallvec::SmallVec;

use super::Result;
use crate::ast;
use crate::parse::Parser;
use crate::tok::{self, Token, TokenType::*};

impl Parser {
  /// if this function is called, the following invaraints hold:
  /// - the line is not empty
  /// - the line starts with a word
  /// - we are considering the line _directly_ below the doc title
  ///
  /// Therefore, it would be an error for this line to not be an author line
  pub(super) fn parse_author_line(
    &mut self,
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

  fn parse_single_author(&mut self, line: &mut tok::Line) -> Result<Option<ast::Author>> {
    if line.is_empty() {
      return Ok(None);
    }

    let mut name_parts = SmallVec::<[&Token; 3]>::new();
    for token in &line.tokens {
      match token {
        token if token.is(Word) => name_parts.push(token),
        token if token.is(SemiColon) => break,
        token if token.is(LessThan) => break,
        _ => {}
      }
    }

    if name_parts.len() < 2 {
      self.err(
        "author name must have at least first and last name",
        name_parts
          .first()
          .map(|token| *token)
          .or_else(|| line.current_token()),
      )?;
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

    drop(name_parts);

    let mut author = ast::Author {
      first_name,
      middle_name,
      last_name,
      email: None,
    };

    line.discard_until_one_of(&[LessThan, SemiColon]);

    if line.starts_with_seq(&[LessThan, Word, GreaterThan]) {
      line.consume_current();
      author.email = Some(self.lexeme_string(&line.consume_current().unwrap()));
      line.consume_current();
    }

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
      let (line, mut parser) = line_test(input);
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
