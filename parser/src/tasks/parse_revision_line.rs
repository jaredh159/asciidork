use bumpalo::collections::String;
use regex::Regex;

use crate::ast::*;
use crate::block::Block;
use crate::token::TokenKind::*;
use crate::Parser;

impl<'bmp, 'src> Parser<'bmp, 'src> {
  pub(super) fn parse_revision_line(
    &self,
    block: &mut Block,
    revision: &mut Option<Revision<'bmp>>,
  ) {
    let Some(line) = block.current_line() else {
      return;
    };

    if !line.current_is(Word) {
      return;
    }

    // https://regexr.com/7mbsk
    let pattern = r"^([^\s,:]+)(?:,\s*([^\s:]+))?(?::\s*(.+))?$";
    let re = Regex::new(pattern).unwrap();
    let Some(captures) = re.captures(line.src) else {
      return;
    };

    let raw_version = captures.get(1).unwrap().as_str();
    if !raw_version.chars().any(|c| c.is_ascii_digit()) {
      return;
    }

    let vre = Regex::new(r"\d.*$").unwrap();
    let version = vre.captures(raw_version).unwrap().get(0).unwrap().as_str();
    let version = String::from_str_in(version, self.bump);

    // only revision, must start with `v` then digit
    if captures.get(2).is_none() && captures.get(3).is_none() {
      if Regex::new(r"^v(\d[^\s]+)$").unwrap().is_match(raw_version) {
        *revision = Some(Revision { version, date: None, remark: None });
        block.consume_current();
      }
      return;
    }

    // version and remark
    if captures.get(2).is_none() && captures.get(3).is_some() {
      let remark = captures.get(3).unwrap().as_str();
      *revision = Some(Revision {
        version,
        date: None,
        remark: Some(String::from_str_in(remark, self.bump)),
      });
      block.consume_current();
      return;
    }

    // version and only date
    if captures.get(2).is_some() && captures.get(3).is_none() {
      let date = captures.get(2).unwrap().as_str();
      *revision = Some(Revision {
        version,
        date: Some(String::from_str_in(date, self.bump)),
        remark: None,
      });
      block.consume_current();
      return;
    }

    let date = captures.get(2).unwrap().as_str();
    let remark = captures.get(3).unwrap().as_str();
    *revision = Some(Revision {
      version,
      date: Some(String::from_str_in(date, self.bump)),
      remark: Some(String::from_str_in(remark, self.bump)),
    });
    block.consume_current();
  }
}

// tests

#[cfg(test)]
mod tests {
  use super::*;

  macro_rules! s {
    (in $bump:expr;$s:expr) => {
      bumpalo::collections::String::from_str_in($s, $bump)
    };
  }

  #[test]
  fn test_parse_revision_lines() {
    let b = &bumpalo::Bump::new();
    let cases = vec![
      ("foobar", None),
      (
        "v7.5",
        Some(Revision {
          version: s!(in b; "7.5"),
          date: None,
          remark: None,
        }),
      ),
      (
        "v7.5, 1-29-2020",
        Some(Revision {
          version: s!(in b; "7.5"),
          date: Some(s!(in b; "1-29-2020")),
          remark: None,
        }),
      ),
      (
        "LPR55, 1-29-2020",
        Some(Revision {
          version: s!(in b; "55"),
          date: Some(s!(in b; "1-29-2020")),
          remark: None,
        }),
      ),
      (
        "7.5, 1-29-2020",
        Some(Revision {
          version: s!(in b; "7.5"),
          date: Some(s!(in b; "1-29-2020")),
          remark: None,
        }),
      ),
      (
        "7.5: A new analysis",
        Some(Revision {
          version: s!(in b; "7.5"),
          date: None,
          remark: Some(s!(in b; "A new analysis")),
        }),
      ),
      (
        "v7.5, 1-29-2020: A new analysis",
        Some(Revision {
          version: s!(in b; "7.5"),
          date: Some(s!(in b; "1-29-2020")),
          remark: Some(s!(in b; "A new analysis")),
        }),
      ),
      ("v7.5 1-29-2020 A new analysis", None),
    ];

    for (input, expected) in cases {
      let mut parser = crate::Parser::new(b, input);
      let mut block = parser.read_block().unwrap();
      let mut revision = None;
      parser.parse_revision_line(&mut block, &mut revision);
      assert_eq!(revision, expected);
    }
  }
}
