use crate::internal::*;

impl<'arena> Parser<'arena> {
  pub(crate) fn parse_book_part(&mut self) -> Result<Option<Part<'arena>>> {
    // very dupy...
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
    // ...until here

    if level != 0 {
      self.restore_peeked(lines, meta);
      return Ok(None);
    }

    // dbg!(level);

    // kinda dupy here...
    let mut heading_line = lines.consume_current().unwrap();
    let _equals = heading_line.consume_current().unwrap();
    heading_line.discard_assert(TokenKind::Whitespace);
    let id = self.section_id(&heading_line, &meta.attrs);
    let heading = self.parse_inlines(&mut heading_line.into_lines())?;
    // dbg!(&id);

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

    let title = PartTitle { meta, text: heading, id };
    Ok(Some(self.hoist_intro_title(title)?))
    // let
    // Ok(Some(Part { title, intro: preamble, sections }))
  }

  // match asciidoctor behavior of hoisting the title from the first preamble block
  // @see https://github.com/asciidoctor/asciidoctor/issues/4450
  fn hoist_intro_title(&mut self, mut title: PartTitle<'arena>) -> Result<Part<'arena>> {
    let Sectioned { mut preamble, sections } = self.parse_sectioned()?;
    if let Some(ref mut preamble) = preamble {
      let intro_title = preamble
        .first_mut()
        .and_then(|block| block.meta.title.take());
      if title.meta.title.is_none() {
        title.meta.title = intro_title;
      }
    }
    Ok(Part { title, intro: preamble, sections })
  }
}
