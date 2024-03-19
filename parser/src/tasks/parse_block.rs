use crate::internal::*;
use crate::variants::token::*;
use ast::short::block::*;

impl<'bmp, 'src> Parser<'bmp, 'src> {
  pub(crate) fn parse_block(&mut self) -> Result<Option<Block<'bmp>>> {
    let Some(mut lines) = self.read_lines() else {
      return Ok(None);
    };

    if let Some(comment_block) = self.parse_comment_block(&mut lines) {
      return Ok(Some(comment_block));
    }

    let meta = self.parse_chunk_meta(&mut lines)?;
    if lines.starts_section(&meta) {
      self.restore_lines(lines);
      return Ok(None);
    }

    let first_token = lines.current_token().unwrap();

    if lines.is_block_macro() {
      match first_token.lexeme {
        "image:" => return Ok(Some(self.parse_image_block(lines, meta)?)),
        _ => todo!("block macro type: `{:?}`", first_token.lexeme),
      }
    } else if lines.starts_list() {
      return self.parse_list(lines, Some(meta)).map(Some);
    }

    match first_token.kind {
      DelimiterLine
        if self.ctx.delimiter.is_some() && self.ctx.delimiter == first_token.to_delimeter() =>
      {
        self.restore_lines(lines);
        return Ok(None);
      }
      DelimiterLine => {
        let delimiter = first_token.to_delimeter().unwrap();
        return self.parse_delimited_block(delimiter, lines, meta);
      }
      Colon => {
        let mut attr_entries = AttrEntries::new(); // TODO: this is a little weird...
        if let Some((key, value, end)) = self.parse_doc_attr(&mut lines, &mut attr_entries)? {
          return Ok(Some(Block {
            loc: SourceLocation::new(meta.start, end),
            meta,
            context: Context::DocumentAttributeDecl,
            content: Content::DocumentAttribute(key, value),
          }));
        }
      }
      _ => {}
    }

    if lines.is_quoted_paragraph() {
      self.parse_quoted_paragraph(lines, meta)
    } else {
      self.parse_paragraph(lines, meta)
    }
  }

  // important to represent these as an ast node because
  // they are the documented way to separate adjacent lists
  fn parse_comment_block(
    &mut self,
    lines: &mut ContiguousLines<'bmp, 'src>,
  ) -> Option<Block<'bmp>> {
    if lines.starts(CommentLine) {
      let start = lines.current_token().unwrap().loc.start;
      lines.consume_current();
      while lines.starts(CommentLine) {
        lines.consume_current();
      }
      if lines.is_empty() {
        return Some(Block {
          meta: ChunkMeta::empty(start),
          context: Context::Comment,
          content: Content::Empty(EmptyMetadata::None),
          loc: SourceLocation::new(start, self.loc().end),
        });
      }
    }
    None
  }

  fn parse_delimited_block(
    &mut self,
    delimiter: Delimiter,
    mut lines: ContiguousLines<'bmp, 'src>,
    meta: ChunkMeta<'bmp>,
  ) -> Result<Option<Block<'bmp>>> {
    let prev = self.ctx.delimiter;
    self.ctx.delimiter = Some(delimiter);
    let delimiter_token = lines.consume_current_token().unwrap();
    self.restore_lines(lines);
    let context = meta.block_style_or(Context::from(delimiter));
    let restore_subs = self.ctx.set_subs_for(context);

    // newlines have a different meaning in a listing/literal block, so we have to
    // manually gather all (including empty) lines until the end delimiter
    let content = if matches!(context, Context::Listing | Context::Literal) {
      let mut lines = self
        .read_lines_until(delimiter)
        .unwrap_or_else(|| ContiguousLines::new(bvec![in self.bump]));
      let content = Content::Simple(self.parse_inlines(&mut lines)?);
      self.restore_lines(lines);
      content
    } else {
      let mut blocks = BumpVec::new_in(self.bump);
      while let Some(inner) = self.parse_block()? {
        blocks.push(inner);
      }
      Content::Compound(blocks)
    };

    self.ctx.subs = restore_subs;
    let end = if let Some(mut block) = self.read_lines() {
      let token = block.consume_current_token().unwrap();
      debug_assert!(token.is(DelimiterLine));
      self.restore_lines(block);
      token.loc.end
    } else {
      let end = content.last_loc().unwrap_or(delimiter_token.loc).end;
      let message = format!(
        "unclosed delimiter block, expected `{}`, opened on line {}",
        delimiter_token.lexeme,
        self.lexer.line_number(delimiter_token.loc.start)
      );
      self.err_at(message, end, end + 1)?;
      end
    };
    self.ctx.delimiter = prev;
    Ok(Some(Block {
      loc: SourceLocation::new(meta.start, end),
      meta,
      content,
      context,
    }))
  }

  fn parse_image_block(
    &mut self,
    mut lines: ContiguousLines<'bmp, 'src>,
    meta: ChunkMeta<'bmp>,
  ) -> Result<Block<'bmp>> {
    let mut line = lines.consume_current().unwrap();
    let start = line.loc().unwrap().start;
    line.discard_assert(MacroName);
    line.discard_assert(Colon);
    let target = line.consume_macro_target(self.bump);
    let attrs = self.parse_attr_list(&mut line)?;
    Ok(Block {
      meta,
      loc: SourceLocation::new(start, attrs.loc.end),
      context: Context::Image,
      content: Content::Empty(EmptyMetadata::Image { target, attrs }),
    })
  }

  fn parse_paragraph(
    &mut self,
    mut lines: ContiguousLines<'bmp, 'src>,
    meta: ChunkMeta<'bmp>,
  ) -> Result<Option<Block<'bmp>>> {
    let block_context = meta.block_paragraph_context(&mut lines);

    // TODO: probably a better stack-like context API is possible here...
    let restore_subs = self.ctx.set_subs_for(block_context);
    let inlines = self.parse_inlines(&mut lines)?;
    self.ctx.subs = restore_subs;

    self.restore_lines(lines);
    let Some(end) = inlines.last_loc_end() else {
      return Ok(None);
    };
    Ok(Some(Block {
      loc: SourceLocation::new(meta.start, end),
      meta,
      context: block_context,
      content: Content::Simple(inlines),
    }))
  }

  fn parse_quoted_paragraph(
    &mut self,
    mut lines: ContiguousLines<'bmp, 'src>,
    meta: ChunkMeta<'bmp>,
  ) -> Result<Option<Block<'bmp>>> {
    let mut attr_line = lines.remove_last_unchecked();
    attr_line.discard_assert(TokenKind::Dashes); // `--`
    attr_line.discard_assert(TokenKind::Whitespace);
    let end = attr_line.last_location().unwrap().end;
    let (attr, cite) = attr_line
      .consume_to_string(self.bump)
      .split_once(", ", self.bump);
    lines
      .current_mut()
      .unwrap()
      .discard_assert(TokenKind::DoubleQuote);
    lines
      .last_mut()
      .unwrap()
      .discard_assert_last(TokenKind::DoubleQuote);
    Ok(Some(Block {
      loc: SourceLocation::new(meta.start, end),
      meta,
      context: Context::QuotedParagraph,
      content: Content::QuotedParagraph {
        quote: self.parse_inlines(&mut lines)?,
        attr,
        cite,
      },
    }))
  }
}

// tests

#[cfg(test)]
mod tests {
  use super::*;
  use crate::test::*;
  use ast::variants::inline::*;
  use test_utils::{adoc, assert_eq, parse_block};

  #[test]
  fn test_parse_simple_block() {
    let input = "hello mamma,\nhello papa\n\n";
    parse_block!(input, block, b);
    let expected = Block {
      context: Context::Paragraph,
      content: Content::Simple(b.inodes([
        inode(Text(b.s("hello mamma,")), l(0, 12)),
        inode(JoiningNewline, l(12, 13)),
        inode(Text(b.s("hello papa")), l(13, 23)),
      ])),
      ..b.empty_block(0, 23)
    };
    assert_eq!(block, expected);
  }

  #[test]
  fn test_parse_literal_block() {
    let input = adoc! {"
      [literal]
      foo `bar`
    "};
    parse_block!(input, block, b);
    let expected = Block {
      meta: ChunkMeta::new(Some(b.positional_attrs("literal", l(1, 8))), None, 0),
      context: Context::Literal,
      content: Content::Simple(b.inodes([n_text("foo `bar`", 10, 19, b)])),
      ..b.empty_block(0, 19)
    };
    assert_eq!(block, expected);
  }

  #[test]
  fn test_parse_delimited_literal_block() {
    let input = adoc! {"
      ....
      foo `bar`
      baz
      ....
    "};
    parse_block!(input, block, b);
    let expected = Block {
      context: Context::Literal,
      content: Content::Simple(b.inodes([
        n_text("foo `bar`", 5, 14, b),
        n(Inline::JoiningNewline, l(14, 15)),
        n_text("baz", 15, 18, b),
      ])),
      ..b.empty_block(0, 23)
    };
    assert_eq!(block, expected);
  }

  #[test]
  fn test_parse_delimited_literal_block_w_double_newline() {
    let input = adoc! {"
      ....
      foo `bar`

      baz
      ....
    "};
    parse_block!(input, block, b);
    let expected = Block {
      context: Context::Literal,
      content: Content::Simple(b.inodes([
        n_text("foo `bar`", 5, 14, b),
        n(Inline::JoiningNewline, l(14, 15)),
        n(Inline::JoiningNewline, l(15, 16)),
        n_text("baz", 16, 19, b),
      ])),
      ..b.empty_block(0, 24)
    };
    assert_eq!(block, expected);
  }

  #[test]
  fn test_parse_listing_block() {
    let input = adoc! {"
      [listing]
      foo `bar`
    "};
    parse_block!(input, block, b);
    let expected = Block {
      meta: ChunkMeta::new(Some(b.positional_attrs("listing", l(1, 8))), None, 0),
      context: Context::Listing,
      content: Content::Simple(b.inodes([n_text("foo `bar`", 10, 19, b)])),
      ..b.empty_block(0, 19)
    };
    assert_eq!(block, expected);
  }

  #[test]
  fn test_parse_delimited_listing_block() {
    let input = adoc! {"
      ----
      foo `bar`
      baz
      ----
    "};
    parse_block!(input, block, b);
    let expected = Block {
      context: Context::Listing,
      content: Content::Simple(b.inodes([
        n_text("foo `bar`", 5, 14, b),
        n(Inline::JoiningNewline, l(14, 15)),
        n_text("baz", 15, 18, b),
      ])),
      ..b.empty_block(0, 23)
    };
    assert_eq!(block, expected);
  }

  #[test]
  fn test_parse_delimited_listing_block_w_double_newline() {
    let input = adoc! {"
      ----
      foo `bar`

      baz
      ----
    "};
    parse_block!(input, block, b);
    let expected = Block {
      context: Context::Listing,
      content: Content::Simple(b.inodes([
        n_text("foo `bar`", 5, 14, b),
        n(Inline::JoiningNewline, l(14, 15)),
        n(Inline::JoiningNewline, l(15, 16)),
        n_text("baz", 16, 19, b),
      ])),
      ..b.empty_block(0, 24)
    };
    assert_eq!(block, expected);
  }

  // jared
  // #[test]
  // fn test_parse_indent_method_literal_block() {
  //   parse_block!(" foo `bar`", block, b);
  //   let expected = Block {
  //     context: Context::Literal,
  //     content: Content::Simple(b.inodes([n_text("foo `bar`", 10, 19, b)])),
  //     loc: l(0, 19),
  //     ..b.empty_block()
  //   };
  //   assert_eq!(block, expected);
  // }

  #[test]
  fn test_parse_doc_attr_entry() {
    parse_block!(":!figure-caption:\n\n", block, b);
    let expected = Block {
      context: Context::DocumentAttributeDecl,
      content: Content::DocumentAttribute("figure-caption".to_string(), AttrEntry::Bool(false)),
      ..b.empty_block(0, 17)
    };
    assert_eq!(block, expected);
  }

  #[test]
  fn test_parse_block_titles() {
    parse_block!(".My Title\nfoo\n\n", block, b);
    let expected = Block {
      meta: ChunkMeta::new(None, Some(b.src("My Title", l(1, 9))), 0),
      context: Context::Paragraph,
      content: Content::Simple(b.inodes([inode(Text(b.s("foo")), l(10, 13))])),
      loc: l(0, 13),
    };
    assert_eq!(block, expected);
  }

  #[test]
  fn test_parse_admonitions() {
    parse_block!("TIP: foo\n\n", block, b);
    let expected = Block {
      context: Context::AdmonitionTip,
      content: Content::Simple(b.inodes([inode(Text(b.s("foo")), l(5, 8))])),
      ..b.empty_block(0, 8)
    };
    assert_eq!(block, expected);

    parse_block!("[pos]\nTIP: foo\n\n", block, b);
    let expected = Block {
      meta: ChunkMeta::new(Some(b.positional_attrs("pos", l(1, 4))), None, 0),
      context: Context::AdmonitionTip,
      content: Content::Simple(b.inodes([inode(Text(b.s("foo")), l(11, 14))])),
      loc: l(0, 14),
    };
    assert_eq!(block, expected);

    parse_block!("[WARNING]\nTIP: foo\n\n", block, b);
    let expected = Block {
      meta: ChunkMeta::new(Some(b.positional_attrs("WARNING", l(1, 8))), None, 0),
      context: Context::AdmonitionWarning,
      content: Content::Simple(b.inodes([
        inode(Text(b.s("TIP: foo")), l(10, 18)), // <-- attr list wins
      ])),
      ..b.empty_block(0, 18)
    };
    assert_eq!(block, expected);

    parse_block!("[WARNING]\n====\nfoo\n====\n\n", block, b);
    let expected = Block {
      meta: ChunkMeta::new(Some(b.positional_attrs("WARNING", l(1, 8))), None, 0),
      context: Context::AdmonitionWarning, // <-- turns example into warning
      content: Content::Compound(b.vec([Block {
        context: Context::Paragraph,
        content: Content::Simple(b.inodes([n_text("foo", 15, 18, b)])),
        ..b.empty_block(15, 18)
      }])),
      loc: l(0, 23),
    };
    assert_eq!(block, expected);

    parse_block!("[CAUTION]\n====\nNOTE: foo\n====\n\n", block, b);
    let expected = Block {
      meta: ChunkMeta::new(Some(b.positional_attrs("CAUTION", l(1, 8))), None, 0),
      context: Context::AdmonitionCaution,
      content: Content::Compound(b.vec([Block {
        context: Context::AdmonitionNote,
        content: Content::Simple(b.inodes([inode(Text(b.s("foo")), l(21, 24))])),
        ..b.empty_block(15, 24)
      }])),
      loc: l(0, 29),
    };
    assert_eq!(block, expected);
  }

  #[test]
  fn test_parse_comment_block() {
    parse_block!("//-", block, b);
    let expected = Block {
      context: Context::Comment,
      content: Content::Empty(EmptyMetadata::None),
      ..b.empty_block(0, 3)
    };
    assert_eq!(block, expected);
  }

  #[test]
  fn test_parse_image_block() {
    parse_block!("image::name.png[]\n\n", block, b);
    let expected = Block {
      context: Context::Image,
      content: Content::Empty(EmptyMetadata::Image {
        target: b.src("name.png", l(7, 15)),
        attrs: AttrList::new(l(15, 17), b),
      }),
      ..b.empty_block(0, 17)
    };
    assert_eq!(block, expected);
  }

  #[test]
  fn test_parse_delimited_open_block() {
    parse_block!("--\nfoo\n--\n\n", block, b);
    let expected = Block {
      context: Context::Open,
      content: Content::Compound(b.vec([Block {
        context: Context::Paragraph,
        content: Content::Simple(b.inodes([n_text("foo", 3, 6, b)])),
        ..b.empty_block(3, 6)
      }])),
      ..b.empty_block(0, 9)
    };
    assert_eq!(block, expected);
  }

  #[test]
  fn test_parse_delimited_example_block() {
    parse_block!("====\nfoo\n====\n\n", block, b);
    let expected = Block {
      context: Context::Example,
      content: Content::Compound(b.vec([Block {
        context: Context::Paragraph,
        content: Content::Simple(b.inodes([n_text("foo", 5, 8, b)])),
        ..b.empty_block(5, 8)
      }])),
      ..b.empty_block(0, 13)
    };
    assert_eq!(block, expected);
  }

  #[test]
  fn test_quoted_paragraph() {
    let input = adoc! {r#"
      "I hold it that a little blah,
      and as necessary in the blah."
      -- Thomas Jefferson, Papers of Thomas Jefferson: Volume 11
    "#};
    parse_block!(input, block, b);
    let expected = Block {
      context: Context::QuotedParagraph,
      content: Content::QuotedParagraph {
        quote: b.inodes([
          n_text("I hold it that a little blah,", 1, 30, b),
          n(Inline::JoiningNewline, l(30, 31)),
          n_text("and as necessary in the blah.", 31, 60, b),
        ]),
        attr: b.src("Thomas Jefferson", l(65, 81)),
        cite: Some(b.src("Papers of Thomas Jefferson: Volume 11", l(83, 120))),
      },
      ..b.empty_block(0, 120)
    };
    assert_eq!(block, expected);
  }

  #[test]
  fn test_quoted_paragraph_no_cite_w_attr_meta() {
    let input = adoc! {r#"
      .A Title
      [#foo]
      "I hold it that a little blah,
      and as necessary in the blah."
      -- Thomas Jefferson
    "#};
    parse_block!(input, block, b);
    let expected = Block {
      meta: ChunkMeta::new(
        Some(AttrList {
          id: Some(b.src("foo", l(11, 14))),
          ..AttrList::new(l(9, 15), b)
        }),
        Some(b.src("A Title", l(1, 8))),
        0,
      ),
      context: Context::QuotedParagraph,
      content: Content::QuotedParagraph {
        quote: b.inodes([
          n_text("I hold it that a little blah,", 17, 46, b),
          n(Inline::JoiningNewline, l(46, 47)),
          n_text("and as necessary in the blah.", 47, 76, b),
        ]),
        attr: b.src("Thomas Jefferson", l(81, 97)),
        cite: None,
      },
      loc: l(0, 97),
    };
    assert_eq!(block, expected);
  }

  #[test]
  fn test_simple_blockquote() {
    parse_block!("[quote,author,location]\nfoo\n\n", block, b);
    let expected = Block {
      meta: ChunkMeta::new(
        Some(AttrList {
          positional: b.vec([
            Some(b.inodes([n_text("quote", 1, 6, b)])),
            Some(b.inodes([n_text("author", 7, 13, b)])),
            Some(b.inodes([n_text("location", 14, 22, b)])),
          ]),
          ..AttrList::new(l(0, 23), b)
        }),
        None,
        0,
      ),
      context: Context::BlockQuote,
      content: Content::Simple(b.inodes([n_text("foo", 24, 27, b)])),
      ..b.empty_block(0, 27)
    };
    assert_eq!(block, expected);
  }

  #[test]
  fn test_parse_delimited_blockquote() {
    let input = adoc! {"
      [quote,author,location]
      ____
      foo
      ____
    "};
    parse_block!(input, block, b);
    let expected = Block {
      meta: ChunkMeta::new(
        Some(AttrList {
          positional: b.vec([
            Some(b.inodes([n_text("quote", 1, 6, b)])),
            Some(b.inodes([n_text("author", 7, 13, b)])),
            Some(b.inodes([n_text("location", 14, 22, b)])),
          ]),
          ..AttrList::new(l(0, 23), b)
        }),
        None,
        0,
      ),
      context: Context::BlockQuote,
      content: Content::Compound(b.vec([Block {
        context: Context::Paragraph,
        content: Content::Simple(b.inodes([n_text("foo", 29, 32, b)])),
        ..b.empty_block(29, 32)
      }])),
      ..b.empty_block(0, 37)
    };
    assert_eq!(block, expected);
  }

  #[test]
  fn test_undelimited_sidebar() {
    parse_block!("[sidebar]\nfoo\n\n", block, b);
    let expected = Block {
      meta: ChunkMeta::new(Some(b.positional_attrs("sidebar", l(1, 8))), None, 0),
      context: Context::Sidebar,
      content: Content::Simple(b.inodes([n_text("foo", 10, 13, b)])),
      ..b.empty_block(0, 13)
    };
    assert_eq!(block, expected);
  }

  #[test]
  fn test_parse_empty_delimited_block() {
    parse_block!("--\n--\n\n", block, b);
    let expected = Block {
      context: Context::Open,
      content: Content::Compound(b.vec([])),
      ..b.empty_block(0, 5)
    };
    assert_eq!(block, expected);
  }

  #[test]
  fn test_parse_delimited_sidebar_block() {
    parse_block!("****\nfoo\n****\n\n", block, b);
    let expected = Block {
      context: Context::Sidebar,
      content: Content::Compound(b.vec([Block {
        context: Context::Paragraph,
        content: Content::Simple(b.inodes([n_text("foo", 5, 8, b)])),
        ..b.empty_block(5, 8)
      }])),
      ..b.empty_block(0, 13)
    };
    assert_eq!(block, expected);
  }

  #[test]
  fn test_nested_delimiter_blocks() {
    let input = adoc! {"
      ****
      --
      foo
      --
      ****
    "};
    parse_block!(input, block, b);
    let expected = Block {
      context: Context::Sidebar,
      content: Content::Compound(b.vec([Block {
        context: Context::Open,
        content: Content::Compound(b.vec([Block {
          context: Context::Paragraph,
          content: Content::Simple(b.inodes([n_text("foo", 8, 11, b)])),
          ..b.empty_block(8, 11)
        }])),
        ..b.empty_block(5, 14)
      }])),
      ..b.empty_block(0, 19)
    };
    assert_eq!(block, expected);

    let input = adoc! {"
      ****

      .Bar
      --

      foo


      --

      ****
    "};
    parse_block!(input, block, b);
    let expected = Block {
      context: Context::Sidebar,
      content: Content::Compound(b.vec([Block {
        meta: ChunkMeta::new(None, Some(b.src("Bar", l(7, 10))), 6),
        context: Context::Open,
        content: Content::Compound(b.vec([Block {
          context: Context::Paragraph,
          content: Content::Simple(b.inodes([n_text("foo", 15, 18, b)])),
          ..b.empty_block(15, 18)
        }])),
        loc: l(6, 23),
      }])),
      ..b.empty_block(0, 29)
    };
    assert_eq!(block, expected);
  }

  #[test]
  fn test_parse_multi_para_delimited_sidebar_block() {
    let input = adoc! {"
      ****
      This is content in a sidebar block.

      image::name.png[]

      This is more content in the sidebar block.
      ****
    "};
    parse_block!(input, block, b);
    let para_1_txt = n_text("This is content in a sidebar block.", 5, 40, b);
    let para_2_txt = n_text("This is more content in the sidebar block.", 61, 103, b);
    let expected = Block {
      context: Context::Sidebar,
      content: Content::Compound(b.vec([
        Block {
          context: Context::Paragraph,
          content: Content::Simple(b.inodes([para_1_txt])),
          ..b.empty_block(5, 40)
        },
        Block {
          context: Context::Image,
          content: Content::Empty(EmptyMetadata::Image {
            target: b.src("name.png", l(49, 57)),
            attrs: AttrList::new(l(57, 59), b),
          }),
          ..b.empty_block(42, 59)
        },
        Block {
          context: Context::Paragraph,
          content: Content::Simple(b.inodes([para_2_txt])),
          ..b.empty_block(61, 103)
        },
      ])),
      ..b.empty_block(0, 108)
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
