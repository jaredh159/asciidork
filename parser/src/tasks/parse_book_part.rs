use bumpalo::collections::CollectIn;

use crate::internal::*;

impl<'arena> Parser<'arena> {
  pub(crate) fn parse_book_part(&mut self) -> Result<Option<Part<'arena>>> {
    let Some(peeked) = self.peek_section()? else {
      return Ok(None);
    };

    if peeked.authored_level != 0 || peeked.is_special_sect() {
      self.restore_peeked_section(peeked);
      return Ok(None);
    }

    let PeekedSection {
      meta,
      mut lines,
      authored_level,
      semantic_level,
    } = peeked;
    let mut heading_line = lines.consume_current().unwrap();
    let equals = heading_line.consume_current().unwrap();
    heading_line.discard_assert(TokenKind::Whitespace);
    let id = self.section_id(&heading_line, &meta.attrs);
    let heading = self.parse_inlines(&mut heading_line.into_lines())?;
    self.push_toc_node(
      authored_level,
      semantic_level,
      &heading,
      id.as_ref(),
      meta.attrs.special_sect(),
    );

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

    let mut title = PartTitle { meta, text: heading, id };
    let Sectioned { mut preamble, sections } = self.parse_sectioned()?;

    if sections.is_empty() {
      self.err_line_starting(
        "Invalid empty book part, must have at least one section",
        equals.loc,
      )?;
    }

    // asciidoctor wraps the part intro in an open block, so if it is manually
    // wrapped already in a single open block, transpose it to be the preamble
    if let Some(
      [Block {
        content: BlockContent::Compound(blocks),
        context: BlockContext::Open,
        ..
      }],
    ) = preamble.as_deref_mut()
    {
      preamble = Some(blocks.drain(..).collect_in(self.bump));
    }

    // match asciidoctor behavior of hoisting the title from the first preamble block
    // @see https://github.com/asciidoctor/asciidoctor/issues/4450
    if let Some(ref mut preamble) = preamble {
      let intro_title = preamble
        .first_mut()
        .filter(|block| block.meta.attrs.has_str_positional("partintro"))
        .and_then(|block| block.meta.title.take());
      if title.meta.title.is_none() {
        title.meta.title = intro_title;
      }
    }

    Ok(Some(Part { title, intro: preamble, sections }))
  }
}
