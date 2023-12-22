use crate::ast::short::block::*;
use crate::prelude::*;
use crate::variants::token::*;

impl<'bmp, 'src> Parser<'bmp, 'src> {
  pub(crate) fn parse_block(&mut self) -> Result<Option<Block<'bmp>>> {
    let Some(mut lines) = self.read_lines() else {
      return Ok(None);
    };

    let meta = self.parse_block_metadata(&mut lines)?;
    let first_token = lines.current_token().unwrap();

    if lines.is_block_macro() {
      match first_token.lexeme {
        "image:" => return Ok(Some(self.parse_image_block(lines)?)),
        _ => todo!("block macro type: `{:?}`", first_token.lexeme),
      }
    }

    match first_token.kind {
      DelimiterLine if self.ctx.delimiter == first_token.to_delimeter() => {
        self.restore_lines(lines);
        return Ok(None);
      }
      DelimiterLine => {
        let delimiter = first_token.to_delimeter().unwrap();
        return self.parse_delimited_block(delimiter, lines, meta);
      }
      _ => {}
    }

    self.parse_paragraph(lines, meta)
  }

  fn parse_delimited_block(
    &mut self,
    delimiter: Delimiter,
    mut lines: ContiguousLines<'bmp, 'src>,
    meta: BlockMetadata<'bmp>,
  ) -> Result<Option<Block<'bmp>>> {
    let prev = self.ctx.delimiter;
    self.ctx.delimiter = Some(delimiter);
    let delimiter_token = lines.consume_current_token().unwrap();
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
      let end = blocks
        .last()
        .map(|b| b.loc.end)
        .unwrap_or(delimiter_token.loc.end);
      let message = format!(
        "unclosed delimiter block, expected `{}`, opened on line {}",
        delimiter_token.lexeme,
        self.lexer.line_number(delimiter_token.loc.start)
      );
      self.err_at(message, end, end + 1)?;
      end
    };
    self.ctx.delimiter = prev;
    let context = meta.block_style_or(Context::from(delimiter));
    Ok(Some(Block {
      title: meta.title,
      attrs: meta.attrs,
      content: Content::Compound(blocks),
      context,
      loc: SourceLocation::new(meta.start, end),
    }))
  }

  fn parse_image_block(&mut self, mut lines: ContiguousLines<'bmp, 'src>) -> Result<Block<'bmp>> {
    let mut line = lines.consume_current().unwrap();
    let start = line.location().unwrap().start;
    line.discard_assert(MacroName);
    line.discard_assert(Colon);
    let target = line.consume_macro_target(self.bump);
    let attrs = self.parse_attr_list(&mut line)?;
    Ok(Block {
      title: None,
      attrs: None,
      loc: SourceLocation::new(start, attrs.loc.end),
      context: Context::Image,
      content: Content::Empty(EmptyMetadata::Image { target, attrs }),
    })
  }

  fn parse_paragraph(
    &mut self,
    mut lines: ContiguousLines<'bmp, 'src>,
    meta: BlockMetadata<'bmp>,
  ) -> Result<Option<Block<'bmp>>> {
    let context = meta.paragraph_context(&mut lines);
    let inlines = self.parse_inlines(&mut lines)?;
    let result = match inlines.last() {
      Some(last) => Ok(Some(Block {
        title: meta.title,
        attrs: None,
        loc: SourceLocation::new(meta.start, last.loc.end),
        context,
        content: Content::Simple(inlines),
      })),
      _ => Ok(None),
    };
    self.restore_lines(lines);
    result
  }

  fn parse_block_metadata(
    &mut self,
    lines: &mut ContiguousLines<'bmp, 'src>,
  ) -> Result<BlockMetadata<'bmp>> {
    let start = lines.current_token().unwrap().loc.start;
    let mut attrs = None;
    let mut title = None;
    loop {
      match lines.current().unwrap() {
        line if line.is_block_title() => {
          let mut line = lines.consume_current().unwrap();
          line.discard_assert(Dot);
          title = Some(line.consume_to_string(self.bump));
        }
        line if line.is_attr_list() => {
          let mut line = lines.consume_current().unwrap();
          line.discard_assert(OpenBracket);
          attrs = Some(self.parse_attr_list(&mut line)?);
        }
        _ => break,
      }
    }
    Ok(BlockMetadata { attrs, title, start })
  }
}

struct BlockMetadata<'bmp> {
  title: Option<SourceString<'bmp>>,
  attrs: Option<AttrList<'bmp>>,
  start: usize,
}

impl BlockMetadata<'_> {
  fn block_style_or(&self, default: BlockContext) -> BlockContext {
    self
      .attrs
      .as_ref()
      .and_then(|attrs| attrs.block_style())
      .unwrap_or(default)
  }

  fn paragraph_context(&self, lines: &mut ContiguousLines) -> BlockContext {
    // line from block attrs takes precedence
    if let Some(block_style) = self.attrs.as_ref().and_then(|attrs| attrs.block_style()) {
      return block_style;
    }
    // handle inline admonitions, e.g. `TIP: never start a land war in asia`
    if lines
      .current()
      .map(|line| line.starts_with_seq(&[Word, Colon, Whitespace]))
      .unwrap_or(false)
    {
      let lexeme = lines.current_token().unwrap().lexeme;
      if let Some(context) = BlockContext::derive_admonition(lexeme) {
        let mut line = lines.consume_current().unwrap();
        line.discard(3); // word, colon, whitespace
        lines.restore(line);
        return context;
      }
    }
    // default to pararagraph
    BlockContext::Paragraph
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
      title: None,
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
  fn test_parse_block_titles() {
    let b = &Bump::new();
    let mut parser = Parser::new(b, ".My Title\nfoo\n\n");
    let block = parser.parse_block().unwrap().unwrap();
    let expected = Block {
      title: Some(b.src("My Title", l(1, 9))),
      attrs: None,
      context: Context::Paragraph,
      content: Content::Simple(b.vec([inode(Text(b.s("foo")), l(10, 13))])),
      loc: l(0, 13),
    };
    assert_eq!(block, expected);
  }

  #[test]
  fn test_parse_admonitions() {
    let b = &Bump::new();
    let mut parser = Parser::new(b, "TIP: foo\n\n");
    let block = parser.parse_block().unwrap().unwrap();
    let expected = Block {
      context: Context::AdmonitionTip,
      content: Content::Simple(b.vec([inode(Text(b.s("foo")), l(5, 8))])),
      loc: l(0, 8),
      ..Block::empty(b)
    };
    assert_eq!(block, expected);

    let mut parser = Parser::new(b, "[WARNING]\nTIP: foo\n\n");
    let block = parser.parse_block().unwrap().unwrap();
    let expected = Block {
      context: Context::AdmonitionWarning,
      content: Content::Simple(b.vec([
        inode(Text(b.s("TIP: foo")), l(10, 18)), // <-- attr list wins
      ])),
      loc: l(0, 18),
      ..Block::empty(b)
    };
    assert_eq!(block, expected);

    let mut parser = Parser::new(b, "[WARNING]\n====\nfoo\n====\n\n");
    let block = parser.parse_block().unwrap().unwrap();
    let expected = Block {
      title: None,
      attrs: Some(AttrList::positional("WARNING", l(1, 8), b)),
      context: Context::AdmonitionWarning, // <-- turns example into warning
      content: Content::Compound(b.vec([Block {
        context: Context::Paragraph,
        content: Content::Simple(b.vec([n_text("foo", 15, 18, b)])),
        loc: l(15, 18),
        ..Block::empty(b)
      }])),
      loc: l(0, 23),
    };
    assert_eq!(block, expected);

    let mut parser = Parser::new(b, "[CAUTION]\n====\nNOTE: foo\n====\n\n");
    let block = parser.parse_block().unwrap().unwrap();
    let expected = Block {
      title: None,
      attrs: Some(AttrList::positional("CAUTION", l(1, 8), b)),
      context: Context::AdmonitionCaution,
      content: Content::Compound(b.vec([Block {
        context: Context::AdmonitionNote,
        content: Content::Simple(b.vec([inode(Text(b.s("foo")), l(21, 24))])),
        loc: l(15, 24),
        ..Block::empty(b)
      }])),
      loc: l(0, 29),
    };
    assert_eq!(block, expected);
  }

  #[test]
  fn test_parse_image_block() {
    let b = &Bump::new();
    let mut parser = Parser::new(b, "image::name.png[]\n\n");
    let block = parser.parse_block().unwrap().unwrap();
    let expected = Block {
      context: Context::Image,
      content: Content::Empty(EmptyMetadata::Image {
        target: b.src("name.png", l(7, 15)),
        attrs: AttrList::new(l(15, 17), b),
      }),
      loc: l(0, 17),
      ..Block::empty(b)
    };
    assert_eq!(block, expected);
  }

  #[test]
  fn test_parse_delimited_open_block() {
    let b = &Bump::new();
    let mut parser = Parser::new(b, "--\nfoo\n--\n\n");
    let block = parser.parse_block().unwrap().unwrap();
    let expected = Block {
      context: Context::Open,
      content: Content::Compound(b.vec([Block {
        title: None,
        attrs: None,
        context: Context::Paragraph,
        content: Content::Simple(b.vec([n_text("foo", 3, 6, b)])),
        loc: l(3, 6),
      }])),
      loc: l(0, 9),
      ..Block::empty(b)
    };
    assert_eq!(block, expected);
  }

  #[test]
  fn test_parse_delimited_example_block() {
    let b = &Bump::new();
    let mut parser = Parser::new(b, "====\nfoo\n====\n\n");
    let block = parser.parse_block().unwrap().unwrap();
    let expected = Block {
      context: Context::Example,
      content: Content::Compound(b.vec([Block {
        context: Context::Paragraph,
        content: Content::Simple(b.vec([n_text("foo", 5, 8, b)])),
        loc: l(5, 8),
        ..Block::empty(b)
      }])),
      loc: l(0, 13),
      ..Block::empty(b)
    };
    assert_eq!(block, expected);
  }

  #[test]
  fn test_undelimited_sidebar() {
    let b = &Bump::new();
    let mut parser = Parser::new(b, "[sidebar]\nfoo\n\n");
    let block = parser.parse_block().unwrap().unwrap();
    let expected = Block {
      context: Context::Sidebar,
      content: Content::Simple(b.vec([n_text("foo", 10, 13, b)])),
      loc: l(0, 13),
      ..Block::empty(b)
    };
    assert_eq!(block, expected);
  }

  #[test]
  fn test_parse_empty_delimited_block() {
    let b = &Bump::new();
    let mut parser = Parser::new(b, "--\n--\n\n");
    let block = parser.parse_block().unwrap().unwrap();
    let expected = Block {
      context: Context::Open,
      content: Content::Compound(b.vec([])),
      loc: l(0, 5),
      ..Block::empty(b)
    };
    assert_eq!(block, expected);
  }

  #[test]
  fn test_parse_delimited_sidebar_block() {
    let b = &Bump::new();
    let mut parser = Parser::new(b, "****\nfoo\n****\n\n");
    let block = parser.parse_block().unwrap().unwrap();
    let expected = Block {
      context: Context::Sidebar,
      content: Content::Compound(b.vec([Block {
        context: Context::Paragraph,
        content: Content::Simple(b.vec([n_text("foo", 5, 8, b)])),
        loc: l(5, 8),
        ..Block::empty(b)
      }])),
      loc: l(0, 13),
      ..Block::empty(b)
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
      context: Context::Sidebar,
      content: Content::Compound(b.vec([Block {
        context: Context::Open,
        content: Content::Compound(b.vec([Block {
          context: Context::Paragraph,
          content: Content::Simple(b.vec([n_text("foo", 8, 11, b)])),
          loc: l(8, 11),
          ..Block::empty(b)
        }])),
        loc: l(5, 14),
        ..Block::empty(b)
      }])),
      loc: l(0, 19),
      ..Block::empty(b)
    };
    assert_eq!(block, expected);

    let input = "
****

.Bar
--

foo


--

****";
    let mut parser = Parser::new(b, input.trim());
    let block = parser.parse_block().unwrap().unwrap();
    let expected = Block {
      context: Context::Sidebar,
      content: Content::Compound(b.vec([Block {
        title: Some(b.src("Bar", l(7, 10))),
        context: Context::Open,
        content: Content::Compound(b.vec([Block {
          context: Context::Paragraph,
          content: Content::Simple(b.vec([n_text("foo", 15, 18, b)])),
          loc: l(15, 18),
          ..Block::empty(b)
        }])),
        loc: l(6, 23),
        ..Block::empty(b)
      }])),
      loc: l(0, 29),
      ..Block::empty(b)
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
      context: Context::Sidebar,
      content: Content::Compound(b.vec([
        Block {
          context: Context::Paragraph,
          content: Content::Simple(b.vec([para_1_txt])),
          loc: l(5, 40),
          ..Block::empty(b)
        },
        Block {
          context: Context::Image,
          content: Content::Empty(EmptyMetadata::Image {
            target: b.src("name.png", l(49, 57)),
            attrs: AttrList::new(l(57, 59), b),
          }),
          loc: l(42, 59),
          ..Block::empty(b)
        },
        Block {
          context: Context::Paragraph,
          content: Content::Simple(b.vec([para_2_txt])),
          loc: l(61, 103),
          ..Block::empty(b)
        },
      ])),
      loc: l(0, 108),
      ..Block::empty(b)
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
