use std::collections::HashMap;
use std::io::BufRead;

use super::{ast::*, Result};
use crate::parse::line_block::LineBlock;
use crate::parse::Parser;
use crate::token::TokenType::*;

impl<R: BufRead> Parser<R> {
  pub(super) fn parse_doc_header(&self, mut block: LineBlock) -> Result<DocHeader> {
    block.remove_all(CommentLine);

    let mut doc_header = DocHeader {
      title: None,
      authors: vec![],
      revision: None,
      attrs: HashMap::new(),
    };

    self.parse_doc_title_author_revision(block, &mut doc_header)?;
    Ok(doc_header)
  }

  fn parse_doc_title_author_revision(
    &self,
    mut block: LineBlock,
    doc_header: &mut DocHeader,
  ) -> Result<()> {
    let first_line = block.current_line().expect("non-empty doc header");
    if !first_line.is_header(1) {
      return Ok(());
    }

    let mut header_line = block.consume_current().unwrap();
    header_line.consume_expecting(EqualSigns)?;
    header_line.consume_expecting(Whitespace)?;

    doc_header.title = Some(DocTitle {
      heading: self.parse_inlines(header_line),
      subtitle: None, // todo
    });

    if block.current_line_starts_with(Word) {
      self.parse_author_line(block.consume_current().unwrap(), &mut doc_header.authors)?;
    }

    Ok(())
  }
}

pub fn is_doc_header(block: &LineBlock) -> bool {
  for line in &block.lines {
    if line.is_header(1) {
      return true;
    } else if line.starts_with(&[Colon, Word, Colon]) {
      return true;
    }
  }
  return false;
}
