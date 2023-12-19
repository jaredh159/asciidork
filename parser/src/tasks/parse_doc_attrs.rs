use std::collections::HashMap;

use regex::Regex;

use crate::prelude::*;

impl<'bmp, 'src> Parser<'bmp, 'src> {
  pub(super) fn parse_doc_attrs(
    &self,
    lines: &mut ContiguousLines<'bmp, 'src>,
    attrs: &mut HashMap<String<'bmp>, String<'bmp>>,
  ) -> Result<()> {
    while let Some((key, value)) = self.parse_doc_attr(lines)? {
      attrs.insert(key, value);
    }
    Ok(())
  }

  fn parse_doc_attr(
    &self,
    lines: &mut ContiguousLines,
  ) -> Result<Option<(String<'bmp>, String<'bmp>)>> {
    let Some(line) = lines.current() else {
      return Ok(None);
    };

    // todo: optimize by not recompiling this regex every time
    let re = Regex::new(r"^:([^\s:]+):\s*([^\s].*)?$").unwrap();
    let Some(captures) = re.captures(line.src) else {
      return Ok(None);
    };

    lines.consume_current();
    let key = String::from_str_in(captures.get(1).unwrap().as_str(), self.bump);
    let value = String::from_str_in(captures.get(2).map_or("", |m| m.as_str()), self.bump);
    Ok(Some((key, value)))
  }
}

#[cfg(test)]
mod tests {
  use crate::test::*;

  #[test]
  fn test_parse_doc_attr() {
    let b = &bumpalo::Bump::new();
    let cases = vec![
      (":foo: bar", ("foo", "bar")),
      (":foo:", ("foo", "")),
      (":foo-bar: baz, rofl, lol", ("foo-bar", "baz, rofl, lol")),
    ];
    for (input, authors) in cases {
      let mut parser = crate::Parser::new(b, input);
      let mut block = parser.read_lines().unwrap();
      let attr = parser.parse_doc_attr(&mut block).unwrap().unwrap();
      assert_eq!(attr, (s!(in b; authors.0), s!(in b; authors.1)));
    }
  }
}
