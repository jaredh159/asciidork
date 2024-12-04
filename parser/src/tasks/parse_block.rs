use crate::internal::*;
use crate::variants::token::*;
use ast::short::block::*;

impl<'arena> Parser<'arena> {
  pub(crate) fn parse_block(&mut self) -> Result<Option<Block<'arena>>> {
    let Some(mut lines) = self.read_lines()? else {
      return Ok(None);
    };

    if let Some(comment_block) = self.parse_comment_block(&mut lines) {
      self.restore_lines(lines);
      return Ok(Some(comment_block));
    }

    let meta = self.parse_chunk_meta(&mut lines)?;

    match self.section_start_level(&lines, &meta) {
      Some(0) => {} // skip document titles
      Some(level) => {
        self.restore_peeked(lines, meta);
        if level <= self.ctx.section_level {
          return Ok(None);
        } else {
          let section = self.parse_section()?.unwrap();
          return Ok(Some(Block {
            meta: ChunkMeta::empty(section.meta.start),
            context: Context::Section,
            content: Content::Section(section),
          }));
        }
      }
      None => {}
    }

    let first_token = lines.current_token().unwrap();

    if lines.is_block_macro() {
      return match first_token.lexeme.as_str() {
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
          if let Err(err) = self.document.meta.insert_doc_attr(&key, value.clone()) {
            self.err_at(err, meta.start, end)?;
          }
          return Ok(Some(Block {
            meta,
            context: Context::DocumentAttributeDecl,
            content: Content::DocumentAttribute(key, value),
          }));
        }
      }
      SingleQuote
        if lines.current_satisfies(|line| {
          line.num_tokens() == 3
            && line.starts_with_seq(&[Kind(SingleQuote), Kind(SingleQuote), Kind(SingleQuote)])
        }) =>
      {
        return self.parse_break(Context::ThematicBreak, lines, meta);
      }
      LessThan
        if lines.current_satisfies(|line| {
          line.num_tokens() == 3
            && line.starts_with_seq(&[Kind(LessThan), Kind(LessThan), Kind(LessThan)])
        }) =>
      {
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
    mut lines: ContiguousLines<'arena>,
    meta: ChunkMeta<'arena>,
  ) -> Result<Block<'arena>> {
    let mut line = lines.consume_current().unwrap();
    let level = self.line_heading_level(&line).unwrap();
    line.discard_assert(TokenKind::EqualSigns);
    line.discard_assert(TokenKind::Whitespace);
    let id = self.section_id(&line, meta.attrs.as_ref());
    let content = self.parse_inlines(&mut line.into_lines())?;
    self.restore_lines(lines);
    Ok(Block {
      meta,
      context: Context::DiscreteHeading,
      content: Content::Empty(EmptyMetadata::DiscreteHeading { level, content, id }),
    })
  }

  // important to represent these as an ast node because
  // they are the documented way to separate adjacent lists
  fn parse_comment_block(&mut self, lines: &mut ContiguousLines<'arena>) -> Option<Block<'arena>> {
    if lines.starts_with_comment_line() {
      let start = lines.current_token().unwrap().loc.start;
      lines.consume_current();
      lines.discard_leading_comment_lines();
      if lines.is_empty() {
        return Some(Block {
          meta: ChunkMeta::empty(start),
          context: Context::Comment,
          content: Content::Empty(EmptyMetadata::None),
        });
      }
    }
    None
  }

  fn parse_delimited_block(
    &mut self,
    delimiter: Delimiter,
    mut lines: ContiguousLines<'arena>,
    meta: ChunkMeta<'arena>,
  ) -> Result<Option<Block<'arena>>> {
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
        .read_lines_until(delimiter)?
        .unwrap_or_else(|| ContiguousLines::new(Deq::new(self.bump)));

      if let Some(indent) = meta
        .attr_named("indent")
        .and_then(|s| s.parse::<usize>().ok())
      {
        let delimiter = lines.pop();
        lines.set_indentation(indent);
        delimiter.map(|d| lines.push(d));
      }

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
    if let Some(mut block) = self.read_lines()? {
      let token = block.consume_current_token().unwrap();
      debug_assert!(token.is(DelimiterLine));
      self.restore_lines(block);
    } else {
      self.err_token_full("This delimiter was never closed", &delimiter_token)?;
    };
    self.ctx.delimiter = prev;
    Ok(Some(Block { meta, content, context }))
  }

  fn parse_image_block(
    &mut self,
    mut lines: ContiguousLines<'arena>,
    meta: ChunkMeta<'arena>,
  ) -> Result<Block<'arena>> {
    let mut line = lines.consume_current().unwrap();
    line.discard_assert(MacroName);
    line.discard_assert(Colon);
    let target = line.consume_macro_target(self.bump);
    let attrs = self.parse_block_attr_list(&mut line)?;
    Ok(Block {
      meta,
      context: Context::Image,
      content: Content::Empty(EmptyMetadata::Image { target, attrs }),
    })
  }

  fn parse_paragraph(
    &mut self,
    mut lines: ContiguousLines<'arena>,
    meta: ChunkMeta<'arena>,
  ) -> Result<Option<Block<'arena>>> {
    let context = meta.block_paragraph_context(&mut lines);
    // TODO: probably a better stack-like context API is possible here...
    let restore_subs = self.ctx.set_subs_for(context, &meta);
    let inlines = self.parse_inlines(&mut lines)?;
    self.ctx.subs = restore_subs;

    self.restore_lines(lines);
    let content = if context == Context::Comment {
      // PERF: could squeeze out some speed by not parsing inlines
      Content::Empty(EmptyMetadata::None)
    } else {
      Content::Simple(inlines)
    };

    Ok(Some(Block { meta, context, content }))
  }

  fn parse_quoted_paragraph(
    &mut self,
    mut lines: ContiguousLines<'arena>,
    meta: ChunkMeta<'arena>,
  ) -> Result<Option<Block<'arena>>> {
    let mut attr_line = lines.pop().unwrap();
    attr_line.discard_assert(TokenKind::Dashes); // `--`
    attr_line.discard_assert(TokenKind::Whitespace);
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
    mut lines: ContiguousLines<'arena>,
    meta: ChunkMeta<'arena>,
  ) -> Result<Option<Block<'arena>>> {
    lines.consume_current();
    self.restore_lines(lines);
    Ok(Some(Block {
      meta,
      context,
      content: Content::Empty(EmptyMetadata::None),
    }))
  }

  fn parse_toc_macro(
    &mut self,
    token_loc: SourceLocation,
    lines: ContiguousLines<'arena>,
    meta: ChunkMeta<'arena>,
  ) -> Result<Block<'arena>> {
    self.ctx.saw_toc_macro = true;
    if self.document.toc.is_none() {
      self.err_at(
        "Found macro placing Table of Contents, but TOC not enabled",
        token_loc.start,
        lines.current().unwrap().last_loc().unwrap().end,
      )?;
    }
    Ok(Block {
      meta,
      context: Context::TableOfContents,
      content: Content::Empty(EmptyMetadata::None),
    })
  }
}

// tests

#[cfg(test)]
mod tests {
  use super::*;
  use test_utils::*;

  #[test]
  fn test_parse_doc_attr_entry() {
    let mut parser = test_parser!(":!figure-caption:\n\n");
    let block = parser.parse_block().unwrap().unwrap();
    let expected = Block {
      context: Context::DocumentAttributeDecl,
      content: Content::DocumentAttribute("figure-caption".to_string(), AttrValue::Bool(false)),
      ..empty_block!(0)
    };
    expect_eq!(block, expected);
  }

  #[test]
  fn test_inline_only_parses_single_paragraph() {
    let input = adoc! {"
      first para

      second para (ignored)
    "};

    let mut parser = test_parser!(input);
    parser.apply_job_settings(JobSettings::inline());
    let result = parser.parse().unwrap();
    let blocks = result.document.content.blocks().unwrap();
    expect_eq!(blocks.len(), 1);
    expect_eq!(
      blocks[0],
      Block {
        context: Context::Paragraph,
        content: Content::Simple(just!("first para", 0..10)),
        ..empty_block!(0)
      }
    );
  }

  assert_error!(
    assign_to_header_attr,
    adoc! {"
      para 1

      :doctype: book
    "},
    error! {"
       --> test.adoc:3:1
        |
      3 | :doctype: book
        | ^^^^^^^^^^^^^^ Attribute `doctype` may only be set in the document header
    "}
  );
}
