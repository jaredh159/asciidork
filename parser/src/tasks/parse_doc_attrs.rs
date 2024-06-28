use lazy_static::lazy_static;
use regex::Regex;

use crate::internal::*;

impl<'arena> Parser<'arena> {
  pub(super) fn parse_doc_attrs(&mut self, lines: &mut ContiguousLines<'arena>) -> Result<()> {
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
    }
    Ok(())
  }

  pub(super) fn parse_doc_attr(
    &self,
    lines: &mut ContiguousLines<'arena>,
  ) -> Result<Option<(String, AttrValue, usize)>> {
    let Some(line) = lines.current() else {
      return Ok(None);
    };

    let src = line.reassemble_src();
    let Some(captures) = ATTR_RE.captures(&src) else {
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
          self.document.meta.get(caps.get(1).unwrap().as_str())
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
    mut first_line_src: &str,
    lines: &mut ContiguousLines<'arena>,
  ) -> BumpString<'arena> {
    let has_continuation = if first_line_src.ends_with(" \\") {
      first_line_src = &first_line_src[..first_line_src.len() - 2];
      true
    } else {
      false
    };

    let mut wrapped = BumpString::from_str_in(first_line_src, self.bump);
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
      let mut parser = crate::Parser::from_str(input, b);
      parser
        .document
        .meta
        .insert_doc_attr("custom", "value")
        .unwrap();
      parser.document.meta.insert_doc_attr("baz", "qux").unwrap();
      let mut block = parser.read_lines().unwrap();
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
      1: :doctype: bad
         ^^^^^^^^^^^^^ Invalid doctype: expected `article`, `book`, `manpage`, or `inline`
    "}
  );

  assert_error!(
    test_parse_doc_attr_error_unset,
    adoc! {"
      :!doctype:

      para
    "},
    error! {"
      1: :!doctype:
         ^^^^^^^^^^ Invalid doctype: expected `article`, `book`, `manpage`, or `inline`
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
      2: :chapter-refsig: Capitulo
         ^^^^^^^^^^^^^^^^^^^^^^^^^ Attribute `chapter-refsig` may only be set when doctype is `book`
    "}
  );
}
