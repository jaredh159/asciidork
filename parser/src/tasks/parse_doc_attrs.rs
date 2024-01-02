use lazy_static::lazy_static;
use regex::Regex;

use crate::internal::*;

impl<'bmp, 'src> Parser<'bmp, 'src> {
  pub(super) fn parse_doc_attrs(
    &self,
    lines: &mut ContiguousLines<'bmp, 'src>,
    attrs: &mut AttrEntries,
  ) -> Result<()> {
    while let Some((key, value)) = self.parse_doc_attr(lines, attrs)? {
      attrs.insert(key, value);
    }
    Ok(())
  }

  fn parse_doc_attr(
    &self,
    lines: &mut ContiguousLines,
    attrs: &mut AttrEntries,
  ) -> Result<Option<(StdString, AttrEntry)>> {
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
        let start = line.location().unwrap().start + re_match.start();
        self.err_at(
          "Cannot unset attr with `!` AND provide value",
          start,
          start + re_match.len(),
        )?;
      }

      let value = SUBS_RE.replace_all(re_match.as_str(), |caps: &regex::Captures| {
        println!("caps: {:?}", caps);
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

    Ok(Some((key.to_string(), attr)))
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
    ];
    for (input, (key, value)) in cases {
      let mut existing = AttrEntries::new();
      existing.insert("custom".to_string(), AttrEntry::String("value".to_string()));
      existing.insert("baz".to_string(), AttrEntry::String("qux".to_string()));
      let mut parser = crate::Parser::new(b, input);
      let mut block = parser.read_lines().unwrap();
      let attr = parser
        .parse_doc_attr(&mut block, &mut existing)
        .unwrap()
        .unwrap();
      assert_eq!(attr, (key.to_string(), value));
    }
  }
}
