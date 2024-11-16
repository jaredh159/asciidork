use crate::internal::*;
use crate::variants::token::*;

impl<'arena> Parser<'arena> {
  pub(crate) fn parse_document_header(&mut self) -> Result<()> {
    let Some(mut block) = self.read_lines()? else {
      return Ok(());
    };

    if !self.is_doc_header(&block) {
      self.peeked_lines = Some(block);
      return Ok(());
    }

    self.parse_doc_attrs(&mut block)?;
    self.parse_doc_title_author_revision(&mut block)?;
    self.parse_doc_attrs(&mut block)?;
    self.setup_toc();
    Ok(())
  }

  fn setup_toc(&mut self) {
    let Some(toc_attr) = self.document.meta.get("toc") else {
      return;
    };

    let position = match toc_attr {
      AttrValue::Bool(false) => return,
      AttrValue::Bool(true) => TocPosition::Auto,
      AttrValue::String(s) => match s.as_str() {
        "left" => TocPosition::Left,
        "right" => TocPosition::Right,
        "preamble" => TocPosition::Preamble,
        "macro" => TocPosition::Macro,
        "auto" => TocPosition::Auto,
        _ => return, // err?
      },
    };
    let title = self.string(self.document.meta.str_or("toc-title", "Table of Contents"));
    let nodes = BumpVec::new_in(self.bump);
    self.document.toc = Some(TableOfContents { title, nodes, position })
  }

  fn parse_doc_title_author_revision(&mut self, lines: &mut ContiguousLines<'arena>) -> Result<()> {
    if !lines
      .current()
      .map_or(false, |first| self.line_heading_level(first) == Some(0))
    {
      // author and revision must follow doc title, so if non title, skip
      return Ok(());
    }

    let mut header_line = lines.consume_current().unwrap();
    debug_assert!(header_line.starts_with_seq(&[Kind(EqualSigns), Kind(Whitespace)]));
    header_line.discard(2); // equals, whitespace
    self
      .document
      .meta
      .insert_header_attr("doctitle", header_line.reassemble_src().as_str())
      .unwrap();

    self.document.title = Some(self.parse_inlines(&mut header_line.into_lines())?);
    // TODO: subtitle

    if lines.starts(Word) {
      self.parse_author_line(lines.consume_current().unwrap())?;
      // revision line can only follow an author line (and requires a doc header)
      if self.document.meta.is_set("author") {
        self.parse_revision_line(lines);
      }
    }

    Ok(())
  }

  fn is_doc_header(&self, lines: &ContiguousLines) -> bool {
    for line in lines.iter() {
      if self.line_heading_level(line) == Some(0) {
        return true;
      } else if line.is_comment() {
        continue;
      } else {
        return line.is_attr_decl();
      }
    }
    false
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use test_utils::*;

  #[test]
  fn test_is_doc_header() {
    let cases = vec![
      (
        adoc! {"
          // ignored
          = Title
          :foo: bar
        "},
        true,
      ),
      (
        adoc! {"
          = Title
          :foo: bar
        "},
        true,
      ),
      (":foo: bar\n", true),
      (":!foo:\n", true),
      (":!foo-bar:\n", true),
      (
        adoc! {"
          ----
          = Title
          ----
        "},
        false,
      ),
    ];
    for (input, expected) in cases {
      let mut parser = test_parser!(input);
      let lines = parser.read_lines().unwrap().unwrap();
      expect_eq!(
        parser.is_doc_header(&lines),
        expected,
        from: input
      );
    }
  }
}
