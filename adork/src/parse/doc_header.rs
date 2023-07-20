use std::collections::HashMap;

use super::{ast::*, Result};
use crate::parse::line_block::LineBlock;
use crate::parse::Parser;
use crate::token::TokenType::*;

impl Parser {
  pub(super) fn parse_doc_header(&self, mut block: LineBlock) -> Result<DocHeader> {
    block.remove_all(CommentLine);

    let mut doc_header = DocHeader {
      title: None,
      authors: vec![],
      revision: None,
      attrs: HashMap::new(),
    };

    self.parse_doc_title_author_revision(&mut block, &mut doc_header)?;
    // TODO: revision line https://docs.asciidoctor.org/asciidoc/latest/document/revision-line/
    self.parse_doc_attrs(&mut block, &mut doc_header.attrs)?;
    Ok(doc_header)
  }

  fn parse_doc_title_author_revision(
    &self,
    block: &mut LineBlock,
    doc_header: &mut DocHeader,
  ) -> Result<()> {
    let first_line = block.current_line().expect("non-empty doc header");
    if !first_line.is_header(1) {
      return Ok(());
    }

    let mut header_line = block.consume_current().unwrap();
    header_line.consume_expecting_seq(
      &[EqualSigns, Whitespace],
      "level-0 document header starting `= `",
    )?;

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
    } else if line.starts_with_seq(&[Colon, Word, Colon]) {
      return true;
    }
  }
  return false;
}

// tests

#[cfg(test)]
mod tests {
  use super::*;
  use crate::parse::inline::Inline;
  use crate::t::*;
  use indoc::indoc;
  use std::collections::HashMap;

  #[test]
  fn test_parse_example_doc_header() {
    let input = indoc! {"
      // this comment line is ignored
      = Document Title
      Kismet R. Lee <kismet@asciidoctor.org>
      :description: The document's description.
      :sectanchors:
      :url-repo: https://my-git-repo.com

      The document body starts here.
    "};

    let expected_header = DocHeader {
      title: Some(DocTitle {
        heading: vec![Inline::Text(s("Document Title"))],
        subtitle: None,
      }),
      authors: vec![Author {
        first_name: s("Kismet"),
        middle_name: Some(s("R.")),
        last_name: s("Lee"),
        email: Some(s("kismet@asciidoctor.org")),
      }],
      revision: None,
      attrs: HashMap::from([
        (s("description"), s("The document's description.")),
        (s("sectanchors"), s("")),
        (s("url-repo"), s("https://my-git-repo.com")),
      ]),
    };

    let document = Parser::parse_str(input).unwrap().document;
    assert_eq!(document.header, Some(expected_header));
  }
}
