use std::collections::HashMap;

use super::Result;
use crate::ast;
use crate::parse::Parser;
use crate::tok::{self, TokenType::*};

impl Parser {
  pub(super) fn parse_document_header(&mut self) -> Result<Option<ast::DocHeader>> {
    let Some(mut block) = self.read_block() else {
      return Ok(None)
    };

    if !is_doc_header(&block) {
      self.restore_block(block);
      return Ok(None);
    }

    block.remove_all(CommentLine);

    let mut doc_header = ast::DocHeader {
      title: None,
      authors: vec![],
      revision: None,
      attrs: HashMap::new(),
    };

    self.parse_doc_title_author_revision(&mut block, &mut doc_header)?;
    // TODO: revision line https://docs.asciidoctor.org/asciidoc/latest/document/revision-line/
    self.parse_doc_attrs(&mut block, &mut doc_header.attrs)?;
    Ok(Some(doc_header))
  }

  fn parse_doc_title_author_revision(
    &mut self,
    block: &mut tok::Block,
    doc_header: &mut ast::DocHeader,
  ) -> Result<()> {
    let first_line = block.current_line().expect("non-empty doc header");
    if !first_line.is_header(1) {
      return Ok(());
    }

    let mut header_line = block.consume_current().unwrap();
    let h0 = self.expect_group(
      [EqualSigns, Whitespace],
      "level-0 document header starting `'`",
      &mut header_line,
    )?;

    if h0.is_none() {
      return Ok(());
    }

    doc_header.title = Some(ast::DocTitle {
      heading: self.parse_inlines(header_line)?,
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

pub fn is_doc_header(block: &tok::Block) -> bool {
  for line in &block.lines {
    if line.is_header(1) || line.starts_with_seq(&[Colon, Word, Colon]) {
      return true;
    }
  }
  false
}

// tests

#[cfg(test)]
mod tests {
  use crate::ast;
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

    let expected_header = ast::DocHeader {
      title: Some(ast::DocTitle {
        heading: vec![ast::Inline::Text(s("Document Title"))],
        subtitle: None,
      }),
      authors: vec![ast::Author {
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
    let expected_body = vec![ast::Block {
      context: ast::BlockContext::Paragraph(vec![ast::Inline::Text(s(
        "The document body starts here.",
      ))]),
    }];

    let document = doc_test(input);
    assert_eq!(document.header, Some(expected_header));
    assert_eq!(document.content, ast::DocContent::Blocks(expected_body));
  }
}
