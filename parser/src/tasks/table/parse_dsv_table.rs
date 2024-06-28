use super::{context::*, DataFormat, TableTokens};
use crate::internal::*;

impl<'arena> Parser<'arena> {
  pub(super) fn parse_dsv_implicit_first_row(
    &mut self,
    tokens: &mut TableTokens<'arena>,
    ctx: &mut TableContext<'arena>,
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
    tokens: &mut TableTokens<'arena>,
    ctx: &mut TableContext<'arena>,
  ) -> Result<Option<Row<'arena>>> {
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
    tokens: &mut TableTokens<'arena>,
    ctx: &mut TableContext<'arena>,
    col_index: usize,
  ) -> Result<Option<Cell<'arena>>> {
    if tokens.is_empty() {
      return Ok(None);
    }
    let mut start = tokens.current().unwrap().loc.start;

    let mut trimmed_newline = false;
    while tokens.current().is_whitespaceish() {
      let trimmed = tokens.consume_current().unwrap();
      start = trimmed.loc.end;
      if trimmed.is(TokenKind::Newline) {
        trimmed_newline = true;
        ctx.counting_cols = false;
      }
    }

    // delimiter followed by newline is an empty cell
    if ctx.dsv_last_consumed == DsvLastConsumed::Delimiter && trimmed_newline {
      let spec = CellSpec::default();
      return self
        .finish_cell(spec, Deq::new(self.bump), col_index, ctx, start..start)
        .map(|data| data.map(|(cell, _)| cell));
    }

    let maybe_cell = match ctx.format {
      DataFormat::Csv(_) => self.finish_csv_table_cell(tokens, ctx, col_index, start)?,
      DataFormat::Delimited(_) => self.finish_dsv_table_cell(tokens, ctx, col_index, start)?,
      _ => unreachable!(),
    };

    if ctx.header_row.is_unknown()
      && maybe_cell.is_some()
      && ctx.dsv_last_consumed == DsvLastConsumed::Newline
      && tokens.current().is(TokenKind::Newline)
    {
      ctx.header_row = HeaderRow::FoundImplicit;
    }

    Ok(maybe_cell)
  }

  fn finish_dsv_table_cell(
    &mut self,
    tokens: &mut TableTokens<'arena>,
    ctx: &mut TableContext<'arena>,
    col_index: usize,
    start: usize,
  ) -> Result<Option<Cell<'arena>>> {
    let mut cell_tokens = Deq::new(self.bump);
    let mut end = start;

    loop {
      if tokens.current().is_none() || self.consume_dsv_delimiter(tokens, ctx) {
        return self
          .finish_cell(CellSpec::default(), cell_tokens, col_index, ctx, start..end)
          .map(|data| data.map(|(cell, _)| cell));
      }
      let token = tokens
        .consume_splitting(ctx.embeddable_cell_separator)
        .unwrap();
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

  pub(crate) fn consume_dsv_delimiter(
    &self,
    tokens: &mut TableTokens,
    ctx: &mut TableContext,
  ) -> bool {
    if tokens.current().is(TokenKind::Newline) {
      ctx.dsv_last_consumed = DsvLastConsumed::Newline;
      ctx.counting_cols = false;
      tokens.consume_current();
      return true;
    }

    ctx.dsv_last_consumed = DsvLastConsumed::Other;
    if let Some(tokenkind) = ctx.cell_separator_tokenkind {
      return if tokens.current().is(tokenkind) {
        ctx.dsv_last_consumed = DsvLastConsumed::Delimiter;
        tokens.consume_current();
        true
      } else {
        false
      };
    }

    let sep_len = ctx.cell_separator.len_utf8();
    let token = tokens.current_mut().unwrap();
    if token.lexeme.starts_with(ctx.cell_separator) {
      ctx.dsv_last_consumed = DsvLastConsumed::Delimiter;
      if token.lexeme.len() == sep_len {
        tokens.consume_current();
        true
      } else {
        tokens.drop_leading_bytes(sep_len);
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
        one:
        two:
        |===
      "#},
      Table {
        col_widths: ColWidths::new(vecb![w(1), w(1)]),
        rows: vecb![
          Row::new(vecb![cell!(d: "one", 20..23), empty_cell!(),]),
          Row::new(vecb![cell!(d: "two", 25..28), empty_cell!(),])
        ],
        ..empty_table!()
      }
    );
  }

  #[test]
  fn dsv_trailing_space_empty_cells() {
    assert_table!(
      //         v-- trailing space
      ":===\none: \ntwo:\n:===",
      Table {
        col_widths: ColWidths::new(vecb![w(1), w(1)]),
        rows: vecb![
          Row::new(vecb![cell!(d: "one", 5..8), empty_cell!(),]),
          Row::new(vecb![cell!(d: "two", 11..14), empty_cell!(),])
        ],
        ..empty_table!()
      }
    );
    assert_table!(
      //             v-- trailing space
      ":===\none:two: \n:===",
      Table {
        col_widths: ColWidths::new(vecb![w(1), w(1), w(1)]),
        rows: vecb![Row::new(vecb![
          cell!(d: "one", 5..8),
          cell!(d: "two", 9..12),
          empty_cell!(),
        ]),],
        ..empty_table!()
      }
    );
  }

  #[test]
  fn dsv_multibyte_char_separator() {
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

  #[test]
  fn dsv_empty_non_eol_cell() {
    assert_table!(
      adoc! {r#"
        :===
        a: :c
        :===
      "#},
      Table {
        col_widths: ColWidths::new(vecb![w(1), w(1), w(1)]),
        rows: vecb![Row::new(vecb![
          cell!(d: "a", 5..6),
          empty_cell!(),
          cell!(d: "c", 9..10),
        ]),],
        ..empty_table!()
      }
    );
  }
}
