use crate::ast;
use crate::ast::*;
use crate::block::Block as Lines;
use crate::token::TokenKind::*;
use crate::{Parser, Result};

impl<'bmp, 'src> Parser<'bmp, 'src> {
  pub(crate) fn parse_block(&mut self) -> Result<Option<ast::Block<'bmp>>> {
    let Some(block) = self.read_block() else {
      return Ok(None);
    };

    // parse block attr list `[...]`

    if block.is_block_macro() {
      let token = block.current_token().unwrap();
      match token.lexeme {
        "image:" => return Ok(Some(self.parse_image_block(block)?)),
        _ => todo!("unhandled block macro type: `{:?}`", token.lexeme),
      }
    }

    // if it starts a section, delegate somewhere else?
    //   --> return self.parse_section()

    // is it some kind of compound, delimited block?
    //   --> return self.parse_X()

    self.parse_paragraph(block)
  }

  fn parse_image_block(&mut self, mut lines: Lines<'bmp, 'src>) -> Result<ast::Block<'bmp>> {
    let mut line = lines.consume_current().unwrap();
    let start = line.location().unwrap().start;
    line.discard_assert(MacroName);
    line.discard_assert(Colon);
    let target = line.consume_macro_target(self.bump);
    let attrs = self.parse_attr_list(&mut line)?;
    Ok(ast::Block {
      loc: SourceLocation::new(start, attrs.loc.end),
      context: BlockContext::Image { target, attrs },
    })
  }

  fn parse_paragraph(&mut self, lines: Lines<'bmp, 'src>) -> Result<Option<ast::Block<'bmp>>> {
    let inlines = self.parse_inlines(lines)?;
    match (inlines.first(), inlines.last()) {
      (Some(first), Some(last)) => Ok(Some(ast::Block {
        loc: SourceLocation::new(first.loc.start, last.loc.end),
        context: BlockContext::Paragraph(inlines),
      })),
      _ => Ok(None),
    }
  }
}

// tests

#[cfg(test)]
mod tests {
  use crate::ast::*;
  use crate::parser::Parser;
  use crate::test::*;

  #[test]
  fn test_parse_simple_block() {
    let b = &Bump::new();
    let mut parser = Parser::new(b, "hello mamma,\nhello papa\n\n");
    let block = parser.parse_block().unwrap().unwrap();
    let expected = Block {
      context: BlockContext::Paragraph(b.vec([
        inode(Text(b.s("hello mamma,")), l(0, 12)),
        inode(JoiningNewline, l(12, 13)),
        inode(Text(b.s("hello papa")), l(13, 23)),
      ])),
      loc: l(0, 23),
    };
    assert_eq!(block, expected);
  }

  #[test]
  fn test_parse_image_block() {
    let b = &Bump::new();
    let mut parser = Parser::new(b, "image::name.png[]\n\n");
    let block = parser.parse_block().unwrap().unwrap();
    let expected = Block {
      context: BlockContext::Image {
        target: b.src("name.png", l(7, 15)),
        attrs: AttrList::new(l(15, 17), b),
      },
      loc: l(0, 17),
    };
    assert_eq!(block, expected);
  }
}
