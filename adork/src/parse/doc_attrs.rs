use std::collections::HashMap;
use std::io::BufRead;

use super::line::Line;
use super::Result;
use crate::parse::line_block::LineBlock;
use crate::parse::Parser;
use crate::token::TokenType::*;

impl<R: BufRead> Parser<R> {
  pub(super) fn parse_doc_attrs(
    &self,
    block: &mut LineBlock,
    attrs: &mut HashMap<String, String>,
  ) -> Result<()> {
    while let Some(mut line) = block.consume_current() {
      let (key, value) = self.parse_doc_attr(&mut line)?;
      attrs.insert(key, value);
    }
    Ok(())
  }

  fn parse_doc_attr(&self, line: &mut Line) -> Result<(String, String)> {
    line.consume_expecting(Colon, "doc attr starting with `:`")?;
    let key = self.lexeme_string(&line.consume_expecting(Word, "doc attr name")?);
    line.consume_expecting(Colon, "`:` to end doc attr")?;
    line.consume_if(Whitespace);

    let mut value = String::new();
    while !line.is_empty() {
      value.push_str(self.lexeme_str(&line.consume_current().unwrap()));
    }

    Ok((key, value))
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
      let (mut line, parser) = line_test(input);
      let attr = parser.parse_doc_attr(&mut line).unwrap();
      assert_eq!(attr, (s(authors.0), s(authors.1)));
    }
  }
}
