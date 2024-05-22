use regex::Regex;

use crate::internal::*;
use crate::variants::token::*;

impl<'bmp, 'src> Parser<'bmp, 'src> {
  pub(super) fn parse_revision_line(
    &self,
    lines: &mut ContiguousLines,
  ) -> Option<(String, Option<String>, Option<String>)> {
    let Some(line) = lines.current() else {
      return None;
    };

    if !line.current_is(Word) && !line.current_is(Digits) {
      return None;
    }

    // https://regexr.com/7mbsk
    let pattern = r"^([^\s,:]+)(?:,\s*([^\s:]+))?(?::\s*(.+))?$";
    let re = Regex::new(pattern).unwrap();
    let Some(captures) = re.captures(line.src) else {
      return None;
    };

    let raw_version = captures.get(1).unwrap().as_str();
    if !raw_version.chars().any(|c| c.is_ascii_digit()) {
      return None;
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
        return Some((version, None, None));
      }
      return None;
    }

    // version and remark
    if captures.get(2).is_none() && captures.get(3).is_some() {
      let remark = captures.get(3).unwrap().as_str().to_string();
      lines.consume_current();
      return Some((version, None, Some(remark)));
    }

    // version and only date
    if captures.get(2).is_some() && captures.get(3).is_none() {
      let date = captures.get(2).unwrap().as_str().to_string();
      lines.consume_current();
      return Some((version, Some(date), None));
    }

    let date = captures.get(2).unwrap().as_str().to_string();
    let remark = captures.get(3).unwrap().as_str().to_string();
    lines.consume_current();
    Some((version, Some(date), Some(remark)))
  }
}

// tests

#[cfg(test)]
mod tests {

  #[test]
  fn test_parse_revision_lines() {
    let cases = vec![
      ("foobar", None),
      ("v7.5", Some(("7.5".to_string(), None, None))),
      (
        "v7.5, 1-29-2020",
        Some(("7.5".to_string(), Some("1-29-2020".to_string()), None)),
      ),
      (
        "LPR55, 1-29-2020",
        Some(("55".to_string(), Some("1-29-2020".to_string()), None)),
      ),
      (
        "7.5, 1-29-2020",
        Some(("7.5".to_string(), Some("1-29-2020".to_string()), None)),
      ),
      (
        "7.5: A new analysis",
        Some(("7.5".to_string(), None, Some("A new analysis".to_string()))),
      ),
      (
        "v7.5, 1-29-2020: A new analysis",
        Some((
          "7.5".to_string(),
          Some("1-29-2020".to_string()),
          Some("A new analysis".to_string()),
        )),
      ),
      ("v7.5 1-29-2020 A new analysis", None),
    ];

    for (input, expected) in cases {
      let b = &bumpalo::Bump::new();
      let mut parser = crate::Parser::new(b, input);
      let mut block = parser.read_lines().unwrap();
      let revision = parser.parse_revision_line(&mut block);
      assert_eq!(revision, expected);
    }
  }
}
