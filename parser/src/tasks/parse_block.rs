use crate::ast::short::block::*;
use crate::prelude::*;
use crate::variants::token::*;

impl<'bmp, 'src> Parser<'bmp, 'src> {
  pub(crate) fn parse_block(&mut self) -> Result<Option<Block<'bmp>>> {
    let Some(block) = self.read_lines() else {
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
      DelimiterLine if self.ctx.delimiter == first_token.to_delimeter() => {
        self.restore_lines(block);
        return Ok(None);
      }
      DelimiterLine => {
        let delimiter = first_token.to_delimeter().unwrap();
        return self.parse_delimited_block(delimiter, block);
      }
      _ => {}
    }

    self.parse_paragraph(block)
  }

  fn parse_delimited_block(
    &mut self,
    delimiter: Delimiter,
    mut lines: ContiguousLines<'bmp, 'src>,
  ) -> Result<Option<Block<'bmp>>> {
    let prev = self.ctx.delimiter;
    self.ctx.delimiter = Some(delimiter);
    let start = lines.consume_current_token().unwrap();
    self.restore_lines(lines);
    let mut blocks = Vec::new_in(self.bump);
    while let Some(inner) = self.parse_block()? {
      blocks.push(inner);
    }
    let end = if let Some(mut block) = self.read_lines() {
      let token = block.consume_current_token().unwrap();
      debug_assert!(token.is(DelimiterLine));
      self.restore_lines(block);
      token.loc.end
    } else {
      let end = blocks.last().map(|b| b.loc.end).unwrap_or(start.loc.end);
      let message = format!(
        "unclosed delimiter block, expected `{}`, opened on line {}",
        start.lexeme,
        self.lexer.line_number(start.loc.start)
      );
      self.err_at(message, end, end + 1)?;
      end
    };
    self.ctx.delimiter = prev;
    return Ok(Some(Block {
      attrs: None,
      content: Content::Compound(blocks),
      context: Context::from(delimiter),
      loc: SourceLocation::new(start.loc.start, end),
    }));
  }

  fn parse_image_block(&mut self, mut lines: ContiguousLines<'bmp, 'src>) -> Result<Block<'bmp>> {
    let mut line = lines.consume_current().unwrap();
    let start = line.location().unwrap().start;
    line.discard_assert(MacroName);
    line.discard_assert(Colon);
    let target = line.consume_macro_target(self.bump);
    let attrs = self.parse_attr_list(&mut line)?;
    Ok(Block {
      attrs: None,
      loc: SourceLocation::new(start, attrs.loc.end),
      context: Context::Image,
      content: Content::Empty(EmptyMetadata::Image { target, attrs }),
    })
  }

  fn parse_paragraph(
    &mut self,
    mut lines: ContiguousLines<'bmp, 'src>,
  ) -> Result<Option<Block<'bmp>>> {
    let inlines = self.parse_inlines(&mut lines)?;
    let result = match (inlines.first(), inlines.last()) {
      (Some(first), Some(last)) => Ok(Some(Block {
        attrs: None,
        loc: SourceLocation::new(first.loc.start, last.loc.end),
        context: Context::Paragraph,
        content: Content::Simple(inlines),
      })),
      _ => Ok(None),
    };
    self.restore_lines(lines);
    result
  }
}

// tests

#[cfg(test)]
mod tests {
  use super::*;
  use crate::test::*;

  #[test]
  fn test_parse_simple_block() {
    let b = &Bump::new();
    let mut parser = Parser::new(b, "hello mamma,\nhello papa\n\n");
    let block = parser.parse_block().unwrap().unwrap();
    let expected = Block {
      attrs: None,
      context: Context::Paragraph,
      content: Content::Simple(b.vec([
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
      attrs: None,
      context: Context::Image,
      content: Content::Empty(EmptyMetadata::Image {
        target: b.src("name.png", l(7, 15)),
        attrs: AttrList::new(l(15, 17), b),
      }),
      loc: l(0, 17),
    };
    assert_eq!(block, expected);
  }

  #[test]
  fn test_parse_delimited_open_block() {
    let b = &Bump::new();
    let mut parser = Parser::new(b, "--\nfoo\n--\n\n");
    let block = parser.parse_block().unwrap().unwrap();
    let expected = Block {
      attrs: None,
      context: Context::Open,
      content: Content::Compound(b.vec([Block {
        attrs: None,
        context: Context::Paragraph,
        content: Content::Simple(b.vec([n_text("foo", 3, 6, b)])),
        loc: l(3, 6),
      }])),
      loc: l(0, 9),
    };
    assert_eq!(block, expected);
  }

  // #[test]
  fn test_undelimited_sidebar() {
    let b = &Bump::new();
    let mut parser = Parser::new(b, "[sidebar]\nfoo\n\n");
    let block = parser.parse_block().unwrap().unwrap();
    let expected = Block {
      attrs: None,
      context: Context::Sidebar,
      content: Content::Simple(b.vec([n_text("foo", 9, 12, b)])),
      loc: l(0, 5),
    };
    assert_eq!(block, expected);
  }

  #[test]
  fn test_parse_empty_delimited_block() {
    let b = &Bump::new();
    let mut parser = Parser::new(b, "--\n--\n\n");
    let block = parser.parse_block().unwrap().unwrap();
    let expected = Block {
      attrs: None,
      context: Context::Open,
      content: Content::Compound(b.vec([])),
      loc: l(0, 5),
    };
    assert_eq!(block, expected);
  }

  #[test]
  fn test_parse_delimited_sidebar_block() {
    let b = &Bump::new();
    let mut parser = Parser::new(b, "****\nfoo\n****\n\n");
    let block = parser.parse_block().unwrap().unwrap();
    let expected = Block {
      attrs: None,
      context: Context::Sidebar,
      content: Content::Compound(b.vec([Block {
        attrs: None,
        context: Context::Paragraph,
        content: Content::Simple(b.vec([n_text("foo", 5, 8, b)])),
        loc: l(5, 8),
      }])),
      loc: l(0, 13),
    };
    assert_eq!(block, expected);
  }

  #[test]
  fn test_nested_delimiter_blocks() {
    let b = &Bump::new();
    let input = "
****
--
foo
--
****";
    let mut parser = Parser::new(b, input.trim());
    let block = parser.parse_block().unwrap().unwrap();
    let expected = Block {
      attrs: None,
      context: Context::Sidebar,
      content: Content::Compound(b.vec([Block {
        attrs: None,
        context: Context::Open,
        content: Content::Compound(b.vec([Block {
          attrs: None,
          context: Context::Paragraph,
          content: Content::Simple(b.vec([n_text("foo", 8, 11, b)])),
          loc: l(8, 11),
        }])),
        loc: l(5, 14),
      }])),
      loc: l(0, 19),
    };
    assert_eq!(block, expected);

    let input = "
****

--

foo


--

****";
    let mut parser = Parser::new(b, input.trim());
    let block = parser.parse_block().unwrap().unwrap();
    let expected = Block {
      attrs: None,
      context: Context::Sidebar,
      content: Content::Compound(b.vec([Block {
        attrs: None,
        context: Context::Open,
        content: Content::Compound(b.vec([Block {
          attrs: None,
          context: Context::Paragraph,
          content: Content::Simple(b.vec([n_text("foo", 10, 13, b)])),
          loc: l(10, 13),
        }])),
        loc: l(6, 18),
      }])),
      loc: l(0, 24),
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
    let para_1_txt = n_text("This is content in a sidebar block.", 5, 40, b);
    let para_2_txt = n_text("This is more content in the sidebar block.", 61, 103, b);
    let expected = Block {
      attrs: None,
      context: Context::Sidebar,
      content: Content::Compound(b.vec([
        Block {
          attrs: None,
          context: Context::Paragraph,
          content: Content::Simple(b.vec([para_1_txt])),
          loc: l(5, 40),
        },
        Block {
          attrs: None,
          context: Context::Image,
          content: Content::Empty(EmptyMetadata::Image {
            target: b.src("name.png", l(49, 57)),
            attrs: AttrList::new(l(57, 59), b),
          }),
          loc: l(42, 59),
        },
        Block {
          attrs: None,
          context: Context::Paragraph,
          content: Content::Simple(b.vec([para_2_txt])),
          loc: l(61, 103),
        },
      ])),
      loc: l(0, 108),
    };
    assert_eq!(block, expected);
  }

  #[test]
  fn test_unclosed_delimited_block_err() {
    let b = &Bump::new();
    let mut parser = Parser::new(b, "--\nfoo\n\n");
    let err = parser.parse_block().err().unwrap();
    assert_eq!(
      err,
      Diagnostic {
        line_num: 2,
        line: "foo".to_string(),
        message: "unclosed delimiter block, expected `--`, opened on line 1".to_string(),
        underline_start: 3,
        underline_width: 1,
      }
    )
  }
}
