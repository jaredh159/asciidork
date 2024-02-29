use std::borrow::Cow;

use lazy_static::lazy_static;
use regex::Regex;
use smallvec::SmallVec;

use crate::internal::*;

impl<'bmp, 'src> Parser<'bmp, 'src> {
  pub(super) fn parse_doc_attrs(
    &self,
    lines: &mut ContiguousLines<'bmp, 'src>,
    attrs: &mut AttrEntries,
  ) -> Result<()> {
    while let Some((key, value, _)) = self.parse_doc_attr(lines, attrs)? {
      attrs.insert(key, value);
    }
    Ok(())
  }

  pub(super) fn parse_doc_attr(
    &self,
    lines: &mut ContiguousLines,
    attrs: &mut AttrEntries,
  ) -> Result<Option<(String, AttrEntry, usize)>> {
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
          start,
          start + re_match.len(),
        )?;
      }

      let joined = self.join_wrapped_value(re_match.as_str(), lines);
      let value = SUBS_RE.replace_all(&joined, |caps: &regex::Captures| {
        if let Some(AttrEntry::String(replace)) = attrs.get(caps.get(1).unwrap().as_str()) {
          replace
        } else {
          ""
        }
      });
      AttrEntry::String(value.to_string())
    } else {
      AttrEntry::Bool(!is_negated)
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

  #[test]
  fn test_parse_doc_attr() {
    let b = &bumpalo::Bump::new();
    let cases = vec![
      (":foo: bar", ("foo", AttrEntry::String("bar".to_string()))),
      (":foo:", ("foo", AttrEntry::Bool(true))),
      (":!foo:", ("foo", AttrEntry::Bool(false))),
      (":foo!:", ("foo", AttrEntry::Bool(false))),
      (
        ":foo: {custom}-bar",
        ("foo", AttrEntry::String("value-bar".to_string())),
      ),
      (
        ":foo: {custom}-bar-{baz}",
        ("foo", AttrEntry::String("value-bar-qux".to_string())),
      ),
      (
        ":foo-bar: baz, rofl, lol",
        ("foo-bar", AttrEntry::String("baz, rofl, lol".to_string())),
      ),
      (
        ":foo: bar \\\nand baz",
        ("foo", AttrEntry::String("bar and baz".to_string())),
      ),
      (
        ":foo: bar \\\nand baz \\\nand qux",
        ("foo", AttrEntry::String("bar and baz and qux".to_string())),
      ),
      (
        ":foo: bar \\\n",
        ("foo", AttrEntry::String("bar".to_string())),
      ),
    ];
    for (input, (expected_key, expected_val)) in cases {
      let mut existing = AttrEntries::new();
      existing.insert("custom".to_string(), AttrEntry::String("value".to_string()));
      existing.insert("baz".to_string(), AttrEntry::String("qux".to_string()));
      let mut parser = crate::Parser::new(b, input);
      let mut block = parser.read_lines().unwrap();
      let (key, value, _) = parser
        .parse_doc_attr(&mut block, &mut existing)
        .unwrap()
        .unwrap();
      assert_eq!(&key, expected_key);
      assert_eq!(value, expected_val);
    }
  }
}
