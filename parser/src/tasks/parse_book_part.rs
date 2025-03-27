use crate::internal::*;

impl<'arena> Parser<'arena> {
  pub(crate) fn parse_book_part(&mut self) -> Result<Option<Sectioned<'arena>>> {
    // very dupy...
    let Some(mut lines) = self.read_lines()? else {
      return Ok(None);
    };

    let meta = self.parse_chunk_meta(&mut lines)?;

    let Some(line) = lines.current() else {
      if !meta.is_empty() {
        self.err_line_starting("Unattached block metadata", meta.start_loc)?;
      }
      self.restore_peeked_meta(meta);
      return Ok(None);
    };

    let Some(level) = self.line_heading_level(line) else {
      self.restore_peeked(lines, meta);
      return Ok(None);
    };
    // ...until here

    if level != 0 {
      self.restore_peeked(lines, meta);
      return Ok(None);
    }

    dbg!(level);

    // kinda dupy here...
    let mut heading_line = lines.consume_current().unwrap();
    let _equals = heading_line.consume_current().unwrap();
    heading_line.discard_assert(TokenKind::Whitespace);
    let heading = self.parse_inlines(&mut heading_line.into_lines())?;
    dbg!(heading);

    Ok(None)
  }
}
