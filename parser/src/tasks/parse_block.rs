use crate::internal::*;
use crate::variants::token::*;
use ast::short::block::*;

impl<'arena> Parser<'arena> {
  pub(crate) fn parse_block(&mut self) -> Result<Option<Block<'arena>>> {
    let Some(mut lines) = self.read_lines()? else {
      return Ok(None);
    };

    if let Some(comment_block) = self.parse_inline_comment_block(&mut lines) {
      if let Some(meta) = self.peeked_meta.take() {
        assert!(meta.is_empty());
      }
      self.restore_lines(lines);
      return Ok(Some(comment_block));
    }

    let meta = self.parse_chunk_meta(&mut lines)?;
    if lines.is_empty() {
      self.err_line_starting("Unattached block metadata", meta.start_loc)?;
      return self.parse_block();
    }

    match self.section_start_level(&lines, &meta) {
      Some(0 | 1) if self.ctx.delimiter.is_some() => {}
      // skip doc title and top-level sections
      Some(0 | 1) => {
        self.restore_peeked(lines, meta);
        return Ok(None);
      }
      Some(level) => {
        self.restore_peeked(lines, meta);
        if level <= self.ctx.section_level {
          return Ok(None);
        } else {
          let section = self.parse_section()?.unwrap();
          let mut loc: MultiSourceLocation = section.meta.start_loc.into();
          if let Some(sec_loc) = section.blocks.last().map(|b| &b.loc) {
            loc.extend_end(sec_loc);
          }
          return Ok(Some(Block {
            meta: ChunkMeta::empty(section.meta.start_loc, self.bump),
            context: Context::Section,
            content: Content::Section(section),
            loc,
          }));
        }
      }
      None => {}
    }

    let first_token = lines.current_token().unwrap();

    if lines.is_block_macro() {
      return match first_token.lexeme.as_str() {
        "image:" => self.parse_image_block(lines, meta),
        "toc:" => self.parse_toc_macro(lines, meta),
        _ => self.parse_plugin_block_macro(lines, meta),
      }
      .map(Some);
    } else if lines.starts_list() {
      return self.parse_list(lines, Some(meta)).map(Some);
    } else if lines.current_satisfies(|l| l.is_heading())
      && (meta.attrs.has_str_positional("discrete") || meta.attrs.has_str_positional("float"))
    {
      return self.parse_discrete_heading(lines, meta).map(Some);
    }

    match first_token.kind {
      DelimiterLine
        if self.ctx.delimiter.is_some() && self.ctx.delimiter == first_token.to_delimiter() =>
      {
        self.restore_lines(lines);
        return Ok(None);
      }
      DelimiterLine => {
        return self.parse_delimited_block(lines, meta);
      }
      Pipe | Colon | Bang | Comma
        if lines.nth_token(1).kind(EqualSigns)
          && lines.nth_token(2).is_none()
          && lines.nth_token(1).unwrap().len() > 2 =>
      {
        return Ok(Some(self.parse_table(lines, meta)?));
      }
      Colon => {
        if let Some((key, value, end_loc)) = self.parse_doc_attr(&mut lines, false)? {
          self.restore_lines(lines);
          let attr_loc = meta.start_loc.setting_end(end_loc.end);
          if let Err(err) = self.document.meta.insert_doc_attr(&key, value.clone()) {
            self.err_at(err, attr_loc)?;
          }
          return Ok(Some(Block {
            meta,
            context: Context::DocumentAttributeDecl,
            content: Content::DocumentAttribute(key, value),
            loc: attr_loc.into(),
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
    let line_end_loc = line.last_loc().unwrap();
    let level = self.line_heading_level(&line).unwrap();
    line.discard_assert(TokenKind::EqualSigns);
    line.discard_assert(TokenKind::Whitespace);
    let id = self.section_id(&line, &meta.attrs);
    let content = self.parse_inlines(&mut line.into_lines())?;
    self.restore_lines(lines);
    Ok(Block {
      context: Context::DiscreteHeading,
      content: Content::Empty(EmptyMetadata::DiscreteHeading { level, content, id }),
      loc: MultiSourceLocation::spanning(meta.start_loc, line_end_loc),
      meta,
    })
  }

  // important to represent these as an ast node because
  // they are the documented way to separate adjacent lists
  fn parse_inline_comment_block(
    &mut self,
    lines: &mut ContiguousLines<'arena>,
  ) -> Option<Block<'arena>> {
    if lines.starts_with_comment_line() {
      let start_loc = lines.current_token().unwrap().loc.clamp_start();
      let mut end_loc = lines.consume_current().unwrap().last_loc().unwrap();
      if let Some(final_loc) = lines.discard_leading_comment_lines() {
        end_loc = final_loc;
      }
      lines.discard_leading_empty_lines();
      if lines.is_empty() {
        return Some(Block {
          meta: ChunkMeta::empty(start_loc, self.bump),
          context: Context::Comment,
          content: Content::Empty(EmptyMetadata::None),
          loc: MultiSourceLocation::spanning(start_loc, end_loc),
        });
      }
    }
    None
  }

  fn parse_delimited_block(
    &mut self,
    mut lines: ContiguousLines<'arena>,
    meta: ChunkMeta<'arena>,
  ) -> Result<Option<Block<'arena>>> {
    let open_token = lines.consume_current_token().unwrap();
    let delimiter = open_token.to_delimiter().unwrap();
    let prev = self.ctx.delimiter;
    self.ctx.delimiter = Some(delimiter);
    self.restore_lines(lines);
    let context = meta.block_style_or(Context::from(delimiter.kind));
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
        .attrs
        .named("indent")
        .and_then(|s| s.parse::<usize>().ok())
      {
        let delimiter = lines.pop();
        lines.set_indentation(indent);
        delimiter.map(|d| lines.push(d));
      }

      if context == Context::Listing || context == Context::Literal {
        if let Some(comment) = meta.attrs.named("line-comment") {
          self.ctx.custom_line_comment = Some(SmallVec::from_slice(comment.as_bytes()));
        }
      }

      if context == Context::Comment {
        let start_loc = lines.first_loc().unwrap_or(open_token.loc);
        let mut end_loc = lines.last_loc().unwrap_or(open_token.loc);
        lines.discard_until(|l| l.is_delimiter(delimiter));
        end_loc = lines.first_loc().unwrap_or(end_loc);
        self.restore_lines(lines);
        if start_loc.include_depth != end_loc.include_depth {
          Content::Empty(EmptyMetadata::None)
        } else {
          let span_loc = SourceLocation::spanning(start_loc.clamp_start(), end_loc.clamp_start());
          let comment = self.lexer.src_string_from_loc(span_loc);
          Content::Empty(EmptyMetadata::Comment(comment))
        }
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
    let mut end_loc = None;
    if let Some(mut lines) = self.read_lines()? {
      if lines.current_satisfies(|l| l.is_delimiter(self.ctx.delimiter.unwrap())) {
        let token = lines.consume_current_token().unwrap();
        self.restore_lines(lines);
        end_loc = Some(token.loc);
      }
    }

    if end_loc.is_none() {
      self.err_token_full("This delimiter was never closed", &open_token)?;
      end_loc = Some(self.lexer.loc());
    };

    self.ctx.delimiter = prev;
    Ok(Some(Block {
      content,
      context,
      loc: MultiSourceLocation::spanning(open_token.loc, end_loc.unwrap()),
      meta,
    }))
  }

  fn parse_image_block(
    &mut self,
    mut lines: ContiguousLines<'arena>,
    meta: ChunkMeta<'arena>,
  ) -> Result<Block<'arena>> {
    let mut line = lines.consume_current().unwrap();
    let loc = line.loc().unwrap();
    line.discard_assert(MacroName);
    line.discard_assert(Colon);
    let target = line.consume_macro_target(self.bump);
    let attrs = self.parse_block_attr_list(&mut line)?;
    Ok(Block {
      meta,
      context: Context::Image,
      content: Content::Empty(EmptyMetadata::Image { target, attrs }),
      loc: loc.into(),
    })
  }

  fn parse_paragraph(
    &mut self,
    mut lines: ContiguousLines<'arena>,
    meta: ChunkMeta<'arena>,
  ) -> Result<Option<Block<'arena>>> {
    if self.ctx.parsing_simple_desc_def() {
      lines.current_mut().map(|l| l.discard_leading_whitespace());
    }
    let context = meta.block_paragraph_context(&mut lines);
    // TODO: probably a better stack-like context API is possible here...
    let restore_subs = self.ctx.set_subs_for(context, &meta);
    let inlines = self.parse_inlines(&mut lines)?;
    self.ctx.subs = restore_subs;

    // can be empty when parsing empty desc list principal
    let Some(loc) = inlines.loc() else {
      self.restore_lines(lines);
      return Ok(None);
    };

    self.restore_lines(lines);
    let content = if context == Context::Comment {
      // PERF: could squeeze out some speed by not parsing inlines
      Content::Empty(EmptyMetadata::None)
    } else {
      Content::Simple(inlines)
    };

    let paragraph = Block { meta, context, content, loc };
    Ok(Some(paragraph))
  }

  fn parse_quoted_paragraph(
    &mut self,
    mut lines: ContiguousLines<'arena>,
    meta: ChunkMeta<'arena>,
  ) -> Result<Option<Block<'arena>>> {
    let mut attr_line = lines.pop().unwrap();
    attr_line.discard_assert(TokenKind::Dashes); // `--`
    attr_line.discard_assert(TokenKind::Whitespace);
    let last_loc = attr_line.last_loc().unwrap();
    let (attr, cite) = attr_line
      .consume_to_string(self.bump)
      .split_once(", ", self.bump);
    let start_token = lines
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
      loc: MultiSourceLocation::spanning(start_token.loc, last_loc),
    }))
  }

  fn parse_break(
    &mut self,
    context: BlockContext,
    mut lines: ContiguousLines<'arena>,
    meta: ChunkMeta<'arena>,
  ) -> Result<Option<Block<'arena>>> {
    let line_loc = lines.consume_current().unwrap().last_loc().unwrap();
    self.restore_lines(lines);
    Ok(Some(Block {
      context,
      content: Content::Empty(EmptyMetadata::None),
      loc: MultiSourceLocation::spanning(meta.start_loc, line_loc),
      meta,
    }))
  }

  fn parse_toc_macro(
    &mut self,
    mut lines: ContiguousLines<'arena>,
    meta: ChunkMeta<'arena>,
  ) -> Result<Block<'arena>> {
    self.ctx.saw_toc_macro = true;
    let line = lines.consume_current().unwrap();
    self.restore_lines(lines);
    if self.document.toc.is_none() {
      self.err_line(
        "Found macro placing Table of Contents, but TOC not enabled",
        &line,
      )?;
    }
    Ok(Block {
      meta,
      context: Context::TableOfContents,
      content: Content::Empty(EmptyMetadata::None),
      loc: line.loc().unwrap().into(),
    })
  }

  fn parse_plugin_block_macro(
    &mut self,
    mut lines: ContiguousLines<'arena>,
    meta: ChunkMeta<'arena>,
  ) -> Result<Block<'arena>> {
    let mut line = lines.consume_current().unwrap();
    let line_loc = line.loc().unwrap();
    let source = line.reassemble_src();
    let mut name = line.discard_assert(MacroName).lexeme;
    name.pop(); // remove trailing colon
    line.discard_assert(Colon);
    let target = if !line.current_is(OpenBracket) {
      Some(line.consume_macro_target(self.bump))
    } else {
      line.discard_assert(OpenBracket);
      None
    };
    let attrs = self.parse_block_attr_list(&mut line)?;
    let mut nodes = InlineNodes::new(self.bump);
    nodes.push(InlineNode {
      content: Inline::Macro(MacroNode::Plugin(PluginMacro {
        name,
        target,
        flow: Flow::Block,
        attrs,
        source: SourceString::new(source, line_loc),
      })),
      loc: line_loc,
    });
    Ok(Block {
      meta,
      context: Context::Paragraph,
      content: Content::Simple(nodes),
      loc: line_loc.into(),
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
      ..empty_block!(0, 17)
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
        loc: (0..10).into(),
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
