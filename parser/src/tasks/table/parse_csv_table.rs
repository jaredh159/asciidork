use super::{context::*, TableTokens};
use crate::internal::*;
use crate::variants::token::*;

impl<'arena> Parser<'arena> {
  pub(crate) fn finish_csv_table_cell(
    &mut self,
    tokens: &mut TableTokens<'arena>,
    ctx: &mut TableContext<'arena>,
    col_index: usize,
    mut start: usize,
  ) -> Result<Option<Cell<'arena>>> {
    let mut cell_tokens = Deq::new(self.bump);
    let mut end = start;

    let mut quote = {
      if tokens.current().is(TokenKind::DoubleQuote) {
        let quote = tokens.consume_current().unwrap();
        start = quote.loc.end;
        while tokens.current().is_whitespaceish() {
          let trimmed = tokens.consume_current().unwrap();
          start = trimmed.loc.end;
        }
        Some(quote.loc.start)
      } else {
        None
      }
    };

    loop {
      if tokens.current().is_none() || (quote.is_none() && self.consume_dsv_delimiter(tokens, ctx))
      {
        if let Some(start) = quote {
          self.err_at("Unclosed CSV quote", start, start + 1)?;
        }
        return self
          .finish_cell(CellSpec::default(), cell_tokens, col_index, ctx, start..end)
          .map(|data| data.map(|(cell, _)| cell));
      }

      let token = tokens
        .consume_splitting(ctx.embeddable_cell_separator)
        .unwrap();

      if token.is(Newline) {
        // see note in Parser::parse_psv_table_cell
        ctx.counting_cols = false
      }

      end = token.loc.end;
      match token.kind {
        DoubleQuote if quote.is_none() => {
          self.err_at_loc(
            "Double quote not allowed here, entire field must be quoted",
            token.loc,
          )?;
        }
        DoubleQuote if quote.is_some() && tokens.current().is_not(DoubleQuote) => {
          quote = None;
        }
        DoubleQuote if quote.is_some() && tokens.current().is(DoubleQuote) => {
          cell_tokens.push(tokens.consume_current().unwrap());
        }
        _ => {
          cell_tokens.push(token);
        }
      }
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
  fn basic_csv_table() {
    assert_table!(
      adoc! {r#"
        [format="csv"]
        |===
        one,two
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
  fn csv_quoted() {
    assert_table!(
      adoc! {r#"
        ,===
        one,"two,three"
        ,===
      "#},
      Table {
        col_widths: ColWidths::new(vecb![w(1), w(1)]),
        rows: vecb![Row::new(vecb![
          cell!(d: "one", 5..8),
          cell!(d: "two,three", 10..19),
        ])],
        ..empty_table!()
      }
    );
  }

  #[test]
  fn csv_quoted_w_newline() {
    assert_table!(
      adoc! {r#"
        ,===
        one,"two
        three"
        ,===
      "#},
      Table {
        col_widths: ColWidths::new(vecb![w(1), w(1)]),
        rows: vecb![Row::new(vecb![
          cell!(d: "one", 5..8),
          Cell {
            content: CellContent::Default(vecb![nodes![
              node!("two"; 10..13),
              node!(Inline::Newline, 13..14),
              node!("three"; 14..19),
            ]]),
            ..empty_cell!()
          }
        ])],
        ..empty_table!()
      }
    );
  }

  #[test]
  fn csv_empty_trailing_cell() {
    assert_table!(
      adoc! {r#"
        ,===
        A1,
        B1,B2
        ,===
      "#},
      Table {
        col_widths: ColWidths::new(vecb![w(1), w(1)]),
        rows: vecb![
          Row::new(vecb![cell!(d: "A1", 5..7), empty_cell!()]),
          Row::new(vecb![cell!(d: "B1", 9..11), cell!(d: "B2", 12..14)]),
        ],
        ..empty_table!()
      }
    );
  }

  #[test]
  fn csv_single_cell() {
    assert_table!(
      adoc! {r#"
        ,===
        single cell
        ,===
      "#},
      Table {
        col_widths: ColWidths::new(vecb![w(1)]),
        rows: vecb![Row::new(vecb![cell!(d: "single cell", 5..16)]),],
        ..empty_table!()
      }
    );
  }

  #[test]
  fn csv_unclosed_quote() {
    assert_table_loose!(
      adoc! {r#"
        ,===
        one,"two
        ,===
      "#},
      Table {
        col_widths: ColWidths::new(vecb![w(1), w(1)]),
        rows: vecb![Row::new(vecb![
          cell!(d: "one", 5..8),
          cell!(d: "two", 10..13)
        ]),],
        ..empty_table!()
      }
    );
  }

  #[test]
  fn csv_quote_own_line() {
    assert_table!(
      adoc! {r#"
        [cols=2*]
        ,===
        "
        A
        ","
        B
        "
        ,===
      "#},
      Table {
        col_widths: ColWidths::new(vecb![w(1), w(1)]),
        rows: vecb![Row::new(vecb![
          cell!(d: "A", 17..18),
          cell!(d: "B", 23..24)
        ]),],
        ..empty_table!()
      }
    );
  }

  #[test]
  fn recognizes_implicit_header_inside_asciidoc_cell() {
    let adoc = adoc! {r#"
      [cols="1,1a"]
      ,===
      a,b

      c,d
      ,===
    "#};
    let table = parse_table!(adoc);
    assert_eq!(
      table.header_row,
      Some(Row::new(vecb![cell!(d: "a", 19..20), cell!(d: "b", 21..22)])),
      from: adoc
    );
  }

  assert_error!(
    csv_unterminated_quote,
    adoc! {r#"
      ,===
      one,"two
      ,===
    "#},
    error! { r#"
      2: one,"two
             ^ Unclosed CSV quote
    "#}
  );

  assert_error!(
    csv_bare_quote,
    adoc! {r#"
      ,===
      one,two "foo
      ,===
    "#},
    error! { r#"
      2: one,two "foo
                 ^ Double quote not allowed here, entire field must be quoted
    "#}
  );
}
