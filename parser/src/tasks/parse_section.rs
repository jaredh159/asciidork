use crate::internal::*;

impl<'arena> Parser<'arena> {
  pub(crate) fn parse_section(&mut self) -> Result<Option<Section<'arena>>> {
    let Some(mut lines) = self.read_lines()? else {
      return Ok(None);
    };

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

    if level == 0 && self.document.meta.get_doctype() != DocType::Book {
      self.err_line("Level 0 section allowed only in doctype=book", line)?;
    }

    if meta.attrs.has_str_positional("discrete") || meta.attrs.has_str_positional("float") {
      self.restore_peeked(lines, meta);
      return Ok(None);
    }

    let last_level = self.ctx.section_level;
    self.ctx.section_level = level;
    let mut heading_line = lines.consume_current().unwrap();
    let equals = heading_line.consume_current().unwrap();
    heading_line.discard_assert(TokenKind::Whitespace);
    let id = self.section_id(&heading_line, &meta.attrs);

    let out_of_sequence = level > last_level && level - last_level > 1;
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
      self.push_toc_node(level, &heading, id.as_ref());
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
      blocks.push(inner);
    }

    self.ctx.bibliography_ctx = BiblioContext::None;
    self.ctx.section_level = last_level;
    Ok(Some(Section { meta, level, id, heading, blocks }))
  }

  pub fn push_toc_node(
    &mut self,
    level: u8,
    heading: &InlineNodes<'arena>,
    as_ref: Option<&BumpString<'arena>>,
  ) {
    let Some(toc) = self.document.toc.as_mut() else {
      return;
    };
    if level > self.document.meta.u8_or("toclevels", 2) {
      return;
    }
    let mut depth = level;
    let mut nodes: &mut BumpVec<'_, TocNode<'_>> = toc.nodes.as_mut();
    while depth > 1 {
      // we don't push out of sequence sections, shouldn't panic
      nodes = nodes.last_mut().unwrap().children.as_mut();
      depth -= 1;
    }
    nodes.push(TocNode {
      level,
      title: heading.clone(),
      id: as_ref.cloned(),
      children: BumpVec::new_in(self.bump),
    });
  }
}

#[cfg(test)]
mod tests {
  use super::*;
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
          ..empty_block!(8)
        }]
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
          ..empty_block!(21)
        }]
      }
    );
  }
}
