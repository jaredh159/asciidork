use crate::internal::*;
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

    block.discard_leading_comment_lines();
    let mut doc_header = DocHeader {
      title: None,
      authors: bvec![in self.bump],
      revision: None,
      attrs: bvec![in self.bump],
    };

    self.parse_doc_title_author_revision(&mut block, &mut doc_header)?;
    self.parse_doc_attrs(&mut block)?;
    self.ctx.attrs = self.document.attrs.clone();
    self.setup_toc();
    Ok(Some(doc_header))
  }

  fn setup_toc(&mut self) {
    let Some(toc_attr) = self.document.attrs.get("toc") else {
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
    let title = self.document.attrs.str_or("toc-title", "Table of Contents");
    let title = BumpString::from_str_in(title, self.bump);
    let nodes = BumpVec::new_in(self.bump);
    self.document.toc = Some(TableOfContents { title, nodes, position })
  }

  fn parse_doc_title_author_revision(
    &mut self,
    lines: &mut ContiguousLines<'bmp, 'src>,
    doc_header: &mut DocHeader<'bmp>,
  ) -> Result<()> {
    let first_line = lines.current().expect("non-empty doc header");
    if !first_line.is_heading_level(0) {
      // author and revision must follow doc title, so if non title, skip
      return Ok(());
    }

    let mut header_line = lines.consume_current().unwrap();
    debug_assert!(header_line.starts_with_seq(&[EqualSigns, Whitespace]));
    header_line.discard(2);

    self.document.attrs.insert(
      "doctitle",
      AttrEntry {
        readonly: true,
        value: AttrValue::String(header_line.src.into()),
      },
    );

    doc_header.title = Some(DocTitle {
      heading: self.parse_inlines(&mut header_line.into_lines_in(self.bump))?,
      subtitle: None, // todo
    });

    if lines.starts(Word) {
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
  for line in lines.iter() {
    if line.is_heading_level(0) {
      return true;
    } else if line.is_comment() {
      continue;
    } else if !line.starts(Colon) {
      return false;
    } else {
      return line.starts_with_seq(&[Colon, Word, Colon])
        || line.starts_with_seq(&[Colon, MacroName])
        || line.starts_with_seq(&[Colon, Bang, Word, Colon]);
    }
  }
  false
}

#[cfg(test)]
mod tests {
  use super::*;
  use test_utils::{adoc, assert_eq};

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
    let bump = &Bump::new();
    for (input, expected) in cases {
      let lines = Parser::new(bump, input).read_lines().unwrap();
      assert_eq!(
        is_doc_header(&lines),
        expected,
        from: input
      );
    }
  }
}
