use crate::internal::*;

impl<'bmp, 'src> Parser<'bmp, 'src> {
  pub(crate) fn parse_section(&mut self) -> Result<Option<SectionOutput<'bmp>>> {
    let Some(mut lines) = self.read_lines() else {
      return Ok(None);
    };

    let meta = self.parse_block_metadata(&mut lines)?;
    let Some(line) = lines.current() else {
      return Ok(Some(SectionOutput::PeekedMeta(meta)));
    };

    let Some(level) = line.header_level() else {
      self.restore_lines(lines);
      return Ok(Some(SectionOutput::PeekedMeta(meta)));
    };

    if meta
      .attrs
      .as_ref()
      .map_or(false, |attrs| attrs.has_str_positional("discrete"))
    {
      self.restore_lines(lines);
      return Ok(Some(SectionOutput::PeekedMeta(meta)));
    }

    let mut heading_line = lines.consume_current().unwrap();
    heading_line.discard_assert(TokenKind::EqualSigns);
    heading_line.discard_assert(TokenKind::Whitespace);
    let heading = self.parse_inlines(&mut heading_line.into_lines_in(self.bump))?;

    // blocks
    let mut blocks = BumpVec::new_in(self.bump);
    while let Some(inner) = self.parse_block(None)? {
      blocks.push(inner);
    }

    Ok(Some(SectionOutput::Section(Section {
      meta,
      level,
      heading,
      blocks,
    })))
  }
}

#[derive(Debug)]
pub enum SectionOutput<'bmp> {
  Section(Section<'bmp>),
  PeekedMeta(ChunkMeta<'bmp>),
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::test::*;
  use test_utils::assert_eq;

  #[test]
  fn test_parse_section() {
    let input = "== foo\n\nbar";
    let b = &Bump::new();
    let mut parser = Parser::new(b, input);
    let out = parser.parse_section().unwrap().unwrap();
    let section = match out {
      SectionOutput::Section(section) => section,
      _ => panic!("expected section"),
    };
    assert_eq!(
      section,
      Section {
        meta: ChunkMeta::empty(0),
        level: 1,
        heading: b.inodes([n_text("foo", 3, 6, b)]),
        blocks: b.vec([Block {
          context: BlockContext::Paragraph,
          content: BlockContent::Simple(b.inodes([n_text("bar", 8, 11, b),])),
          ..b.empty_block(8, 11)
        }])
      }
    );
  }
}
