use crate::internal::*;

impl<'arena> Parser<'arena> {
  pub(crate) fn parse_section(&mut self) -> Result<Option<Section<'arena>>> {
    let Some(peeked) = self.peek_section()? else {
      return Ok(None);
    };

    if peeked.semantic_level == 0 && self.document.meta.get_doctype() != DocType::Book {
      self.err_line(
        "Level 0 section allowed only in doctype=book",
        peeked.lines.current().unwrap(),
      )?;
    } else if peeked.semantic_level == 0 {
      self.restore_peeked_section(peeked);
      return Ok(None);
    }
    Ok(Some(self.parse_peeked_section(peeked)?))
  }

  pub(crate) fn peek_section(&mut self) -> Result<Option<PeekedSection<'arena>>> {
    let Some(mut lines) = self.read_lines()? else {
      return Ok(None);
    };

    lines.discard_leading_comment_lines();
    if lines.is_empty() {
      return self.peek_section();
    }

    let meta = self.parse_chunk_meta(&mut lines)?;

    let Some(line) = lines.current() else {
      if !meta.is_empty() {
        self.err_line_starting("Unattached block metadata", meta.start_loc)?;
      }
      self.restore_peeked_meta(meta);
      return Ok(None);
    };

    let Some(level) = self.line_heading_level(line) else {
      self.restore_peeked(lines, meta);
      return Ok(None);
    };

    Ok(Some(PeekedSection {
      meta,
      lines,
      semantic_level: level,
      authored_level: level,
    }))
  }

  pub(crate) fn parse_peeked_section(
    &mut self,
    peeked: PeekedSection<'arena>,
  ) -> Result<Section<'arena>> {
    let special_sect = peeked.special_sect();
    let PeekedSection {
      meta,
      mut lines,
      semantic_level,
      authored_level,
    } = peeked;
    let last_level = self.ctx.section_level;
    self.ctx.section_level = semantic_level;
    let mut heading_line = lines.consume_current().unwrap();
    let mut loc: MultiSourceLocation = heading_line.loc().unwrap().into();
    let equals = heading_line.consume_current().unwrap();
    heading_line.discard_assert(TokenKind::Whitespace);
    let id = self.section_id(&heading_line, &meta.attrs);

    let out_of_sequence = semantic_level > last_level && semantic_level - last_level > 1;
    if out_of_sequence {
      self.err_token_full(
        format!(
          "Section title out of sequence: expected level {} `{}`",
          last_level + 1,
          "=".repeat((last_level + 2) as usize)
        ),
        &equals,
      )?;
    }

    let heading = self.parse_inlines(&mut heading_line.into_lines())?;
    if !out_of_sequence {
      self.push_toc_node(
        authored_level,
        semantic_level,
        &heading,
        id.as_ref(),
        meta.attrs.special_sect(),
      );
    }

    if let Some(id) = &id {
      let reftext = meta
        .attrs
        .iter()
        .find_map(|a| a.named.get("reftext"))
        .cloned();
      self.document.anchors.borrow_mut().insert(
        id.clone(),
        Anchor {
          reftext,
          title: heading.clone(),
          source_loc: None,
          source_idx: self.lexer.source_idx(),
          is_biblio: false,
        },
      );
    }

    if meta.attrs.str_positional_at(0) == Some("bibliography") {
      self.ctx.bibliography_ctx = BiblioContext::Section;
    }

    self.restore_lines(lines);
    let mut blocks = BumpVec::new_in(self.bump);
    while let Some(inner) = self.parse_block()? {
      loc.extend_end(&inner.loc);
      blocks.push(inner);
    }

    if let Some(special_sect) = special_sect {
      if !special_sect.supports_subsections() {
        for block in &blocks {
          if let BlockContent::Section(subsection) = &block.content {
            self.err_line_starting(
              format!(
                "{} sections do not support nested sections",
                special_sect.to_str()
              ),
              subsection.heading.first_loc().unwrap(),
            )?;
          }
        }
      }
    }

    self.ctx.bibliography_ctx = BiblioContext::None;
    self.ctx.section_level = last_level;
    Ok(Section {
      meta,
      level: semantic_level,
      id,
      heading,
      blocks,
      loc,
    })
  }

  pub(crate) fn restore_peeked_section(&mut self, peeked: PeekedSection<'arena>) {
    self.restore_peeked(peeked.lines, peeked.meta);
  }

  pub fn push_toc_node(
    &mut self,
    authored_level: u8,
    semantic_level: u8,
    heading: &InlineNodes<'arena>,
    as_ref: Option<&BumpString<'arena>>,
    special_sect: Option<SpecialSection>,
  ) {
    let Some(toc) = self.document.toc.as_mut() else {
      return;
    };
    if authored_level > self.document.meta.u8_or("toclevels", 2) {
      return;
    }
    let node = TocNode {
      level: authored_level,
      title: heading.clone(),
      id: as_ref.cloned(),
      special_sect,
      children: BumpVec::new_in(self.bump),
    };
    let mut nodes: &mut BumpVec<'_, TocNode<'_>> = toc.nodes.as_mut();
    let Some(last_level) = nodes.last().map(|n| n.level) else {
      nodes.push(node);
      return;
    };
    if authored_level < last_level || authored_level == 0 {
      nodes.push(node);
      return;
    }

    let mut depth = semantic_level;

    // special case: book special sections can go from 0 to 2
    if last_level == 0
      && semantic_level == 2
      && !nodes.last().unwrap().children.iter().any(|n| n.level == 1)
    {
      depth = 1;
    }

    while depth > last_level {
      // we don't push out of sequence sections, shouldn't panic
      nodes = nodes.last_mut().unwrap().children.as_mut();
      depth -= 1;
    }
    nodes.push(node);
  }
}

#[derive(Debug)]
pub struct PeekedSection<'arena> {
  pub meta: ChunkMeta<'arena>,
  pub lines: ContiguousLines<'arena>,
  pub semantic_level: u8,
  pub authored_level: u8,
}

impl PeekedSection<'_> {
  pub fn is_special_sect(&self) -> bool {
    self.meta.attrs.special_sect().is_some()
  }

  pub fn special_sect(&self) -> Option<SpecialSection> {
    self.meta.attrs.special_sect()
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use pretty_assertions::assert_eq;
  use test_utils::*;

  #[test]
  fn test_parse_2_sections() {
    let input = adoc! {"
      == one

      foo

      == two

      bar
    "};
    let mut parser = test_parser!(input);
    let section = parser.parse_section().unwrap().unwrap();
    assert_eq!(
      section,
      Section {
        meta: chunk_meta!(0),
        level: 1,
        id: Some(bstr!("_one")),
        heading: nodes![node!("one"; 3..6)],
        blocks: vecb![Block {
          context: BlockContext::Paragraph,
          content: BlockContent::Simple(nodes![node!("foo"; 8..11)]),
          loc: (8..11).into(),
          ..empty_block!(8)
        }],
        loc: (0..11).into()
      }
    );
    let section = parser.parse_section().unwrap().unwrap();
    assert_eq!(
      section,
      Section {
        meta: chunk_meta!(13),
        level: 1,
        id: Some(bstr!("_two")),
        heading: nodes![node!("two"; 16..19)],
        blocks: vecb![Block {
          context: BlockContext::Paragraph,
          content: BlockContent::Simple(nodes![node!("bar"; 21..24)]),
          loc: (21..24).into(),
          ..empty_block!(21)
        }],
        loc: (13..24).into()
      }
    );
  }
}
