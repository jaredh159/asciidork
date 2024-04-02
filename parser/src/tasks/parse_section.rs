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
      self.restore_peeked(lines, meta);
      return Ok(None);
    };

    if meta.attrs_has_str_positional("discrete") {
      self.restore_peeked(lines, meta);
      return Ok(None);
    }

    let last_level = self.ctx.section_level;
    self.ctx.section_level = level;
    let mut heading_line = lines.consume_current().unwrap();
    heading_line.discard_assert(TokenKind::EqualSigns);
    heading_line.discard_assert(TokenKind::Whitespace);
    let heading = self.parse_inlines(&mut heading_line.into_lines_in(self.bump))?;

    // blocks
    self.restore_lines(lines);
    let mut blocks = BumpVec::new_in(self.bump);
    while let Some(inner) = self.parse_block()? {
      blocks.push(inner);
    }

    self.ctx.section_level = last_level;
    Ok(Some(Section { meta, level, heading, blocks }))
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use test_utils::{assert_eq, *};

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
  fn test_parse_nested_section() {
    let input = "== one\n\n=== two\n\nbar";
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
          meta: ChunkMeta::empty(8),
          context: BlockContext::Section,
          content: BlockContent::Section(Section {
            meta: ChunkMeta::empty(8),
            level: 2,
            heading: b.inodes([n_text("two", 12, 15, b)]),
            blocks: b.vec([Block {
              context: BlockContext::Paragraph,
              content: BlockContent::Simple(b.inodes([n_text("bar", 17, 20, b),])),
              ..b.empty_block(17, 20)
            }])
          }),
          ..b.empty_block(8, 20)
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
