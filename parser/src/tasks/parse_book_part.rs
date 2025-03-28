use crate::internal::*;

impl<'arena> Parser<'arena> {
  pub(crate) fn parse_book_part(&mut self) -> Result<Option<Part<'arena>>> {
    // very dupy...
    let Some(mut lines) = self.read_lines()? else {
      println!("No lines");
      return Ok(None);
    };

    let meta = self.parse_chunk_meta(&mut lines)?;
    lines.debug_print();

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
    // ...until here

    if level != 0 {
      self.restore_peeked(lines, meta);
      return Ok(None);
    }

    dbg!(level);

    // kinda dupy here...
    let mut heading_line = lines.consume_current().unwrap();
    let _equals = heading_line.consume_current().unwrap();
    heading_line.discard_assert(TokenKind::Whitespace);
    let id = self.section_id(&heading_line, &meta.attrs);
    let heading = self.parse_inlines(&mut heading_line.into_lines())?;
    dbg!(&id);

    // this is also muy dupy
    if let Some(id) = &id {
      self.document.anchors.borrow_mut().insert(
        id.clone(),
        Anchor {
          reftext: None,
          title: heading.clone(),
          source_loc: None,
          source_idx: self.lexer.source_idx(),
          is_biblio: false,
        },
      );
    }

    let sectioned = self.parse_sectioned()?;

    Ok(Some(Part {
      title: PartTitle { attrs: meta.attrs, text: heading, id },
      intro: sectioned.preamble,
      sections: sectioned.sections,
    }))
  }
}
