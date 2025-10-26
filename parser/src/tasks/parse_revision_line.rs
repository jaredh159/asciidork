use regex::Regex;

use crate::internal::*;
use crate::variants::token::*;

impl Parser<'_> {
  pub(super) fn parse_revision_line(&mut self, lines: &mut ContiguousLines) -> Option<u32> {
    let line = lines.current()?;
    if !line.current_is(Word) && !line.current_is(Digits) {
      return None;
    }

    // https://regexr.com/7mbsk
    let pattern = r"^([^\s,:]+)(?:,\s*([^:]+?))?(?::\s*(.+))?$";
    let re = Regex::new(pattern).unwrap();
    let src = line.reassemble_src();
    let captures = re.captures(&src)?;
    let raw_version = captures.get(1).unwrap().as_str();
    if !raw_version.chars().any(|c| c.is_ascii_digit()) {
      return None;
    }

    let end = lines.consume_current().unwrap().last_loc().unwrap().end;
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
        self
          .document
          .meta
          .insert_header_attr("revnumber", version)
          .unwrap();
      }
      return Some(end);
    }
    self
      .document
      .meta
      .insert_header_attr("revnumber", version)
      .unwrap();

    // version and remark
    if captures.get(2).is_none() && captures.get(3).is_some() {
      let remark = captures.get(3).unwrap().as_str().to_string();
      self
        .document
        .meta
        .insert_header_attr("revremark", remark)
        .unwrap();
      return Some(end);
    }

    // version and only date
    if captures.get(2).is_some() && captures.get(3).is_none() {
      let date = captures.get(2).unwrap().as_str().to_string();
      self
        .document
        .meta
        .insert_header_attr("revdate", date)
        .unwrap();
      return Some(end);
    }

    let date = captures.get(2).unwrap().as_str().to_string();
    let remark = captures.get(3).unwrap().as_str().to_string();
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
    Some(end)
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
      (
        "2.9, October 31, 2021: Fall incarnation",
        Some("2.9"),
        Some("October 31, 2021"),
        Some("Fall incarnation"),
      ),
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
