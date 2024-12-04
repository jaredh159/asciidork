use regex::Regex;

use crate::internal::*;
use crate::variants::token::*;

impl Parser<'_> {
  pub(super) fn parse_revision_line(&mut self, lines: &mut ContiguousLines) {
    let Some(line) = lines.current() else {
      return;
    };

    if !line.current_is(Word) && !line.current_is(Digits) {
      return;
    }

    // https://regexr.com/7mbsk
    let pattern = r"^([^\s,:]+)(?:,\s*([^\s:]+))?(?::\s*(.+))?$";
    let re = Regex::new(pattern).unwrap();
    let src = line.reassemble_src();
    let Some(captures) = re.captures(&src) else {
      return;
    };

    let raw_version = captures.get(1).unwrap().as_str();
    if !raw_version.chars().any(|c| c.is_ascii_digit()) {
      return;
    }

    let vre = Regex::new(r"\d.*$").unwrap();
    let version = vre
      .captures(raw_version)
      .unwrap()
      .get(0)
      .unwrap()
      .as_str()
      .to_string();

    // only revision, must start with `v` then digit
    if captures.get(2).is_none() && captures.get(3).is_none() {
      if Regex::new(r"^v(\d[^\s]+)$").unwrap().is_match(raw_version) {
        lines.consume_current();
        self
          .document
          .meta
          .insert_header_attr("revnumber", version)
          .unwrap();
      }
      return;
    }
    self
      .document
      .meta
      .insert_header_attr("revnumber", version)
      .unwrap();

    // version and remark
    if captures.get(2).is_none() && captures.get(3).is_some() {
      let remark = captures.get(3).unwrap().as_str().to_string();
      lines.consume_current();
      self
        .document
        .meta
        .insert_header_attr("revremark", remark)
        .unwrap();
      return;
    }

    // version and only date
    if captures.get(2).is_some() && captures.get(3).is_none() {
      let date = captures.get(2).unwrap().as_str().to_string();
      lines.consume_current();
      self
        .document
        .meta
        .insert_header_attr("revdate", date)
        .unwrap();
      return;
    }

    let date = captures.get(2).unwrap().as_str().to_string();
    let remark = captures.get(3).unwrap().as_str().to_string();
    lines.consume_current();
    self
      .document
      .meta
      .insert_header_attr("revdate", date)
      .unwrap();
    self
      .document
      .meta
      .insert_header_attr("revremark", remark)
      .unwrap();
  }
}

// tests

#[cfg(test)]
mod tests {
  use super::*;
  use test_utils::*;

  #[test]
  fn test_parse_revision_lines() {
    let cases = vec![
      ("foobar", None, None, None),
      ("v7.5", Some("7.5"), None, None),
      ("v7.5, 1-29-2020", Some("7.5"), Some("1-29-2020"), None),
      ("LPR55, 1-29-2020", Some("55"), Some("1-29-2020"), None),
      ("7.5, 1-29-2020", Some("7.5"), Some("1-29-2020"), None),
      (
        "7.5: A new analysis",
        Some("7.5"),
        None,
        Some("A new analysis"),
      ),
      (
        "v7.5, 1-29-2020: A new analysis",
        Some("7.5"),
        Some("1-29-2020"),
        Some("A new analysis"),
      ),
      ("v7.5 1-29-2020 A new analysis", None, None, None),
    ];

    for (input, rev, date, remark) in cases {
      let mut parser = test_parser!(input);
      let mut block = parser.read_lines().unwrap().unwrap();
      parser.parse_revision_line(&mut block);
      assert_eq!(
        parser.document.meta.get("revnumber"),
        rev.map(|s| s.into()).as_ref()
      );
      assert_eq!(
        parser.document.meta.get("revdate"),
        date.map(|s| s.into()).as_ref()
      );
      assert_eq!(
        parser.document.meta.get("revremark"),
        remark.map(|s| s.into()).as_ref()
      );
    }
  }
}
