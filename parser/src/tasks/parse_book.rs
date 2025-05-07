use crate::internal::*;

impl<'arena> Parser<'arena> {
  pub(crate) fn parse_book(&mut self) -> Result<Option<DocContent<'arena>>> {
    if self.document.meta.get_doctype() != DocType::Book {
      return Ok(None);
    }

    let mut opening_special_sects = bvec![in self.bump];
    while let Some(special) = self.parse_special_sect()? {
      opening_special_sects.push(special);
    }
    if self.lexer.is_eof() && !opening_special_sects.is_empty() {
      return Ok(Some(DocContent::Sections(Sectioned {
        preamble: None,
        sections: opening_special_sects,
      })));
    }

    if let Some(part) = self.parse_book_part()? {
      let mut parts = bvec![in self.bump; part];
      while let Some(part) = self.parse_book_part()? {
        parts.push(part);
      }
      let mut closing_special_sects = bvec![in self.bump];
      while let Some(special) = self.parse_special_sect()? {
        closing_special_sects.push(special);
      }
      return Ok(Some(DocContent::Parts(MultiPartBook {
        preamble: None,
        opening_special_sects,
        parts,
        closing_special_sects,
      })));
    }

    if let Some(peeked) = self.peek_section()? {
      self.restore_peeked_section(peeked);
      let mut sectioned = self.parse_sectioned()?;
      if !opening_special_sects.is_empty() {
        opening_special_sects.extend(sectioned.sections);
        sectioned.sections = opening_special_sects;
      }
      return Ok(Some(DocContent::Sections(sectioned)));
    }

    let mut blocks = bvec![in self.bump];
    while let Some(block) = self.parse_block()? {
      blocks.push(block);
    }
    if self.lexer.is_eof() {
      return Ok(Some(DocContent::Blocks(blocks)));
    }

    // NB: the recursion here is to deal with malformed books
    // like a preamble before multi-part sections
    match self.parse_book()? {
      None => Ok(Some(DocContent::Blocks(blocks))),
      Some(DocContent::Sections(sectioned)) => {
        if let Some(preamble) = sectioned.preamble {
          blocks.extend(preamble);
        }
        Ok(Some(DocContent::Sections(Sectioned {
          preamble: (!blocks.is_empty()).then_some(blocks),
          sections: sectioned.sections,
        })))
      }
      Some(DocContent::Blocks(_)) => unreachable!(),
      Some(DocContent::Parts(book)) => {
        if let Some(preamble) = book.preamble {
          blocks.extend(preamble);
        }
        Ok(Some(DocContent::Parts(MultiPartBook {
          preamble: (!blocks.is_empty()).then_some(blocks),
          ..book
        })))
      }
    }
  }

  fn parse_special_sect(&mut self) -> Result<Option<Section<'arena>>> {
    let Some(mut peeked) = self.peek_section()? else {
      return Ok(None);
    };
    if !peeked.is_special_sect() {
      self.restore_peeked_section(peeked);
      return Ok(None);
    }

    peeked.semantic_level = 1; // written as level 0
    let section = self.parse_peeked_section(peeked)?;

    Ok(Some(section))
  }
}
