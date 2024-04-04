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

    match lines.section_start_level(&meta) {
      Some(level) if level <= self.ctx.section_level => {
        self.restore_peeked(lines, meta);
        return Ok(None);
      }
      Some(_) => {
        self.restore_peeked(lines, meta);
        let section = self.parse_section()?.unwrap();
        return Ok(Some(Block {
          loc: SourceLocation::new(section.meta.start, self.loc().end),
          meta: ChunkMeta::empty(section.meta.start),
          context: Context::Section,
          content: Content::Section(section),
        }));
      }
      _ => {}
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
          self.restore_lines(lines);
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
    if lines.starts_with_comment_line() {
      let start = lines.current_token().unwrap().loc.start;
      lines.consume_current();
      lines.discard_leading_comment_lines();
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
    let restore_subs = self.ctx.set_subs_for(context, &meta);

    // newlines have a different meaning in a these contexts, so we have to
    // manually gather all (including empty) lines until the end delimiter
    let content = if matches!(
      context,
      Context::Listing | Context::Literal | Context::Passthrough
    ) {
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
    let restore_subs = self.ctx.set_subs_for(block_context, &meta);
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
  use ast::variants::inline::*;
  use test_utils::{assert_eq, *};

  #[test]
  fn test_parse_simple_block() {
    let input = "hello mamma,\nhello papa\n\n";
    let expected = Block {
      context: Context::Paragraph,
      content: Content::Simple(nodes![
        node!("hello mamma,"; 0..12),
        node!(JoiningNewline, 12..13),
        node!("hello papa"; 13..23),
      ]),
      ..empty_block!(0..23)
    };
    assert_eq!(parse_single_block!(input), expected);
  }

  #[test]
  fn test_parse_literal_block() {
    assert_block!(
      adoc! {"
        [literal]
        foo `bar`
      "},
      Block {
        meta: ChunkMeta::new(Some(attrs::pos("literal", 1..8)), None, 0),
        context: Context::Literal,
        content: Content::Simple(nodes![node!("foo `bar`"; 10..19)]),
        ..empty_block!(0..19)
      }
    );
  }

  #[test]
  fn test_parse_delimited_literal_block() {
    let input = adoc! {"
      ....
      foo `bar`
      baz
      ....
    "};
    assert_block!(
      input,
      Block {
        context: Context::Literal,
        content: Content::Simple(nodes![
          node!("foo `bar`"; 5..14),
          node!(Inline::JoiningNewline, 14..15),
          node!("baz"; 15..18),
        ]),
        ..empty_block!(0..23)
      }
    )
  }

  #[test]
  fn test_parse_passthrough() {
    assert_block!(
      adoc! {"
        [pass]
        foo <bar>
      "},
      Block {
        meta: ChunkMeta::new(Some(attrs::pos("pass", 1..5)), None, 0),
        context: Context::Passthrough,
        content: Content::Simple(nodes![node!("foo <bar>"; 7..16)]),
        ..empty_block!(0..16)
      }
    );
  }

  #[test]
  fn test_parse_delimited_passthrough_block() {
    let input = adoc! {"
      ++++
      foo <bar>
      baz
      ++++
    "};
    let expected = Block {
      context: Context::Passthrough,
      content: Content::Simple(nodes![
        node!("foo <bar>"; 5..14),
        node!(Inline::JoiningNewline, 14..15),
        node!("baz"; 15..18),
      ]),
      ..empty_block!(0..23)
    };
    assert_block!(input, expected);
  }

  #[test]
  fn test_parse_delimited_passthrough_block_subs_normal() {
    let input = adoc! {"
      [subs=normal]
      ++++
      foo & _<bar>_
      baz
      ++++
    "};
    let expected = Block {
      meta: ChunkMeta {
        attrs: Some(attrs::named(&[("subs", 1..5, "normal", 6..12)])),
        ..ChunkMeta::default()
      },
      context: Context::Passthrough,
      content: Content::Simple(nodes![
        node!("foo "; 19..23),
        node!(SpecialChar(SpecialCharKind::Ampersand), 23..24),
        node!(" "; 24..25),
        node!(
          Italic(nodes![
            node!(SpecialChar(SpecialCharKind::LessThan), 26..27),
            node!("bar"; 27..30),
            node!(SpecialChar(SpecialCharKind::GreaterThan), 30..31),
          ]),
          25..32,
        ),
        node!(Inline::JoiningNewline, 32..33),
        node!("baz"; 33..36),
      ]),
      ..empty_block!(0..41)
    };
    assert_block!(input, expected);
  }

  #[test]
  fn test_parse_delimited_literal_block_w_double_newline() {
    let input = adoc! {"
      ....
      foo `bar`

      baz
      ....
    "};
    let expected = Block {
      context: Context::Literal,
      content: Content::Simple(nodes![
        node!("foo `bar`"; 5..14),
        node!(Inline::JoiningNewline, 14..15),
        node!(Inline::JoiningNewline, 15..16),
        node!("baz"; 16..19),
      ]),
      ..empty_block!(0..24)
    };
    assert_block!(input, expected);
  }

  #[test]
  fn test_parse_listing_block() {
    assert_block!(
      adoc! {"
        [listing]
        foo `bar`
      "},
      Block {
        meta: ChunkMeta::new(Some(attrs::pos("listing", 1..8)), None, 0),
        context: Context::Listing,
        content: Content::Simple(nodes![node!("foo `bar`"; 10..19)]),
        ..empty_block!(0..19)
      }
    );
  }

  #[test]
  fn test_parse_delimited_listing_block() {
    let input = adoc! {"
      ----
      foo `bar`
      baz
      ----
    "};
    let expected = Block {
      context: Context::Listing,
      content: Content::Simple(nodes![
        node!("foo `bar`"; 5..14),
        node!(Inline::JoiningNewline, 14..15),
        node!("baz"; 15..18),
      ]),
      ..empty_block!(0..23)
    };
    assert_block!(input, expected);
  }

  #[test]
  fn test_parse_delimited_listing_block_w_double_newline() {
    let input = adoc! {"
      ----
      foo `bar`

      baz
      ----
    "};
    let expected = Block {
      context: Context::Listing,
      content: Content::Simple(nodes![
        node!("foo `bar`"; 5..14),
        node!(Inline::JoiningNewline, 14..15),
        node!(Inline::JoiningNewline, 15..16),
        node!("baz"; 16..19),
      ]),
      ..empty_block!(0..24)
    };
    assert_block!(input, expected);
  }

  #[test]
  fn test_parse_doc_attr_entry() {
    let bump = &Bump::new();
    let mut parser = Parser::new(bump, ":!figure-caption:\n\n");
    let block = parser.parse_block().unwrap().unwrap();
    let expected = Block {
      context: Context::DocumentAttributeDecl,
      content: Content::DocumentAttribute("figure-caption".to_string(), AttrEntry::Bool(false)),
      ..empty_block!(0..17)
    };
    assert_eq!(block, expected);
  }

  #[test]
  fn test_parse_block_titles() {
    let input = ".My Title\nfoo\n\n";
    let expected = Block {
      meta: ChunkMeta::new(None, Some(src!("My Title", 1..9)), 0),
      context: Context::Paragraph,
      content: Content::Simple(nodes![node!("foo"; 10..13)]),
      ..empty_block!(0..13)
    };
    assert_block!(input, expected);
  }

  #[test]
  fn test_parse_admonitions() {
    assert_block!(
      adoc! {"
        TIP: foo
      "},
      Block {
        context: Context::AdmonitionTip,
        content: Content::Simple(nodes![node!("foo"; 5..8)]),
        ..empty_block!(0..8)
      }
    );

    assert_block!(
      adoc! {"
        [pos]
        TIP: foo
      "},
      Block {
        meta: ChunkMeta::new(Some(attrs::pos("pos", 1..4)), None, 0),
        context: Context::AdmonitionTip,
        content: Content::Simple(nodes![node!("foo"; 11..14)]),
        ..empty_block!(0..14)
      }
    );

    assert_block!(
      adoc! {"
        [WARNING]
        TIP: foo
      "},
      Block {
        meta: ChunkMeta::new(Some(attrs::pos("WARNING", 1..8)), None, 0),
        context: Context::AdmonitionWarning,
        content: Content::Simple(nodes![
          node!("TIP: foo"; 10..18), // <-- attr list wins
        ]),
        ..empty_block!(0..18)
      }
    );

    assert_block!(
      adoc! {"
        [WARNING]
        ====
        foo
        ====
      "},
      Block {
        meta: ChunkMeta::new(Some(attrs::pos("WARNING", 1..8)), None, 0),
        context: Context::AdmonitionWarning, // <-- turns example into warning
        content: Content::Compound(vecb![Block {
          context: Context::Paragraph,
          content: Content::Simple(nodes![node!("foo"; 15..18)]),
          ..empty_block!(15..18)
        }]),
        ..empty_block!(0..23)
      }
    );

    assert_block!(
      adoc! {"
        [CAUTION]
        ====
        NOTE: foo
        ====
      "},
      Block {
        meta: ChunkMeta::new(Some(attrs::pos("CAUTION", 1..8)), None, 0),
        context: Context::AdmonitionCaution,
        content: Content::Compound(vecb![Block {
          context: Context::AdmonitionNote,
          content: Content::Simple(nodes![node!("foo"; 21..24)]),
          ..empty_block!(15..24)
        }]),
        ..empty_block!(0..29)
      }
    );
  }

  #[test]
  fn test_parse_comment_block() {
    assert_block!(
      "//-",
      Block {
        context: Context::Comment,
        content: Content::Empty(EmptyMetadata::None),
        ..empty_block!(0..3)
      }
    );
  }

  #[test]
  fn test_parse_image_block() {
    assert_block!(
      "image::name.png[]\n\n",
      Block {
        context: Context::Image,
        content: Content::Empty(EmptyMetadata::Image {
          target: src!("name.png", 7..15),
          attrs: attr_list!(15..17),
        }),
        ..empty_block!(0..17)
      }
    );
  }

  #[test]
  fn test_parse_delimited_open_block() {
    assert_block!(
      adoc! {"
        --
        foo
        --
      "},
      Block {
        context: Context::Open,
        content: Content::Compound(vecb![Block {
          context: Context::Paragraph,
          content: Content::Simple(nodes![node!("foo"; 3..6)]),
          ..empty_block!(3..6)
        }]),
        ..empty_block!(0..9)
      }
    );
  }

  #[test]
  fn test_parse_delimited_example_block() {
    assert_block!(
      adoc! {"
        ====
        foo
        ====
      "},
      Block {
        context: Context::Example,
        content: Content::Compound(vecb![Block {
          context: Context::Paragraph,
          content: Content::Simple(nodes![node!("foo"; 5..8)]),
          ..empty_block!(5..8)
        }]),
        ..empty_block!(0..13)
      },
    );
  }

  #[test]
  fn test_quoted_paragraph() {
    let input = adoc! {r#"
      "I hold it that a little blah,
      and as necessary in the blah."
      -- Thomas Jefferson, Papers of Thomas Jefferson: Volume 11
    "#};
    let expected = Block {
      context: Context::QuotedParagraph,
      content: Content::QuotedParagraph {
        quote: nodes![
          node!("I hold it that a little blah,"; 1..30),
          node!(Inline::JoiningNewline, 30..31),
          node!("and as necessary in the blah."; 31..60),
        ],
        attr: src!("Thomas Jefferson", 65..81),
        cite: Some(src!("Papers of Thomas Jefferson: Volume 11", 83..120)),
      },
      ..empty_block!(0..120)
    };
    assert_block!(input, expected);
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
    let expected = Block {
      meta: ChunkMeta::new(
        Some(AttrList {
          id: Some(src!("foo", 11..14)),
          ..attr_list!(9..15)
        }),
        Some(src!("A Title", 1..8)),
        0,
      ),
      context: Context::QuotedParagraph,
      content: Content::QuotedParagraph {
        quote: nodes![
          node!("I hold it that a little blah,"; 17..46),
          node!(Inline::JoiningNewline, 46..47),
          node!("and as necessary in the blah."; 47..76),
        ],
        attr: src!("Thomas Jefferson", 81..97),
        cite: None,
      },
      ..empty_block!(0..97)
    };
    assert_block!(input, expected);
  }

  #[test]
  fn test_simple_blockquote() {
    let input = adoc! {"
      [quote,author,location]
      foo
    "};
    let expected = Block {
      meta: ChunkMeta {
        attrs: Some(AttrList {
          positional: vecb![
            Some(nodes![node!("quote"; 1..6)]),
            Some(nodes![node!("author"; 7..13)]),
            Some(nodes![node!("location"; 14..22)]),
          ],
          ..attr_list!(0..23)
        }),
        ..ChunkMeta::default()
      },
      context: Context::BlockQuote,
      content: Content::Simple(nodes![node!("foo"; 24.. 27)]),
      ..empty_block!(0..27)
    };
    assert_block!(input, expected,)
  }

  #[test]
  fn test_parse_delimited_blockquote() {
    let input = adoc! {"
      [quote,author,location]
      ____
      foo
      ____
    "};
    let expected = Block {
      meta: ChunkMeta {
        attrs: Some(AttrList {
          positional: vecb![
            Some(nodes![node!("quote"; 1..6)]),
            Some(nodes![node!("author"; 7..13)]),
            Some(nodes![node!("location"; 14..22)]),
          ],
          ..attr_list!(0..23)
        }),
        ..ChunkMeta::default()
      },
      context: Context::BlockQuote,
      content: Content::Compound(vecb![Block {
        context: Context::Paragraph,
        content: Content::Simple(nodes![node!("foo"; 29.. 32)]),
        ..empty_block!(29..32)
      }]),
      ..empty_block!(0..37)
    };
    assert_block!(input, expected);
  }

  #[test]
  fn test_undelimited_sidebar() {
    assert_block!(
      adoc! {"
        [sidebar]
        foo
      "},
      Block {
        meta: ChunkMeta::new(Some(attrs::pos("sidebar", 1..8)), None, 0),
        context: Context::Sidebar,
        content: Content::Simple(nodes![node!("foo"; 10.. 13)]),
        ..empty_block!(0..13)
      }
    );
  }

  #[test]
  fn test_parse_empty_delimited_block() {
    assert_block!(
      adoc! {"
        --
        --
      "},
      Block {
        context: Context::Open,
        content: Content::Compound(vecb![]),
        ..empty_block!(0..5)
      }
    );
  }

  #[test]
  fn test_parse_delimited_sidebar_block() {
    assert_block!(
      adoc! {"
        ****
        foo
        ****
      "},
      Block {
        context: Context::Sidebar,
        content: Content::Compound(vecb![Block {
          context: Context::Paragraph,
          content: Content::Simple(nodes![node!("foo"; 5.. 8)]),
          ..empty_block!(5..8)
        }]),
        ..empty_block!(0..13)
      },
    )
  }

  #[test]
  fn test_nested_delimiter_blocks() {
    assert_block!(
      adoc! {"
        ****
        --
        foo
        --
        ****
      "},
      Block {
        context: Context::Sidebar,
        content: Content::Compound(vecb![Block {
          context: Context::Open,
          content: Content::Compound(vecb![Block {
            context: Context::Paragraph,
            content: Content::Simple(nodes![node!("foo"; 8.. 11)]),
            ..empty_block!(8..11)
          }]),
          ..empty_block!(5..14)
        }]),
        ..empty_block!(0..19)
      }
    );

    assert_block!(
      adoc! {"
        ****

        .Bar
        --

        foo


        --

        ****
      "},
      Block {
        context: Context::Sidebar,
        content: Content::Compound(vecb![Block {
          meta: ChunkMeta::new(None, Some(src!("Bar", 7..10)), 6),
          context: Context::Open,
          content: Content::Compound(vecb![Block {
            context: Context::Paragraph,
            content: Content::Simple(nodes![node!("foo"; 15..18)]),
            ..empty_block!(15..18)
          }]),
          ..empty_block!(6..23)
        }]),
        ..empty_block!(0..29)
      }
    );
  }

  #[test]
  fn test_parse_multi_para_delimited_sidebar_block() {
    assert_block!(
      adoc! {"
        ****
        This is content in a sidebar block.

        image::name.png[]

        This is more content in the sidebar block.
        ****
      "},
      Block {
        context: Context::Sidebar,
        content: Content::Compound(vecb![
          Block {
            context: Context::Paragraph,
            content: Content::Simple(nodes![node!("This is content in a sidebar block."; 5..40)]),
            ..empty_block!(5..40)
          },
          Block {
            context: Context::Image,
            content: Content::Empty(EmptyMetadata::Image {
              target: src!("name.png", 49..57),
              attrs: attr_list!(57..59),
            }),
            ..empty_block!(42..59)
          },
          Block {
            context: Context::Paragraph,
            content: Content::Simple(nodes![
              node!("This is more content in the sidebar block."; 61..103)
            ]),
            ..empty_block!(61..103)
          },
        ]),
        ..empty_block!(0..108)
      }
    );
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
