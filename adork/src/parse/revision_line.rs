use std::borrow::Cow;

use regex_macro::regex;

use crate::ast::Revision;
use crate::parse::Parser;
use crate::tok::{self, TokenType::*};

impl Parser {
  pub(super) fn parse_revision_line(
    &mut self,
    block: &mut tok::Block,
    revision: &mut Option<Revision>,
  ) {
    let Some(line) = block.current_line() else {
      return ;
    };

    let Some(first_token) = line.current_token() else {
      return ;
    };

    if !first_token.is(Word) {
      return;
    }

    let second_token = line.peek_token();

    // single word prefixed with 'v' is revision number
    if second_token.is_none() {
      let version = self.lexeme_str(first_token);
      if version.starts_with('v') {
        *revision = Some(Revision {
          version: trim_version(version).to_string(),
          date: None,
          remark: None,
        });
        block.consume_current();
      }
      return;
    }

    if !line.nth_token_one_of(1, &[Colon, Comma]) || !line.nth_token_is(2, Whitespace) {
      return;
    }

    // we know we have a valid revision live, consume the line
    let mut line = block.consume_current().unwrap();
    let version = trim_version(self.lexeme_str(&line.consume_current().unwrap())).to_string();
    let [first_delimiter, _] = line.consume::<2>().map(|t| t.unwrap());

    // version and remark
    if first_delimiter.is(Colon) {
      *revision = Some(Revision {
        version,
        date: None,
        remark: Some(line.consume_to_string(self)),
      });
      return;
    }

    // version and only date
    if !line.contains(Colon) {
      *revision = Some(Revision {
        version,
        date: Some(line.consume_to_string(self)),
        remark: None,
      });
      return;
    }

    // version, date, and remark
    let date = line.consume_to_string_until(Colon, self);
    line.consume_current(); // colon
    line.consume_if(Whitespace);
    *revision = Some(Revision {
      version,
      date: Some(date),
      remark: Some(line.consume_to_string(self)),
    });
  }
}

fn trim_version(version: &str) -> Cow<str> {
  regex!(r"^[^0-9.]+").replace(version, "")
}

// tests

#[cfg(test)]
mod tests {
  use super::*;
  use crate::t::*;

  #[test]
  fn test_parse_revision_lines() {
    let cases = vec![
      ("foobar", None),
      (
        "v7.5",
        Some(Revision {
          version: s("7.5"),
          date: None,
          remark: None,
        }),
      ),
      (
        "v7.5, 1-29-2020",
        Some(Revision {
          version: s("7.5"),
          date: Some(s("1-29-2020")),
          remark: None,
        }),
      ),
      (
        "LPR55, 1-29-2020",
        Some(Revision {
          version: s("55"),
          date: Some(s("1-29-2020")),
          remark: None,
        }),
      ),
      (
        "7.5, 1-29-2020",
        Some(Revision {
          version: s("7.5"),
          date: Some(s("1-29-2020")),
          remark: None,
        }),
      ),
      (
        "7.5: A new analysis",
        Some(Revision {
          version: s("7.5"),
          date: None,
          remark: Some(s("A new analysis")),
        }),
      ),
      (
        "v7.5, 1-29-2020: A new analysis",
        Some(Revision {
          version: s("7.5"),
          date: Some(s("1-29-2020")),
          remark: Some(s("A new analysis")),
        }),
      ),
      ("v7.5 1-29-2020 A new analysis", None),
    ];

    for (input, expected) in cases {
      let (mut block, mut parser) = block_test(input);
      let mut revision = None;
      parser.parse_revision_line(&mut block, &mut revision);
      assert_eq!(revision, expected);
    }
  }
}
