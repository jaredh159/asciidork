use crate::internal::*;
use crate::variants::token::*;

impl<'arena> Parser<'arena> {
  pub(crate) fn parse_list(
    &mut self,
    mut lines: ContiguousLines<'arena>,
    meta: Option<ChunkMeta<'arena>>,
  ) -> Result<Block<'arena>> {
    let first_line = lines.consume_current().unwrap();
    let start_loc = first_line.first_loc().unwrap();
    let mut last_loc = first_line.last_loc().unwrap();
    let marker = first_line.list_marker().unwrap();
    let variant = ListVariant::from(marker);
    lines.restore_if_nonempty(first_line);
    self.restore_lines(lines);

    self.ctx.list.stack.push(marker);
    let depth = self.ctx.list.stack.depth();
    let mut items = BumpVec::new_in(self.bump);
    let mut auto_conum = 1;

    if variant == ListVariant::Unordered
      && (self.ctx.bibliography_ctx == BiblioContext::Section
        || meta
          .as_ref()
          .is_some_and(|meta| meta.attrs.has_str_positional("bibliography")))
    {
      self.ctx.bibliography_ctx = BiblioContext::List;
    }

    while let Some(item) = self.parse_list_item(variant, &mut auto_conum)? {
      last_loc = item.last_loc().unwrap_or(last_loc);
      items.push(item);
    }

    if self.ctx.bibliography_ctx == BiblioContext::List {
      self.ctx.bibliography_ctx = BiblioContext::Section;
    }
    self.ctx.list.stack.pop();
    if variant == ListVariant::Callout {
      self.ctx.advance_callout_list(self.bump);
    }

    Ok(Block {
      meta: meta.unwrap_or_else(|| ChunkMeta::empty(start_loc.clamp_start(), self.bump)),
      context: variant.to_context(),
      content: BlockContent::List { variant, depth, items },
      loc: MultiSourceLocation::spanning(start_loc, last_loc),
    })
  }

  fn parse_list_item(
    &mut self,
    list_variant: ListVariant,
    autogen_conum: &mut u8,
  ) -> Result<Option<ListItem<'arena>>> {
    let Some(mut lines) = self.read_lines()? else {
      return Ok(None);
    };

    let Some(marker) = lines.current().and_then(|line| line.list_marker()) else {
      self.restore_lines(lines);
      return Ok(None);
    };

    if !self.ctx.list.stack.continues_current_list(marker) {
      self.restore_lines(lines);
      return Ok(None);
    }

    let mut line = lines.consume_current().unwrap();
    line.trim_leading_whitespace();
    if list_variant == ListVariant::Description {
      return self.parse_description_list_item(marker, line, lines);
    }

    let marker_src = line.consume_to_string_until(Whitespace, self.bump);
    line.discard_assert(Whitespace);
    let mut type_meta = ListItemTypeMeta::None;
    if list_variant == ListVariant::Unordered {
      if let Some(checklist) = line.consume_checklist_item(self.bump) {
        type_meta = ListItemTypeMeta::Checklist(checklist.0, checklist.1);
      }
    } else if list_variant == ListVariant::Callout {
      let conum = marker.callout_num().unwrap_or(*autogen_conum);
      if conum != *autogen_conum {
        self.err_at(
          format!("Unexpected callout number, expected `<{autogen_conum}>`"),
          marker_src.loc,
        )?;
      }
      let callouts = self.ctx.get_callouts(conum);
      if callouts.is_empty() {
        self.err_at(
          format!("No callout found for number `{conum}`"),
          marker_src.loc,
        )?;
      }
      type_meta = ListItemTypeMeta::Callout(callouts);
      *autogen_conum = conum + 1;
    }

    let mut item_lines = Deq::with_capacity(1, self.bump);
    if !line.is_empty() {
      item_lines.push(line);
    }
    while lines
      .current()
      .map(|line| line.continues_list_item_principle())
      .unwrap_or(false)
    {
      let mut line = lines.consume_current().unwrap();
      line.discard_leading_whitespace();
      item_lines.push(line);
    }

    let mut item_lines = ContiguousLines::new(item_lines);
    let principle = self.parse_inlines(&mut item_lines)?;
    let blocks = self.parse_list_item_blocks(lines, BumpVec::new_in(self.bump))?;

    Ok(Some(ListItem {
      blocks,
      marker,
      marker_src,
      type_meta,
      principle,
    }))
  }

  fn parse_list_item_blocks(
    &mut self,
    lines: ContiguousLines<'arena>,
    mut blocks: BumpVec<'arena, Block<'arena>>,
  ) -> Result<BumpVec<'arena, Block<'arena>>> {
    if lines.starts_nested_list(&self.ctx.list.stack, true) {
      self.restore_lines(lines);
      if let Some(block) = self.parse_block()? {
        blocks.push(block);
      }
      return Ok(blocks);
    }

    if lines.starts_list_continuation() {
      self.restore_lines(lines);
      return self.parse_list_continuation_blocks(blocks);
    }

    if !lines.is_empty() {
      self.restore_lines(lines);
      return Ok(blocks);
    }

    let Some(lines) = self.read_lines()? else {
      return Ok(blocks);
    };

    // ELSE IF the next Contiguous Lines starts a NESTED list, parse a block
    if lines.starts_nested_list(&self.ctx.list.stack, false) {
      blocks.push(self.parse_list(lines, None)?);
      return Ok(blocks);
    }

    self.restore_lines(lines);
    Ok(blocks)
  }

  fn parse_list_continuation_blocks(
    &mut self,
    mut accum: BumpVec<'arena, Block<'arena>>,
  ) -> Result<BumpVec<'arena, Block<'arena>>> {
    let Some(mut lines) = self.read_lines()? else {
      return Ok(accum);
    };

    if lines.starts_nested_list(&self.ctx.list.stack, false) {
      accum.push(self.parse_list(lines, None)?);
      return Ok(accum);
    }

    if !lines.starts_list_continuation() {
      self.restore_lines(lines);
      return Ok(accum);
    }

    if let Some(marker) = lines.nth(1).and_then(|line| line.list_marker()) {
      if self.ctx.list.stack.continues_current_list(marker) {
        self.err_line("Dangling list continuation", lines.current().unwrap())?;
        lines.consume_current();
        self.restore_lines(lines);
        return Ok(accum);
      }
    }

    lines.consume_current(); // the `+` line starting the continuation
    self.restore_lines(lines);
    self.ctx.list.parsing_continuations = true;
    if let Some(block) = self.parse_block()? {
      accum.push(block);
    }
    self.ctx.list.parsing_continuations = false;
    self.parse_list_continuation_blocks(accum)
  }

  fn parse_description_list_item(
    &mut self,
    marker: ListMarker,
    mut line: Line<'arena>,
    mut lines: ContiguousLines<'arena>,
  ) -> Result<Option<ListItem<'arena>>> {
    let (principle, marker_src) = self.parse_description_list_term(&mut line)?;
    let mut extra_terms = BumpVec::new_in(self.bump);

    if line.is_empty() {
      self.restore_lines(lines);
      self.gather_extra_terms(marker, &mut extra_terms)?;
    } else {
      lines.restore_if_nonempty(line);
      self.restore_lines(lines);
    }

    let description = self
      .parse_block()?
      .filter(|block| block.context != BlockContext::Comment);

    Ok(Some(ListItem {
      blocks: self.parse_description_list_item_blocks()?,
      marker,
      marker_src,
      type_meta: ListItemTypeMeta::DescList { description, extra_terms },
      principle,
    }))
  }

  fn parse_description_list_term(
    &mut self,
    line: &mut Line<'arena>,
  ) -> Result<(InlineNodes<'arena>, SourceString<'arena>)> {
    let principle = {
      let before_delim = line.extract_line_before(&[Kind(TermDelimiter)]);
      self.parse_inlines(&mut before_delim.into_lines())?
    };

    let marker_token = line.consume_current().unwrap();
    let marker_src = marker_token.into_source_string();
    line.trim_leading_whitespace();
    Ok((principle, marker_src))
  }

  fn gather_extra_terms(
    &mut self,
    marker: ListMarker,
    terms: &mut BumpVec<'arena, (InlineNodes<'arena>, SourceString<'arena>)>,
  ) -> Result<()> {
    let Some(mut lines) = self.read_lines()? else {
      return Ok(());
    };
    if !lines.starts_extra_description_list_term(marker) {
      self.restore_lines(lines);
      return Ok(());
    }
    lines.discard_leading_comment_lines();
    let mut line = lines.consume_current().unwrap();
    terms.push(self.parse_description_list_term(&mut line)?);
    lines.restore_if_nonempty(line);
    self.restore_lines(lines);
    self.gather_extra_terms(marker, terms)
  }

  fn parse_description_list_item_blocks(&mut self) -> Result<BumpVec<'arena, Block<'arena>>> {
    let mut blocks = BumpVec::new_in(self.bump);
    let Some(lines) = self.read_lines()? else {
      return Ok(blocks);
    };
    if lines.starts_list_continuation() {
      self.restore_lines(lines);
      blocks = self.parse_list_continuation_blocks(blocks)?;
    } else if lines.starts_nested_list(&self.ctx.list.stack, true) {
      self.restore_lines(lines);
      if let Some(block) = self.parse_block()? {
        blocks.push(block);
      }
    } else {
      self.restore_lines(lines);
    }
    Ok(blocks)
  }
}

// tests

#[cfg(test)]
mod tests {
  use super::*;
  use test_utils::*;

  #[test]
  fn test_list_separation() {
    use BlockContext::*;
    let cases: Vec<(&str, &[BlockContext])> = vec![
      ("* foo\n\n[]\n* bar", &[UnorderedList, UnorderedList]),
      ("* foo\n\n\n* bar", &[UnorderedList]),
      ("* foo\n\n\n** bar", &[UnorderedList]),
      ("[square]\n* foo\n[circle]\n** bar", &[UnorderedList]),
      (
        "* foo\n\n//-\n\n. bar",
        &[UnorderedList, Comment, OrderedList],
      ),
      (
        "* foo\n\n//-\n\n* bar",
        &[UnorderedList, Comment, UnorderedList],
      ),
    ];

    for (input, block_contexts) in cases {
      let parser = test_parser!(input);
      let content = parser.parse().unwrap().document.content;
      match content {
        DocContent::Blocks(blocks) => {
          expect_eq!( blocks.len(), block_contexts.len(), from: input);
          for (block, context) in blocks.iter().zip(block_contexts.iter()) {
            expect_eq!(block.context, *context, from: input);
          }
        }
        _ => panic!("expected blocks, got {content:?}"),
      }
    }
  }
}
