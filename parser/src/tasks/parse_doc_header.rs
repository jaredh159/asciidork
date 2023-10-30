use std::collections::HashMap;

use bumpalo::vec as bump_vec;

use crate::ast::{DocHeader, DocTitle};
use crate::block::Block;
use crate::token::TokenKind::*;
use crate::{Parser, Result};

impl<'bmp, 'src> Parser<'bmp, 'src> {
  pub(crate) fn parse_document_header(&mut self) -> Result<Option<DocHeader<'bmp>>> {
    let Some(mut block) = self.read_block() else {
      return Ok(None);
    };

    if !is_doc_header(&block) {
      self.peeked_block = Some(block);
      return Ok(None);
    }

    // remove all comment lines
    block.lines.retain(|line| !line.starts(CommentLine));

    let mut doc_header = DocHeader {
      title: None,
      authors: bump_vec![in self.bump],
      revision: None,
      attrs: HashMap::new(),
    };

    self.parse_doc_title_author_revision(&mut block, &mut doc_header)?;
    self.parse_doc_attrs(&mut block, &mut doc_header.attrs)?;
    Ok(Some(doc_header))
  }

  fn parse_doc_title_author_revision(
    &mut self,
    block: &mut Block<'bmp, 'src>,
    doc_header: &mut DocHeader<'bmp>,
  ) -> Result<()> {
    let first_line = block.current_line().expect("non-empty doc header");
    if !first_line.is_header(1) {
      // author and revision must follow doc title, so if non title, skip
      return Ok(());
    }

    let mut header_line = block.consume_current().unwrap();
    debug_assert!(header_line.starts_with_seq(&[EqualSigns, Whitespace]));
    header_line.discard(2);

    doc_header.title = Some(DocTitle {
      heading: self.parse_inlines(header_line.into_block_in(self.bump))?,
      subtitle: None, // todo
    });

    if block.current_line_starts_with(Word) {
      self.parse_author_line(block.consume_current().unwrap(), &mut doc_header.authors)?;
      // revision line can only follow an author line (and requires a doc header)
      if !doc_header.authors.is_empty() {
        self.parse_revision_line(block, &mut doc_header.revision);
      }
    }

    Ok(())
  }
}

pub fn is_doc_header(block: &Block) -> bool {
  for line in &block.lines {
    if line.is_header(1) || line.starts_with_seq(&[Colon, Word, Colon]) {
      return true;
    }
  }
  false
}
