use std::collections::HashMap;

use super::Result;
use crate::parse::Parser;
use crate::tok;
use crate::tok::TokenType::*;

impl Parser {
  pub(super) fn parse_doc_attrs(
    &self,
    block: &mut tok::Block,
    attrs: &mut HashMap<String, String>,
  ) -> Result<()> {
    while let Some((key, value)) = self.parse_doc_attr(block)? {
      attrs.insert(key, value);
    }
    Ok(())
  }

  fn parse_doc_attr(&self, block: &mut tok::Block) -> Result<Option<(String, String)>> {
    let Some(ref mut line) = block.consume_current() else {
      return Ok(None);
    };

    let expected = self.expect_each(
      [
        (Colon, "doc attr starting with `:`"),
        (Word, "doc attr name"),
        (Colon, "`:` to end doc attr"),
      ],
      line,
    )?;

    let Some([_, key, _]) = expected else {
      // restore block?
      return Ok(None);
    };

    line.consume_if(Whitespace);

    let mut value = String::new();
    while !line.is_empty() {
      value.push_str(self.lexeme_str(&line.consume_current().unwrap()));
    }

    Ok(Some((self.lexeme_string(&key), value)))
  }
}

#[cfg(test)]
mod tests {
  use crate::t::*;

  #[test]
  fn test_parse_doc_attr() {
    let cases = vec![
      (":foo: bar", ("foo", "bar")),
      (":foo:", ("foo", "")),
      (":foo-bar: baz, rofl, lol", ("foo-bar", "baz, rofl, lol")),
    ];
    for (input, authors) in cases {
      let (mut block, parser) = block_test(input);
      let attr = parser.parse_doc_attr(&mut block).unwrap().unwrap();
      assert_eq!(attr, (s(authors.0), s(authors.1)));
    }
  }
}
