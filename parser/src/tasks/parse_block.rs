use crate::ast;
use crate::ast::*;
use crate::block::Block as LineBlock;
use crate::parser::Delimiter;
use crate::token::TokenKind::*;
use crate::utils::bump::*;
use crate::{Parser, Result};

impl<'bmp, 'src> Parser<'bmp, 'src> {
  pub(crate) fn parse_block(&mut self) -> Result<Option<ast::Block<'bmp>>> {
    let Some(mut block) = self.read_block() else {
      return Ok(None);
    };

    // parse block attr list `[...]`

    let Some(first_token) = block.current_token() else {
      return Ok(None);
    };

    if block.is_block_macro() {
      match first_token.lexeme {
        "image:" => return Ok(Some(self.parse_image_block(block)?)),
        _ => todo!("block macro type: `{:?}`", first_token.lexeme),
      }
    }

    match first_token.kind {
      DelimiterLine if self.ctx.delimiter.is_some() => {
        self.ctx.delimiter = None;
        self.restore_block(block);
        return Ok(None);
      }
      DelimiterLine => {
        let first_delimiter = block.consume_current().unwrap().consume_current().unwrap();
        let start = first_delimiter.loc.start;
        self.ctx.delimiter = Some(Delimiter::Sidebar);
        let mut blocks = Vec::new_in(self.bump);
        self.restore_block(block);
        while let Some(inner) = self.parse_block()? {
          blocks.push(inner);
        }
        let Some(mut block) = self.read_block() else {
          todo!("throw error, no end delimiter")
        };
        let mut line = block.consume_current().unwrap();
        debug_assert!(line.current_is(DelimiterLine));
        let end_delimiter = line.consume_current().unwrap();
        self.ctx.delimiter = None;
        return Ok(Some(ast::Block {
          context: BlockContext::Sidebar(blocks),
          loc: SourceLocation::new(start, end_delimiter.loc.end),
        }));
      }
      _ => {}
    }

    // if it starts a section, delegate somewhere else?
    //   --> return self.parse_section()

    // is it some kind of compound, delimited block?
    //   --> return self.parse_X()

    self.parse_paragraph(block)
  }

  fn parse_image_block(&mut self, mut lines: LineBlock<'bmp, 'src>) -> Result<ast::Block<'bmp>> {
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

  fn parse_paragraph(
    &mut self,
    mut line_block: LineBlock<'bmp, 'src>,
  ) -> Result<Option<ast::Block<'bmp>>> {
    let inlines = self.parse_inlines(&mut line_block)?;
    let result = match (inlines.first(), inlines.last()) {
      (Some(first), Some(last)) => Ok(Some(ast::Block {
        loc: SourceLocation::new(first.loc.start, last.loc.end),
        context: BlockContext::Paragraph(inlines),
      })),
      _ => Ok(None),
    };
    if !line_block.is_empty() {
      self.restore_block(line_block);
    }
    result
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

  #[test]
  fn test_parse_delimited_sidebar_block() {
    let b = &Bump::new();
    let mut parser = Parser::new(b, "****\nfoo\n****\n\n");
    let block = parser.parse_block().unwrap().unwrap();
    let expected = Block {
      context: BlockContext::Sidebar(b.vec([Block {
        context: BlockContext::Paragraph(b.vec([n_text("foo", 5, 8, b)])),
        loc: l(5, 8),
      }])),
      loc: l(0, 13),
    };
    assert_eq!(block, expected);
  }

  #[test]
  fn test_parse_multi_para_delimited_sidebar_block() {
    let b = &Bump::new();
    let input = "
****
This is content in a sidebar block.

image::name.png[]

This is more content in the sidebar block.
****
      ";
    let mut parser = Parser::new(b, input.trim());
    let block = parser.parse_block().unwrap().unwrap();
    let expected = Block {
      context: BlockContext::Sidebar(b.vec([
        Block {
          context: BlockContext::Paragraph(b.vec([n_text(
            "This is content in a sidebar block.",
            5,
            40,
            b,
          )])),
          loc: l(5, 40),
        },
        Block {
          context: BlockContext::Image {
            target: b.src("name.png", l(49, 57)),
            attrs: AttrList::new(l(57, 59), b),
          },
          loc: l(42, 59),
        },
        Block {
          context: BlockContext::Paragraph(b.vec([n_text(
            "This is more content in the sidebar block.",
            61,
            103,
            b,
          )])),
          loc: l(61, 103),
        },
      ])),
      loc: l(0, 108),
    };
    assert_eq!(block, expected);
  }
}
