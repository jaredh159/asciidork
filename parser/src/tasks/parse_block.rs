use crate::internal::*;
use crate::variants::token::*;
use ast::short::block::*;

impl<'bmp, 'src> Parser<'bmp, 'src> {
  pub(crate) fn parse_block(&mut self) -> Result<Option<Block<'bmp>>> {
    let Some(mut lines) = self.read_lines() else {
      return Ok(None);
    };

    if let Some(comment_block) = self.parse_line_comment_block(&mut lines) {
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
      return match first_token.lexeme {
        "image:" => self.parse_image_block(lines, meta),
        "toc:" => self.parse_toc_macro(first_token.loc, lines, meta),
        _ => todo!("unhandled block macro type: `{:?}`", first_token.lexeme),
      }
      .map(Some);
    } else if lines.starts_list() {
      return self.parse_list(lines, Some(meta)).map(Some);
    } else if lines.current_satisfies(|line| line.is_heading()) {
      return self.parse_discrete_heading(lines, meta).map(Some);
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
      Pipe | Colon | Bang | Comma
        if lines.nth_token(1).is_len(EqualSigns, 3) && lines.nth_token(2).is_none() =>
      {
        return Ok(Some(self.parse_table(lines, meta)?));
      }
      Colon => {
        if let Some((key, value, end)) = self.parse_doc_attr(&mut lines)? {
          self.restore_lines(lines);
          // TODO: test error
          _ = self.document.meta.insert_doc_attr(&key, value.clone());
          return Ok(Some(Block {
            loc: SourceLocation::new(meta.start, end),
            meta,
            context: Context::DocumentAttributeDecl,
            content: Content::DocumentAttribute(key, value),
          }));
        }
      }
      SingleQuote if lines.current_satisfies(|line| line.src == "'''") => {
        return self.parse_break(Context::ThematicBreak, lines, meta);
      }
      LessThan if lines.current_satisfies(|line| line.src == "<<<") => {
        return self.parse_break(Context::PageBreak, lines, meta);
      }
      _ => {}
    }

    if lines.is_quoted_paragraph() {
      self.parse_quoted_paragraph(lines, meta)
    } else {
      self.parse_paragraph(lines, meta)
    }
  }

  fn parse_discrete_heading(
    &mut self,
    mut lines: ContiguousLines<'bmp, 'src>,
    meta: ChunkMeta<'bmp>,
  ) -> Result<Block<'bmp>> {
    let mut line = lines.consume_current().unwrap();
    let level = line.heading_level().unwrap();
    line.discard_assert(TokenKind::EqualSigns);
    line.discard_assert(TokenKind::Whitespace);
    let id = self.section_id(line.src, meta.attrs.as_ref());
    let content = self.parse_inlines(&mut line.into_lines_in(self.bump))?;
    let end = content.last_loc().unwrap().end;
    self.restore_lines(lines);
    Ok(Block {
      loc: SourceLocation::new(meta.start, end),
      meta,
      context: Context::DiscreteHeading,
      content: Content::Empty(EmptyMetadata::DiscreteHeading { level, content, id }),
    })
  }

  // important to represent these as an ast node because
  // they are the documented way to separate adjacent lists
  fn parse_line_comment_block(
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
      Context::Listing
        | Context::Literal
        | Context::Passthrough
        | Context::Comment
        | Context::Verse
    ) {
      let mut lines = self
        .read_lines_until(delimiter)
        .unwrap_or_else(|| ContiguousLines::new(bvec![in self.bump]));

      if context == Context::Listing || context == Context::Literal {
        if let Some(comment) = meta.attrs.as_ref().and_then(|a| a.named("line-comment")) {
          self.ctx.custom_line_comment = Some(SmallVec::from_slice(comment.as_bytes()));
        }
      }

      if context == Context::Comment {
        lines.discard_until(|l| l.starts_with(|token| token.lexeme == "////"));
        self.restore_lines(lines);
        Content::Empty(EmptyMetadata::None)
      } else {
        self.ctx.can_nest_blocks = false;
        let simple = Content::Simple(self.parse_inlines(&mut lines)?);
        self.ctx.can_nest_blocks = true;
        self.ctx.custom_line_comment = None;
        self.restore_lines(lines);
        simple
      }
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
      self.err_token_full("This delimiter was never closed", &delimiter_token)?;
      content.last_loc().unwrap_or(delimiter_token.loc).end
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
    let context = meta.block_paragraph_context(&mut lines);
    // TODO: probably a better stack-like context API is possible here...
    let restore_subs = self.ctx.set_subs_for(context, &meta);
    let inlines = self.parse_inlines(&mut lines)?;
    self.ctx.subs = restore_subs;

    self.restore_lines(lines);
    let Some(end) = inlines.last_loc_end() else {
      return Ok(None);
    };

    let content = if context == Context::Comment {
      // PERF: could squeeze out some speed by not parsing inlines
      Content::Empty(EmptyMetadata::None)
    } else {
      Content::Simple(inlines)
    };

    Ok(Some(Block {
      loc: SourceLocation::new(meta.start, end),
      meta,
      context,
      content,
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

  fn parse_break(
    &mut self,
    context: BlockContext,
    mut lines: ContiguousLines<'bmp, 'src>,
    meta: ChunkMeta<'bmp>,
  ) -> Result<Option<Block<'bmp>>> {
    let end = lines.consume_current().unwrap().last_loc().unwrap().end;
    self.restore_lines(lines);
    Ok(Some(Block {
      loc: SourceLocation::new(meta.start, end),
      meta,
      context,
      content: Content::Empty(EmptyMetadata::None),
    }))
  }

  fn parse_toc_macro(
    &mut self,
    token_loc: SourceLocation,
    lines: ContiguousLines<'bmp, 'src>,
    meta: ChunkMeta<'bmp>,
  ) -> Result<Block<'bmp>> {
    self.ctx.saw_toc_macro = true;
    if self.document.toc.is_none() {
      self.err_at(
        "Found macro placing Table of Contents, but TOC not enabled",
        token_loc.start,
        lines.current().unwrap().last_loc().unwrap().end,
      )?;
    }
    return Ok(Block {
      loc: SourceLocation::new(meta.start, self.loc().end),
      meta,
      context: Context::TableOfContents,
      content: Content::Empty(EmptyMetadata::None),
    });
  }
}

// tests

#[cfg(test)]
mod tests {
  use super::*;
  use test_utils::{assert_eq, *};

  #[test]
  fn test_parse_doc_attr_entry() {
    let mut parser = Parser::new(leaked_bump(), ":!figure-caption:\n\n");
    let block = parser.parse_block().unwrap().unwrap();
    let expected = Block {
      context: Context::DocumentAttributeDecl,
      content: Content::DocumentAttribute("figure-caption".to_string(), AttrValue::Bool(false)),
      ..empty_block!(0..17)
    };
    assert_eq!(block, expected);
  }

  #[test]
  fn test_inline_only_parses_single_paragraph() {
    let input = adoc! {"
      first para

      second para (ignored)
    "};
    let parser = Parser::new_opts(leaked_bump(), input, opts::Opts::embedded());
    let result = parser.parse().unwrap();
    let blocks = result.document.content.blocks().unwrap();
    assert_eq!(blocks.len(), 1);
    assert_eq!(
      blocks[0],
      Block {
        context: Context::Paragraph,
        content: Content::Simple(just!("first para", 0..10)),
        ..empty_block!(0..10)
      }
    );
  }
}
