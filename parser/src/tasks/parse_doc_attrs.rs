use std::borrow::Cow;

use lazy_static::lazy_static;
use regex::Regex;

use crate::internal::*;

impl<'bmp, 'src> Parser<'bmp, 'src> {
  pub(super) fn parse_doc_attrs(&mut self, lines: &mut ContiguousLines<'bmp, 'src>) -> Result<()> {
    while let Some((key, value, _)) = self.parse_doc_attr(lines)? {
      if key == "doctype" {
        if let AttrValue::String(s) = &value {
          match s.as_str().parse::<DocType>() {
            Ok(doc_type) => self.document.set_type(doc_type),
            Err(err) => self.err_doc_attr(":doctype:", err)?,
          }
        } else {
          self.err_doc_attr(":!doctype:", "".parse::<DocType>().err().unwrap())?;
        }
      }
      self.document.attrs.insert(key, AttrEntry::new(value));
    }
    Ok(())
  }

  pub(super) fn parse_doc_attr(
    &self,
    lines: &mut ContiguousLines,
  ) -> Result<Option<(String, AttrValue, usize)>> {
    let Some(line) = lines.current() else {
      return Ok(None);
    };

    let Some(captures) = ATTR_RE.captures(line.src) else {
      return Ok(None);
    };

    let line = lines.consume_current().unwrap();

    let mut key = captures.get(1).unwrap().as_str();
    let is_negated = if key.starts_with('!') {
      key = &key[1..];
      true
    } else if key.ends_with('!') {
      key = &key[..key.len() - 1];
      true
    } else {
      false
    };

    let attr = if let Some(re_match) = captures.get(2) {
      if is_negated {
        let start = line.loc().unwrap().start + re_match.start();
        self.err_at(
          "Cannot unset attr with `!` AND provide value",
          start + 1,
          start + 1 + re_match.len(),
        )?;
      }

      let joined = self.join_wrapped_value(re_match.as_str(), lines);
      let value = SUBS_RE.replace_all(&joined, |caps: &regex::Captures| {
        if let Some(AttrValue::String(replace)) =
          self.document.attrs.get(caps.get(1).unwrap().as_str())
        {
          replace
        } else {
          ""
        }
      });
      AttrValue::String(value.to_string())
    } else {
      AttrValue::Bool(!is_negated)
    };

    Ok(Some((
      key.to_string(),
      attr,
      line.last_location().unwrap().end,
    )))
  }

  fn join_wrapped_value(
    &self,
    mut first_line_src: &'src str,
    lines: &mut ContiguousLines,
  ) -> Cow<str> {
    let has_continuation = if first_line_src.ends_with(" \\") {
      first_line_src = &first_line_src[..first_line_src.len() - 2];
      true
    } else {
      false
    };

    if lines.is_empty() || !has_continuation {
      return Cow::Borrowed(first_line_src);
    }

    let mut pieces = SmallVec::<[&str; 8]>::new();
    pieces.push(first_line_src);

    while !lines.is_empty() {
      let next_line = lines.consume_current().unwrap();
      if next_line.src.ends_with(" \\") {
        pieces.push(&next_line.src[..next_line.src.len() - 2]);
      } else {
        pieces.push(next_line.src);
        break;
      }
    }
    Cow::Owned(pieces.join(" "))
  }
}

lazy_static! {
  pub static ref ATTR_RE: Regex = Regex::new(r"^:([^\s:]+):\s*([^\s].*)?$").unwrap();
}

lazy_static! {
  pub static ref SUBS_RE: Regex = Regex::new(r"\{([^\s}]+)\}").unwrap();
}

#[cfg(test)]
mod tests {
  use super::*;
  use test_utils::{assert_eq, *};

  #[test]
  fn test_parse_doc_attr() {
    let b = &bumpalo::Bump::new();
    let cases = vec![
      (":foo: bar", ("foo", AttrValue::String("bar".to_string()))),
      (":foo:", ("foo", AttrValue::Bool(true))),
      (":!foo:", ("foo", AttrValue::Bool(false))),
      (":foo!:", ("foo", AttrValue::Bool(false))),
      (
        ":foo: {custom}-bar",
        ("foo", AttrValue::String("value-bar".to_string())),
      ),
      (
        ":foo: {custom}-bar-{baz}",
        ("foo", AttrValue::String("value-bar-qux".to_string())),
      ),
      (
        ":foo-bar: baz, rofl, lol",
        ("foo-bar", AttrValue::String("baz, rofl, lol".to_string())),
      ),
      (
        ":foo: bar \\\nand baz",
        ("foo", AttrValue::String("bar and baz".to_string())),
      ),
      (
        ":foo: bar \\\nand baz \\\nand qux",
        ("foo", AttrValue::String("bar and baz and qux".to_string())),
      ),
      (
        ":foo: bar \\\n",
        ("foo", AttrValue::String("bar".to_string())),
      ),
    ];
    for (input, (expected_key, expected_val)) in cases {
      let mut existing = AttrEntries::default();
      existing.insert(
        "custom".to_string(),
        AttrEntry::new(AttrValue::String("value".to_string())),
      );
      existing.insert(
        "baz".to_string(),
        AttrEntry::new(AttrValue::String("qux".to_string())),
      );
      let mut parser = crate::Parser::new(b, input);
      parser.document.attrs = existing;
      let mut block = parser.read_lines().unwrap();
      let (key, value, _) = parser.parse_doc_attr(&mut block).unwrap().unwrap();
      assert_eq!(&key, expected_key);
      assert_eq!(value, expected_val);
    }
  }

  test_error!(
    test_parse_doc_attr_error_str,
    adoc! {"
      :doctype: bad

      para
    "},
    error! {"
      1: :doctype: bad
         ^^^^^^^^^^^^^ Invalid doc type: expected `article`, `book`, `manpage`, or `inline`
    "}
  );

  test_error!(
    test_parse_doc_attr_error_unset,
    adoc! {"
      :!doctype:

      para
    "},
    error! {"
      1: :!doctype:
         ^^^^^^^^^^ Invalid doc type: expected `article`, `book`, `manpage`, or `inline`
    "}
  );
}
