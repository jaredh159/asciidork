use super::Result;
use crate::ast;
use crate::parse::Parser;

impl Parser {
  pub(super) fn parse_block(&mut self) -> Result<Option<ast::Block>> {
    // parse block attr list `[...]`

    // if it starts a section, delegate somewhere else?
    //   --> return self.parse_section()

    // is it some kind of compound, delimited block?
    //   --> return self.parse_X()

    return self.parse_paragraph();
  }

  fn parse_paragraph(&mut self) -> Result<Option<ast::Block>> {
    let Some(block) = self.read_block() else {
      return Ok(None);
    };
    Ok(Some(ast::Block {
      context: ast::BlockContext::Paragraph(self.parse_inlines(block)),
    }))
  }
}

// tests

#[cfg(test)]
mod tests {
  use super::*;
  use crate::ast::*;
  use crate::t::*;

  #[test]
  fn test_parse_simple_block() {
    let mut parser = Parser::from("hello mamma,\nhello papa\n\n");
    let block = parser.parse_block().unwrap().unwrap();
    assert_eq!(
      block,
      Block {
        context: BlockContext::Paragraph(vec![t("hello mamma, hello papa")]),
      }
    )
  }
}
