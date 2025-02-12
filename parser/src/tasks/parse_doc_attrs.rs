use std::borrow::Cow;

use crate::internal::*;

impl<'arena> Parser<'arena> {
  pub(super) fn parse_doc_attrs(&mut self, lines: &mut ContiguousLines<'arena>) -> Result<()> {
    lines.discard_leading_comment_lines();
    while let Some((key, value, _)) = self.parse_doc_attr(lines)? {
      if key == "doctype" {
        if let AttrValue::String(s) = &value {
          match s.as_str().parse::<DocType>() {
            Ok(doc_type) => self.document.meta.set_doctype(doc_type),
            Err(err) => self.err_doc_attr(":doctype:", err)?,
          }
        } else {
          self.err_doc_attr(":!doctype:", "".parse::<DocType>().err().unwrap())?;
        }
      } else if let Err(err) = self.document.meta.insert_header_attr(&key, value) {
        self.err_doc_attr(format!(":{}:", key), err)?;
      }
      lines.discard_leading_comment_lines();
    }
    Ok(())
  }

  pub(super) fn parse_doc_attr(
    &mut self,
    lines: &mut ContiguousLines<'arena>,
  ) -> Result<Option<(String, AttrValue, u32)>> {
    let Some(line) = lines.current() else {
      return Ok(None);
    };

    let src = line.reassemble_src();
    let Some(captures) = regx::ATTR_DECL.captures(&src) else {
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
        let loc = line.loc().unwrap();
        self.err_at(
          "Cannot unset attr with `!` AND provide value",
          loc.incr_start().adding_to_end(re_match.end() as u32),
        )?;
      }

      let joined = self.join_wrapped_value(re_match.as_str(), lines);
      let value = self.replace_attr_vals(&joined);
      AttrValue::String(value.to_string())
    } else {
      AttrValue::Bool(!is_negated)
    };

    if key == "leveloffset" {
      Parser::adjust_leveloffset(&mut self.ctx.leveloffset, &attr);
    }

    Ok(Some((
      key.to_string(),
      attr,
      line.last_location().unwrap().end,
    )))
  }

  pub(crate) fn replace_attr_vals<'h>(&self, haystack: &'h str) -> Cow<'h, str> {
    regx::ATTR_VAL_REPLACE.replace_all(haystack, |caps: &regex::Captures| {
      if let Some(AttrValue::String(replace)) =
        self.document.meta.get(caps.get(1).unwrap().as_str())
      {
        replace
      } else {
        ""
      }
    })
  }

  fn join_wrapped_value(
    &self,
    mut first_line_src: &str,
    lines: &mut ContiguousLines<'arena>,
  ) -> BumpString<'arena> {
    let has_continuation = if first_line_src.ends_with(" \\") {
      first_line_src = &first_line_src[..first_line_src.len() - 2];
      true
    } else {
      false
    };

    let mut wrapped = self.string(first_line_src);
    if lines.is_empty() || !has_continuation {
      return wrapped;
    }

    while !lines.is_empty() {
      wrapped.push(' ');
      let next_line = lines.consume_current().unwrap();
      let next_line_src = next_line.reassemble_src();
      if next_line_src.ends_with(" \\") {
        wrapped.push_str(&next_line_src[..next_line_src.len() - 2]);
      } else {
        wrapped.push_str(&next_line_src);
        break;
      }
    }
    wrapped
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use test_utils::*;

  #[test]
  fn test_leveloffset() {
    let cases = vec![
      (0, AttrValue::String("1".into()), 1),
      (0, AttrValue::String("+1".into()), 1),
      (0, AttrValue::String("+4".into()), 4),
      (1, AttrValue::String("+1".into()), 2),
      (2, AttrValue::String("-1".into()), 1),
      (2, AttrValue::String("-6".into()), -4),
      (0, AttrValue::Bool(false), 0),
      (1, AttrValue::Bool(false), 0),
      (4, AttrValue::Bool(false), 0),
    ];
    for (mut initial, attr_value, expected) in cases {
      Parser::adjust_leveloffset(&mut initial, &attr_value);
      assert_eq!(initial, expected);
    }
  }

  #[test]
  fn test_parse_doc_attr() {
    let cases = vec![
      (":foo: bar", ("foo", "bar".into())),
      (":foo:", ("foo", true.into())),
      (":!foo:", ("foo", false.into())),
      (":foo!:", ("foo", false.into())),
      (":foo: {custom}-bar", ("foo", "value-bar".into())),
      (":foo: {custom}-bar-{baz}", ("foo", "value-bar-qux".into())),
      (
        ":foo-bar: baz, rofl, lol",
        ("foo-bar", "baz, rofl, lol".into()),
      ),
      (":foo: bar \\\nand baz", ("foo", "bar and baz".into())),
      (
        ":foo: bar \\\nand baz \\\nand qux",
        ("foo", "bar and baz and qux".into()),
      ),
      (":foo: bar \\\n", ("foo", "bar".into())),
    ];
    for (input, (expected_key, expected_val)) in cases {
      let mut parser = test_parser!(input);
      parser
        .document
        .meta
        .insert_doc_attr("custom", "value")
        .unwrap();
      parser.document.meta.insert_doc_attr("baz", "qux").unwrap();
      let mut block = parser.read_lines().unwrap().unwrap();
      let (key, value, _) = parser.parse_doc_attr(&mut block).unwrap().unwrap();
      assert_eq!(&key, expected_key);
      assert_eq!(value, expected_val);
    }
  }

  assert_error!(
    test_parse_doc_attr_error_str,
    adoc! {"
      :doctype: bad

      para
    "},
    error! {"
       --> test.adoc:1:1
        |
      1 | :doctype: bad
        | ^^^^^^^^^^^^^ Invalid doctype: expected `article`, `book`, `manpage`, or `inline`
    "}
  );

  assert_error!(
    test_parse_doc_attr_error_unset,
    adoc! {"
      :!doctype:

      para
    "},
    error! {"
       --> test.adoc:1:1
        |
      1 | :!doctype:
        | ^^^^^^^^^^ Invalid doctype: expected `article`, `book`, `manpage`, or `inline`
    "}
  );

  assert_error!(
    doc_attr_error_invalid,
    adoc! {"
      :doctype: article
      :chapter-refsig: Capitulo

      para
    "},
    error! {"
       --> test.adoc:2:1
        |
      2 | :chapter-refsig: Capitulo
        | ^^^^^^^^^^^^^^^^^^^^^^^^^ Attribute `chapter-refsig` may only be set when doctype is `book`
    "}
  );
}
