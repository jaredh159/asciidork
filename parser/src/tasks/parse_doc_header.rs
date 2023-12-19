use std::collections::HashMap;

use crate::prelude::*;
use crate::variants::token::*;

impl<'bmp, 'src> Parser<'bmp, 'src> {
  pub(crate) fn parse_document_header(&mut self) -> Result<Option<DocHeader<'bmp>>> {
    let Some(mut block) = self.read_lines() else {
      return Ok(None);
    };

    if !is_doc_header(&block) {
      self.peeked_lines = Some(block);
      return Ok(None);
    }

    // remove all comment lines
    block.retain(|line| !line.starts(CommentLine));

    let mut doc_header = DocHeader {
      title: None,
      authors: bvec![in self.bump],
      revision: None,
      attrs: HashMap::new(),
    };

    self.parse_doc_title_author_revision(&mut block, &mut doc_header)?;
    self.parse_doc_attrs(&mut block, &mut doc_header.attrs)?;
    Ok(Some(doc_header))
  }

  fn parse_doc_title_author_revision(
    &mut self,
    lines: &mut ContiguousLines<'bmp, 'src>,
    doc_header: &mut DocHeader<'bmp>,
  ) -> Result<()> {
    let first_line = lines.current().expect("non-empty doc header");
    if !first_line.is_header(1) {
      // author and revision must follow doc title, so if non title, skip
      return Ok(());
    }

    let mut header_line = lines.consume_current().unwrap();
    debug_assert!(header_line.starts_with_seq(&[EqualSigns, Whitespace]));
    header_line.discard(2);

    doc_header.title = Some(DocTitle {
      heading: self.parse_inlines(&mut header_line.into_lines_in(self.bump))?,
      subtitle: None, // todo
    });

    if lines.current_starts_with(Word) {
      self.parse_author_line(lines.consume_current().unwrap(), &mut doc_header.authors)?;
      // revision line can only follow an author line (and requires a doc header)
      if !doc_header.authors.is_empty() {
        self.parse_revision_line(lines, &mut doc_header.revision);
      }
    }

    Ok(())
  }
}

pub fn is_doc_header(lines: &ContiguousLines) -> bool {
  lines.any(|l| l.is_header(1) || l.starts_with_seq(&[Colon, Word, Colon]))
}
