use std::collections::HashMap;

use bumpalo::collections::String;
use regex::Regex;

use crate::{block::Block, Parser, Result};

impl<'alloc, 'src> Parser<'alloc, 'src> {
  pub(super) fn parse_doc_attrs(
    &self,
    block: &mut Block<'alloc, 'src>,
    attrs: &mut HashMap<String<'alloc>, String<'alloc>>,
  ) -> Result<()> {
    while let Some((key, value)) = self.parse_doc_attr(block)? {
      attrs.insert(key, value);
    }
    Ok(())
  }

  fn parse_doc_attr(&self, block: &mut Block) -> Result<Option<(String<'alloc>, String<'alloc>)>> {
    let Some(line) = block.current_line() else {
      return Ok(None);
    };

    // todo: optmize by not recompiling this regex every time
    let re = Regex::new(r"^:([^\s:]+):\s*([^\s].*)?$").unwrap();
    let Some(captures) = re.captures(line.src) else {
      println!("no captures, {}", line.src);
      return Ok(None);
    };

    block.consume_current();
    let key = String::from_str_in(captures.get(1).unwrap().as_str(), self.allocator);
    let value = String::from_str_in(captures.get(2).map_or("", |m| m.as_str()), self.allocator);
    Ok(Some((key, value)))
  }
}

#[cfg(test)]
mod tests {
  macro_rules! s {
    (in $bump:expr;$s:expr) => {
      bumpalo::collections::String::from_str_in($s, $bump)
    };
  }

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
      let mut block = parser.read_block().unwrap();
      let attr = parser.parse_doc_attr(&mut block).unwrap().unwrap();
      assert_eq!(attr, (s!(in b; authors.0), s!(in b; authors.1)));
    }
  }
}
