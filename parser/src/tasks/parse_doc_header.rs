use crate::internal::*;
use crate::variants::token::*;

impl<'arena> Parser<'arena> {
  pub(crate) fn parse_document_header(&mut self) -> Result<()> {
    let mut header = DocHeader::default();
    let Some(mut block) = self.parse_prefixed_exception_blocks(&mut header)? else {
      self.finalize_doc_header(header);
      return Ok(());
    };

    if !self.is_doc_header(&block) {
      self.peeked_lines = Some(block);
      self.finalize_doc_header(header);
      return Ok(());
    }

    self.parse_header_doc_attrs(&mut block, &mut header)?;
    self.parse_doc_title_author_revision(&mut block, &mut header)?;
    self.parse_header_doc_attrs(&mut block, &mut header)?;
    self.finalize_doc_header(header);
    Ok(())
  }

  fn parse_prefixed_exception_blocks(
    &mut self,
    header: &mut DocHeader,
  ) -> Result<Option<ContiguousLines<'arena>>> {
    let Some(mut lines) = self.read_lines()? else {
      return Ok(None);
    };

    if let Some(discarded) = lines.discard_leading_comment_lines() {
      header.loc.extend(discarded);
    }

    if lines.is_empty() {
      return self.parse_prefixed_exception_blocks(header);
    }

    if lines.current_satisfies(|l| l.is_attr_decl()) {
      self.parse_header_doc_attrs(&mut lines, header)?;
      self.restore_lines(lines);
      return self.parse_prefixed_exception_blocks(header);
    }

    if let Some(end) = self.discard_comment_block(&mut lines)? {
      self.restore_lines(lines);
      header.loc.extend(end);
      return self.parse_prefixed_exception_blocks(header);
    }

    if lines.is_empty() {
      return self.parse_prefixed_exception_blocks(header);
    }

    Ok(Some(lines))
  }

  fn discard_comment_block(
    &mut self,
    lines: &mut ContiguousLines<'arena>,
  ) -> Result<Option<SourceLocation>> {
    if !lines.current_satisfies(Line::is_comment_block_delimiter) {
      return Ok(None);
    }

    let block_start = lines.consume_current().unwrap();
    if lines.discard_until(Line::is_comment_block_delimiter) {
      let end_delim = lines.consume_current().unwrap();
      return Ok(end_delim.first_loc());
    }

    loop {
      let Some(mut next_lines) = self.read_lines()? else {
        self.err_line("Unclosed comment block, started here", &block_start)?;
        return Ok(None);
      };
      if next_lines.discard_until(Line::is_comment_block_delimiter) {
        let end_delim = next_lines.consume_current().unwrap();
        return Ok(end_delim.first_loc());
      }
    }
  }

  pub(crate) fn prepare_toc(&mut self) {
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

  fn parse_doc_title_author_revision(
    &mut self,
    lines: &mut ContiguousLines<'arena>,
    header: &mut DocHeader<'arena>,
  ) -> Result<()> {
    if lines.is_empty() {
      return Ok(());
    }
    let meta = self.parse_chunk_meta(lines)?;
    if lines
      .current()
      .is_none_or(|first| self.line_heading_level(first) != Some(0))
    {
      // author and revision must follow doc title, so if non title, skip
      self.restore_peeked_meta(meta);
      return Ok(());
    }

    let mut header_line = lines.consume_current().unwrap();
    debug_assert!(header_line.starts_with_seq(&[Kind(EqualSigns), Kind(Whitespace)]));
    header_line.discard(2); // equals, whitespace
    self
      .document
      .meta
      .insert_header_attr(
        "_asciidork_derived_doctitle",
        header_line.reassemble_src().as_str(),
      )
      .unwrap();

    header.loc.extend(header_line.last_loc().unwrap());
    header.title = Some(DocTitle {
      attrs: meta.attrs,
      main: self.parse_inlines(&mut header_line.into_lines())?,
      subtitle: None, // TODO: subtitle
    });

    if lines.starts(Word) {
      let author_line = lines.consume_current().unwrap();
      header.loc.extend(author_line.last_loc().unwrap());
      self.parse_author_line(author_line)?;
      // revision line can only follow an author line (and requires a doc header)
      if self.document.meta.is_set("author") {
        if let Some(end) = self.parse_revision_line(lines) {
          header.loc.end = end;
        }
      }
    }

    Ok(())
  }

  fn is_doc_header(&self, lines: &ContiguousLines) -> bool {
    for line in lines.iter() {
      if self.line_heading_level(line) == Some(0) {
        return true;
      } else if line.is_comment() || line.is_block_attr_list() || line.is_block_anchor() {
        continue;
      } else {
        return line.is_attr_decl();
      }
    }
    false
  }

  fn finalize_doc_header(&mut self, header: DocHeader<'arena>) {
    if header.title.is_some() || !header.loc.is_empty() {
      self.document.header = Some(header);
    }
  }

  fn parse_header_doc_attrs(
    &mut self,
    lines: &mut ContiguousLines<'arena>,
    header: &mut DocHeader,
  ) -> Result<()> {
    if let Some(loc) = self.parse_doc_attrs(lines)? {
      header.loc.end = loc.end;
    }
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use test_utils::*;

  #[test]
  fn header_existence_and_loc() {
    let cases: Vec<(&str, Option<SourceLocation>)> = vec![
      ("foobar\n\n", None),
      ("= Title\n\n", Some(loc!(0..7))),
      ("// rofl\n\n", Some(loc!(0..7))),
      (
        adoc! {"
          = Title
          :a: b
        "},
        Some(loc!(0..13)),
      ),
      (
        adoc! {"
          ////
          block comment
          ////

          foobar
        "},
        Some(loc!(0..23)),
      ),
      (
        adoc! {"
          = Title
          Bob Law
        "},
        Some(loc!(0..15)),
      ),
      (
        adoc! {"
          = Document Title
        "},
        Some(loc!(0..16)),
      ),
      (
        adoc! {"
          = Title
          Bob Law
          v1.2334
        "},
        Some(loc!(0..23)),
      ),
      (
        adoc! {"
          ////
          block comment
          ////

          // line comment

          :a: b

          = Title
          :c: d

          foobar
        "},
        Some(loc!(0..62)),
      ),
    ];
    for (input, expected) in cases {
      let mut parser = test_parser!(input);
      parser.parse_document_header().unwrap();
      expect_eq!(
        parser.document.header.as_ref().map(|h| h.loc),
        expected,
        from: input
      );
    }
  }

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
          [[foo-bar]]
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
