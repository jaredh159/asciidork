use crate::internal::*;

impl<'bmp, 'src> Parser<'bmp, 'src> {
  pub(crate) fn parse_section(&mut self) -> Result<Option<Section<'bmp>>> {
    let Some(mut lines) = self.read_lines() else {
      return Ok(None);
    };

    let meta = self.parse_chunk_meta(&mut lines)?;
    let Some(line) = lines.current() else {
      self.restore_peeked_meta(meta);
      return Ok(None);
    };

    let Some(level) = line.heading_level() else {
      self.restore_lines(lines);
      self.restore_peeked_meta(meta);
      return Ok(None);
    };

    if meta.attrs_has_str_positional("discrete") {
      self.restore_lines(lines);
      self.restore_peeked_meta(meta);
      return Ok(None);
    }

    let mut heading_line = lines.consume_current().unwrap();
    heading_line.discard_assert(TokenKind::EqualSigns);
    heading_line.discard_assert(TokenKind::Whitespace);
    let heading = self.parse_inlines(&mut heading_line.into_lines_in(self.bump))?;

    // blocks
    let mut blocks = BumpVec::new_in(self.bump);
    while let Some(inner) = self.parse_block()? {
      blocks.push(inner);
    }

    Ok(Some(Section { meta, level, heading, blocks }))
  }
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
    let section = parser.parse_section().unwrap().unwrap();
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

  #[test]
  fn test_parse_2_sections() {
    let input = "== one\n\nfoo\n\n== two\n\nbar";
    let b = &Bump::new();
    let mut parser = Parser::new(b, input);
    let section = parser.parse_section().unwrap().unwrap();
    assert_eq!(
      section,
      Section {
        meta: ChunkMeta::empty(0),
        level: 1,
        heading: b.inodes([n_text("one", 3, 6, b)]),
        blocks: b.vec([Block {
          context: BlockContext::Paragraph,
          content: BlockContent::Simple(b.inodes([n_text("foo", 8, 11, b),])),
          ..b.empty_block(8, 11)
        }])
      }
    );
    let section = parser.parse_section().unwrap().unwrap();
    assert_eq!(
      section,
      Section {
        meta: ChunkMeta::empty(13),
        level: 1,
        heading: b.inodes([n_text("two", 16, 19, b)]),
        blocks: b.vec([Block {
          context: BlockContext::Paragraph,
          content: BlockContent::Simple(b.inodes([n_text("bar", 21, 24, b),])),
          ..b.empty_block(21, 24)
        }])
      }
    );
  }
}
