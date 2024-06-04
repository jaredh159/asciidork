use super::{context::*, TableTokens};
use crate::internal::*;

impl<'bmp, 'src> Parser<'bmp, 'src> {
  pub(super) fn parse_dsv_implicit_first_row(
    &mut self,
    tokens: &mut TableTokens<'bmp, 'src>,
    ctx: &mut TableContext<'bmp, 'src>,
  ) -> Result<()> {
    let mut cells = bvec![in self.bump];
    while let Some(cell) = self.parse_dsv_table_cell(tokens, ctx, cells.len())? {
      cells.push(cell);
      if !ctx.counting_cols {
        break;
      }
    }
    ctx.num_cols = cells.len();
    self.finish_implicit_header_row(cells, ctx)
  }

  pub(super) fn parse_dsv_table_row(
    &mut self,
    tokens: &mut TableTokens<'bmp, 'src>,
    ctx: &mut TableContext<'bmp, 'src>,
  ) -> Result<Option<Row<'bmp>>> {
    let mut cells = bvec![in self.bump];
    while let Some(cell) = self.parse_dsv_table_cell(tokens, ctx, cells.len())? {
      cells.push(cell);
      if cells.len() == ctx.num_cols {
        break;
      }
    }
    if cells.is_empty() {
      Ok(None)
    } else {
      ctx.effective_row_idx += 1;
      Ok(Some(Row::new(cells)))
    }
  }

  fn parse_dsv_table_cell(
    &mut self,
    tokens: &mut TableTokens<'bmp, 'src>,
    ctx: &mut TableContext<'bmp, 'src>,
    col_index: usize,
  ) -> Result<Option<Cell<'bmp>>> {
    if tokens.is_empty() {
      return Ok(None);
    }
    let mut start = tokens.current().unwrap().loc.start;
    let mut cell_tokens = bvec![in self.bump];

    // trim leading whitespace
    while tokens.current().is_whitespaceish() {
      let trimmed = tokens.consume_current().unwrap();
      start = trimmed.loc.end;
    }

    let mut end = start;
    let sep = ctx.format.separator();
    let embeddable_sep = match sep {
      ':' | ';' | '|' | ',' => None,
      _ => Some(sep),
    };

    loop {
      if tokens.current().is_none() || self.consume_dsv_delimiter(tokens, ctx, sep) {
        return self
          .finish_cell(CellSpec::default(), cell_tokens, col_index, ctx, start..end)
          .map(|data| data.map(|(cell, _)| cell));
      }
      let token = tokens.consume_splitting(embeddable_sep).unwrap();
      if token.is(TokenKind::Newline) {
        // see note in Parser::parse_psv_table_cell
        ctx.counting_cols = false
      }
      end = token.loc.end;
      if !token.is(TokenKind::Backslash) {
        cell_tokens.push(token);
      } else if let Some(next) = tokens.consume_current() {
        end = next.loc.end;
        cell_tokens.push(next);
      }
    }
  }

  fn consume_dsv_delimiter(
    &self,
    tokens: &mut TableTokens,
    ctx: &mut TableContext,
    delim: char,
  ) -> bool {
    if tokens.current().is(TokenKind::Newline) {
      ctx.counting_cols = false;
      tokens.consume_current();
      return true;
    }

    let sep_len = delim.len_utf8();
    let token = tokens.current_mut().unwrap();
    if token.lexeme.starts_with(delim) {
      if token.lexeme.len() == sep_len {
        tokens.consume_current();
        true
      } else {
        token.loc.start += sep_len;
        token.lexeme = &token.lexeme[sep_len..];
        true
      }
    } else {
      false
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use test_utils::{assert_eq, *};
  use ColWidth::*;

  const fn w(width: u8) -> ColWidth {
    Proportional(width)
  }

  #[test]
  fn basic_dsv_table() {
    assert_table!(
      adoc! {r#"
        [format="dsv"]
        |===
        one:two
        |===
      "#},
      Table {
        col_widths: ColWidths::new(vecb![w(1), w(1)]),
        rows: vecb![Row::new(vecb![
          cell!(d: "one", 20..23),
          cell!(d: "two", 24..27),
        ])],
        ..empty_table!()
      }
    );
  }

  #[test]
  fn dsv_escaped_separator() {
    assert_table!(
      adoc! {r#"
        [format="dsv"]
        |===
        one \: and :two
        |===
      "#},
      Table {
        col_widths: ColWidths::new(vecb![w(1), w(1)]),
        rows: vecb![Row::new(vecb![
          cell!(d: "one : and", 20..30),
          cell!(d: "two", 32..35),
        ])],
        ..empty_table!()
      }
    );
  }

  #[test]
  fn dsv_table_trailing_sep_is_empty_cell() {
    assert_table!(
      adoc! {r#"
        [format="dsv"]
        |===
        one:two:
        |===
      "#},
      Table {
        col_widths: ColWidths::new(vecb![w(1), w(1), w(1)]),
        rows: vecb![Row::new(vecb![
          cell!(d: "one", 20..23),
          cell!(d: "two", 24..27),
          empty_cell!(),
        ])],
        ..empty_table!()
      }
    );
  }

  #[test]
  fn dsv_weird_multibyte_separator() {
    assert_table!(
      adoc! {r#"
        [format="dsv",separator=¦]
        |===
        one¦two
        |===
      "#},
      Table {
        col_widths: ColWidths::new(vecb![w(1), w(1)]),
        rows: vecb![Row::new(vecb![
          cell!(d: "one", 33..36),
          cell!(d: "two", 38..41),
        ])],
        ..empty_table!()
      }
    );
  }

  #[test]
  fn dsv_shorthand() {
    assert_table!(
      adoc! {r#"
        :===
        one:two
        :===
      "#},
      Table {
        col_widths: ColWidths::new(vecb![w(1), w(1)]),
        rows: vecb![Row::new(vecb![
          cell!(d: "one", 5..8),
          cell!(d: "two", 9..12),
        ])],
        ..empty_table!()
      }
    );
  }

  #[test]
  fn dsv_single_cell() {
    assert_table!(
      adoc! {r#"
        :===
        one
        :===
      "#},
      Table {
        col_widths: ColWidths::new(vecb![w(1)]),
        rows: vecb![Row::new(vecb![cell!(d: "one", 5..8)])],
        ..empty_table!()
      }
    );
  }

  #[test]
  fn dsv_empty_lines_skipped() {
    assert_table!(
      adoc! {r#"
        :===
        one
        two

        three:four
        :===
      "#},
      Table {
        col_widths: ColWidths::new(vecb![w(1)]),
        rows: vecb![
          Row::new(vecb![cell!(d: "one", 5..8)]),
          Row::new(vecb![cell!(d: "two", 9..12)]),
          Row::new(vecb![cell!(d: "three", 14..19)]),
          Row::new(vecb![cell!(d: "four", 20..24)]),
        ],
        ..empty_table!()
      }
    );
  }

  #[test]
  fn dsv_table_ragged_rows() {
    assert_table!(
      adoc! {r#"
        [format="dsv",separator=;,cols="1,1"]
        |===
        1;2;3
        4
        |===
      "#},
      Table {
        col_widths: ColWidths::new(vecb![w(1), w(1)]),
        rows: vecb![
          Row::new(vecb![cell!(d: "1", 43..44), cell!(d: "2", 45..46),]),
          Row::new(vecb![cell!(d: "3", 47..48), cell!(d: "4", 49..50),])
        ],
        ..empty_table!()
      }
    );
  }
}
