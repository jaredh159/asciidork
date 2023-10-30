use crate::ast;
use crate::{Parser, Result};

impl<'bmp, 'src> Parser<'bmp, 'src> {
  pub(crate) fn parse_block(&mut self) -> Result<Option<ast::Block<'bmp>>> {
    // parse block attr list `[...]`

    // if it starts a section, delegate somewhere else?
    //   --> return self.parse_section()

    // is it some kind of compound, delimited block?
    //   --> return self.parse_X()

    self.parse_paragraph()
  }

  fn parse_paragraph(&mut self) -> Result<Option<ast::Block<'bmp>>> {
    let Some(block) = self.read_block() else {
      return Ok(None);
    };
    Ok(Some(ast::Block {
      context: ast::BlockContext::Paragraph(self.parse_inlines(block)?),
    }))
  }
}

// tests

#[cfg(test)]
mod tests {
  use crate::ast::*;

  // todo, repeated
  macro_rules! s {
    (in $bump:expr; $s:expr) => {
      bumpalo::collections::String::from_str_in($s, $bump)
    };
  }

  #[test]
  fn test_parse_simple_block() {
    let b = &bumpalo::Bump::new();
    let mut parser = crate::Parser::new(b, "hello mamma,\nhello papa\n\n");
    let block = parser.parse_block().unwrap().unwrap();
    let expected = Block {
      context: BlockContext::Paragraph(
        bumpalo::vec![in b; Inline::Text(s!(in b; "hello mamma, hello papa"))],
      ),
    };
    assert_eq!(block, expected);
  }
}
