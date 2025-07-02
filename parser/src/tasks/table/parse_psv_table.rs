use super::{context::*, TableTokens};
use crate::internal::*;

impl<'arena> Parser<'arena> {
  pub(super) fn parse_psv_implicit_first_row(
    &mut self,
    tokens: &mut TableTokens<'arena>,
    ctx: &mut TableContext<'arena>,
  ) -> Result<()> {
    let mut cells = bvec![in self.bump];
    while let Some((cell, dupe)) = self.parse_psv_table_cell(tokens, ctx, cells.len())? {
      if dupe > 1 {
        for _ in 1..dupe {
          ctx.add_phantom_cells(&cell, cells.len());
          cells.push(cell.clone());
        }
      }
      ctx.add_phantom_cells(&cell, cells.len());
      cells.push(cell);
      if !ctx.counting_cols {
        break;
      }
    }
    while tokens.current().kind(TokenKind::Newline) {
      tokens.consume_current();
    }

    ctx.num_cols = cells.iter().map(|c| c.col_span as usize).sum();
    self.finish_implicit_header_row(cells, ctx)
  }

  pub(super) fn parse_psv_table_row(
    &mut self,
    tokens: &mut TableTokens<'arena>,
    ctx: &mut TableContext<'arena>,
  ) -> Result<Option<Row<'arena>>> {
    let mut cells = bvec![in self.bump];
    std::mem::swap(&mut cells, &mut ctx.spilled_cells);
    let mut num_effective_cells = ctx.row_phantom_cells();
    while let Some((cell, dupe)) = self.parse_psv_table_cell(tokens, ctx, cells.len())? {
      if dupe > 1 {
        for _ in 1..dupe {
          ctx.add_phantom_cells(&cell, num_effective_cells);
          num_effective_cells += cell.col_span as usize;
          if num_effective_cells < ctx.num_cols {
            cells.push(cell.clone());
          } else {
            ctx.spilled_cells.push(cell.clone());
          }
        }
      }
      ctx.add_phantom_cells(&cell, num_effective_cells);
      num_effective_cells += cell.col_span as usize;
      cells.push(cell);
      if num_effective_cells >= ctx.num_cols {
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

  fn parse_psv_table_cell(
    &mut self,
    tokens: &mut TableTokens<'arena>,
    ctx: &mut TableContext<'arena>,
    col_index: usize,
  ) -> Result<Option<(Cell<'arena>, u8)>> {
    if tokens.is_empty() {
      return Ok(None);
    }

    let spec_loc = tokens.current().unwrap().loc;
    let (spec, mut start) = match self.consume_cell_start(tokens, ctx.format.separator()) {
      Some((spec, start)) => (spec, start),
      None => {
        let sep = ctx.format.separator();
        self.err_token(format!("Expected cell separator `{sep}`"), tokens.nth(0))?;
        (CellSpec::default(), tokens.current().unwrap().loc.start)
      }
    };

    let mut drop_invalid_cell = false;
    if !ctx.counting_cols && spec.col_span.unwrap_or(0) > ctx.num_cols as u8 {
      self.err_at(
        format!(
          "Cell column span ({}) exceeds number of columns ({})",
          spec.col_span.unwrap(),
          ctx.num_cols
        ),
        spec_loc.setting_end(start - 1),
      )?;
      drop_invalid_cell = true;
    }

    let mut end = start;
    let mut cell_tokens = Line::empty(self.bump);

    // trim leading whitespace
    while tokens.current().kind(TokenKind::Whitespace) {
      let trimmed = tokens.consume_current().unwrap();
      start = trimmed.loc.end;
    }

    loop {
      if self.starts_psv_cell(tokens, ctx.cell_separator) {
        if drop_invalid_cell {
          return self.parse_psv_table_cell(tokens, ctx, col_index);
        } else {
          return self.finish_cell(spec, cell_tokens, col_index, ctx, start..end);
        }
      }
      let Some(token) = tokens.consume_splitting(ctx.embeddable_cell_separator) else {
        if drop_invalid_cell {
          return self.parse_psv_table_cell(tokens, ctx, col_index);
        } else {
          return self.finish_cell(spec, cell_tokens, col_index, ctx, start..end);
        }
      };

      if ctx.counting_cols && token.kind(TokenKind::Newline) {
        // once we've seen one newline, we finish the current cell (even if
        // it continues multiple lines) but we're done counting newlines
        // doesn't exactly match asciidoctor docs, but matches behavior
        ctx.counting_cols = false;
      }

      end = token.loc.end;
      // NB: allow for escaping a delimiter, but other backslashes should pass thru
      if !token.kind(TokenKind::Backslash) || !self.starts_psv_cell(tokens, ctx.cell_separator) {
        cell_tokens.push(token);
      } else if let Some(next) = tokens.consume_current() {
        end = next.loc.end;
        cell_tokens.push(next);
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use pretty_assertions::assert_eq;
  use test_utils::*;
  use ColWidth::*;

  const fn w(width: u8) -> ColWidth {
    Proportional(width)
  }

  #[test]
  fn test_parse_table() {
    assert_table!(
      adoc! {r#"
        [cols="1,1"]
        |===
        |c1, r1
        |c2, r1

        |c1, r2
        |c2, r2
        |===
      "#},
      Table {
        col_widths: ColWidths::new(vecb![w(1), w(1)]),
        rows: vecb![
          Row::new(vecb![
            cell!(d: "c1, r1", 19..25),
            cell!(d: "c2, r1", 27..33)
          ]),
          Row::new(vecb![
            cell!(d: "c1, r2", 36..42),
            cell!(d: "c2, r2", 44..50),
          ])
        ],
        ..empty_table!()
      }
    )
  }

  #[test]
  fn asciidoc_cell_content() {
    assert_table!(
      adoc! {r#"
        |===
        a| * foo |two
        |===
      "#},
      Table {
        col_widths: ColWidths::new(vecb![w(1), w(1)]),
        rows: vecb![Row::new(vecb![
          Cell {
            content: CellContent::AsciiDoc(Document {
              content: DocContent::Blocks(vecb![Block {
                meta: chunk_meta!(8),
                loc: (8..13).into(),
                content: BlockContent::List {
                  variant: ListVariant::Unordered,
                  depth: 1,
                  items: vecb![ListItem {
                    marker: ListMarker::Star(1),
                    marker_src: src!("*", 8..9),
                    principle: just!("foo", 10..13),
                    ..empty_list_item!()
                  }]
                },
                context: BlockContext::UnorderedList,
              }]),
              meta: doc_meta!(DocType::Article),
              ..Document::new(leaked_bump())
            }),
            ..empty_cell!()
          },
          cell!(d: "two", 15..18)
        ])],
        ..empty_table!()
      }
    )
  }

  #[test]
  fn indented_literal_in_asciidoc_cell() {
    let input = adoc! {r#"
      [cols="1a,1"]
      |===
      |
        literal
      | normal
      |===
    "#};
    assert_eq!(
      parse_table!(input).rows[0].cells[0],
      Cell {
        content: CellContent::AsciiDoc(Document {
          content: DocContent::Blocks(vecb![Block {
            context: BlockContext::Literal,
            content: BlockContent::Simple(just!("literal", 23..30)),
            loc: (23..30).into(),
            ..empty_block!(21)
          }]),
          meta: doc_meta!(DocType::Article),
          ..Document::new(leaked_bump())
        },),
        ..empty_cell!()
      }
    );
    // make sure our nested parser offsets are correct
    assert_eq!(&input[23..30], "literal");
  }

  #[test]
  fn test_table_escaped_separator() {
    assert_table!(
      adoc! {r#"
        |===
        |a \| b |c \| d \|
        |===
      "#},
      Table {
        col_widths: ColWidths::new(vecb![w(1), w(1)]),
        rows: vecb![Row::new(vecb![
          cell!(d: "a | b", 6..12),
          cell!(d: "c | d |", 14..23),
        ])],
        ..empty_table!()
      }
    )
  }

  #[test]
  fn trailing_sep_is_empty_cell() {
    assert_table!(
      adoc! {r#"
        |===
        |a |
        |c | d
        |===
      "#},
      Table {
        col_widths: ColWidths::new(vecb![w(1), w(1)]),
        rows: vecb![
          Row::new(vecb![cell!(d: "a", 6..7), empty_cell!()]),
          Row::new(vecb![cell!(d: "c", 11..12), cell!(d: "d", 15..16)])
        ],
        ..empty_table!()
      }
    )
  }

  #[test]
  fn custom_sep_psv() {
    assert_table!(
      adoc! {r#"
        [separator=;]
        |===
        ;a ; b ;  c
        |===
      "#},
      Table {
        col_widths: ColWidths::new(vecb![w(1), w(1), w(1)]),
        rows: vecb![Row::new(vecb![
          cell!(d: "a", 20..21),
          cell!(d: "b", 24..25),
          cell!(d: "c", 29..30),
        ])],
        ..empty_table!()
      }
    )
  }

  #[test]
  fn missing_first_sep_recovers() {
    assert_table_loose!(
      adoc! {r#"
        |===
        a | b | c
        |===
      "#},
      Table {
        col_widths: ColWidths::new(vecb![w(1), w(1), w(1)]),
        rows: vecb![Row::new(vecb![
          cell!(d: "a", 5..6),
          cell!(d: "b", 9..10),
          cell!(d: "c", 13..14),
        ])],
        ..empty_table!()
      }
    )
  }

  #[test]
  fn table_implicit_num_rows() {
    assert_table!(
      adoc! {r#"
        |===
        |c1, r1|c2, r1

        |c1, r2
        |c2, r2
        |===
      "#},
      Table {
        col_widths: ColWidths::new(vecb![w(1), w(1)]),
        header_row: Some(Row::new(vecb![
          cell!(d: "c1, r1", 6..12),
          cell!(d: "c2, r1", 13..19)
        ]),),
        rows: vecb![Row::new(vecb![
          cell!(d: "c1, r2", 22..28),
          cell!(d: "c2, r2", 30..36),
        ])],
        ..empty_table!()
      }
    )
  }

  #[test]
  fn table_implicit_num_rows_multiline_cell_content() {
    assert_table!(
      adoc! {r#"
        |===
        |c1, r1|c2,
        r1
        |c1, r2
        |c2, r2
        |===
      "#},
      Table {
        col_widths: ColWidths::new(vecb![w(1), w(1)]),
        rows: vecb![
          Row::new(vecb![
            cell!(d: "c1, r1", 6..12),
            Cell {
              content: CellContent::Default(vecb![nodes![
                node!("c2,"; 13..16),
                node!(Inline::Newline, 16..17),
                node!("r1"; 17..19),
              ]]),
              ..empty_cell!()
            }
          ]),
          Row::new(vecb![
            cell!(d: "c1, r2", 21..27),
            cell!(d: "c2, r2", 29..35),
          ])
        ],
        ..empty_table!()
      }
    )
  }

  #[test]
  fn basic_col_span() {
    let table = parse_table!(adoc! {r#"
      |===
      |1 | 2
      2+|3
      |4 | 5
      |===
    "#});
    assert_eq!(table.rows.len(), 3);
    assert_eq!(table.rows[0].cells.len(), 2);
    assert_eq!(table.rows[1].cells.len(), 1);
    assert_eq!(table.rows[2].cells.len(), 2);
  }

  #[test]
  fn count_cols_col_span() {
    let table = parse_table!(adoc! {r#"
      |===
      |1 2+| 2
      |3 | 4 | 5
      |===
    "#});
    assert_eq!(table.rows.len(), 2);
    assert_eq!(table.rows[0].cells.len(), 2);
    assert_eq!(table.rows[1].cells.len(), 3);
    assert_eq!(table.col_widths, ColWidths::new(vecb![w(1), w(1), w(1)]));
    let table = parse_table!(adoc! {r#"
      |===
      2+^|AAA |CCC
      |AAA |BBB |CCC
      |===
    "#});
    assert_eq!(table.rows.len(), 2);
    assert_eq!(table.col_widths.len(), 3);
    assert_eq!(table.rows[0].cells.len(), 2);
    assert_eq!(table.rows[1].cells.len(), 3);
  }

  #[test]
  fn basic_row_span() {
    let table = parse_table!(adoc! {r#"
      [cols="1,1"]
      |===
      |1 .2+|2
      |3
      |4 |5
      |===
    "#});
    assert_eq!(
      table,
      Table {
        col_widths: ColWidths::new(vecb![w(1), w(1)]),
        rows: vecb![
          Row::new(vecb![
            cell!(d: "1", 19..20),
            Cell { row_span: 2, ..cell!(d: "2", 25..26) }
          ]),
          Row::new(vecb![cell!(d: "3", 28..29)]),
          Row::new(vecb![cell!(d: "4", 31..32), cell!(d: "5", 34..35),]),
        ],
        ..empty_table!()
      }
    );
  }

  #[test]
  fn complex_row_col_spans() {
    assert_table!(
      adoc! {r#"
        |===
        |1 |2 |3 |4
        |5 2.2+|6 .3+|7
        |8
        |9 2+|10
        |===
      "#},
      Table {
        col_widths: ColWidths::new(vecb![w(1), w(1), w(1), w(1)]),
        rows: vecb![
          Row::new(vecb![
            cell!(d: "1", 6..7),
            cell!(d: "2", 9..10),
            cell!(d: "3", 12..13),
            cell!(d: "4", 15..16),
          ]),
          Row::new(vecb![
            cell!(d: "5", 18..19),
            Cell {
              col_span: 2,
              row_span: 2,
              ..cell!(d: "6", 25..26)
            },
            Cell { row_span: 3, ..cell!(d: "7", 31..32) },
          ]),
          Row::new(vecb![cell!(d: "8", 34..35)]),
          Row::new(vecb![
            cell!(d: "9", 37..38),
            Cell {
              col_span: 2,
              ..cell!(d: "10", 42..44)
            }
          ]),
        ],
        ..empty_table!()
      }
    );
  }

  #[test]
  fn literal_cell_only_escapes_special_chars() {
    assert_table!(
      adoc! {r#"
        |===
        l|one
        *two*
        three
        <four>
        |===
      "#},
      Table {
        col_widths: ColWidths::new(vecb![w(1)]),
        rows: vecb![Row::new(vecb![Cell {
          content: CellContent::Literal(nodes![
            node!("one"; 7..10),
            node!(Inline::Newline, 10..11),
            node!("*two*"; 11..16),
            node!(Inline::Newline, 16..17),
            node!("three"; 17..22),
            node!(Inline::Newline, 22..23),
            node!(Inline::SpecialChar(SpecialCharKind::LessThan), 23..24),
            node!("four"; 24..28),
            node!(Inline::SpecialChar(SpecialCharKind::GreaterThan), 28..29),
          ]),
          ..empty_cell!()
        }])],
        ..empty_table!()
      }
    )
  }

  #[test]
  fn literal_cell_spacing() {
    assert_table!(
      adoc! {r#"
        [cols="1,1"]
        |===
        l|
          one
          two
        three

          | normal
        |===
      "#},
      Table {
        col_widths: ColWidths::new(vecb![w(1), w(1)]),
        rows: vecb![Row::new(vecb![
          Cell {
            content: CellContent::Literal(nodes![
              node!("  one"; 21..26),
              node!(Inline::Newline, 26..27),
              node!("  two"; 27..32),
              node!(Inline::Newline, 32..33),
              node!("three"; 33..38),
            ]),
            ..empty_cell!()
          },
          cell!(d: "normal", 44..50),
        ])],
        ..empty_table!()
      }
    )
  }

  #[test]
  fn col_spec_multiplier() {
    assert_table!(
      adoc! {r#"
        [cols="3*"]
        |===
        |A |B |C |a |b |c
        |===
      "#},
      Table {
        col_widths: ColWidths::new(vecb![w(1), w(1), w(1)]),
        rows: vecb![
          Row::new(vecb![
            cell!(d: "A", 18..19),
            cell!(d: "B", 21..22),
            cell!(d: "C", 24..25),
          ]),
          Row::new(vecb![
            cell!(d: "a", 27..28),
            cell!(d: "b", 30..31),
            cell!(d: "c", 33..34),
          ]),
        ],
        ..empty_table!()
      }
    )
  }

  #[test]
  fn duplicate_cels() {
    let table = parse_table!(adoc! {r#"
      |===
      2*|dupe
      |===
    "#});
    assert_eq!(
      table.rows,
      vecb![Row::new(vecb![
        cell!(d: "dupe", 8..12),
        cell!(d: "dupe", 8..12),
      ]),]
    );
  }

  #[test]
  fn colspec_style_inheritance() {
    let table = parse_table!(adoc! {r#"
      [cols="e,s"]
      |===
      |one |two
      |1 d|2
      |===
    "#});
    assert_eq!(
      table.rows,
      vecb![
        Row::new(vecb![cell!(e: "one", 19..22), cell!(s: "two", 24..27)]),
        Row::new(vecb![cell!(e: "1", 29..30), cell!(d: "2", 33..34)]),
      ]
    );
  }

  #[test]
  fn implicit_header_row() {
    assert_table!(
      adoc! {r#"
        |===
        |one |two

        |1 |2
        |===
      "#},
      Table {
        col_widths: ColWidths::new(vecb![w(1), w(1)]),
        header_row: Some(Row::new(vecb![
          cell!(d: "one", 6..9),
          cell!(d: "two", 11..14),
        ])),
        rows: vecb![Row::new(vecb![
          cell!(d: "1", 17..18),
          cell!(d: "2", 20..21),
        ])],
        ..empty_table!()
      }
    )
  }

  #[test]
  fn supress_implicit_header_row() {
    let table = parse_table!(adoc! {r#"
      [%noheader]
      |===
      |one |two

      |1 |2
      |===
    "#});
    assert!(table.header_row.is_none());
    assert_eq!(table.rows.len(), 2);
  }

  #[test]
  fn explicit_header_row() {
    assert_table!(
      adoc! {r#"
        [%header]
        |===
        |one |two
        |1 |2
        |===
      "#},
      Table {
        col_widths: ColWidths::new(vecb![w(1), w(1)]),
        header_row: Some(Row::new(vecb![
          cell!(d: "one", 16..19),
          cell!(d: "two", 21..24),
        ])),
        rows: vecb![Row::new(vecb![
          cell!(d: "1", 26..27),
          cell!(d: "2", 29..30),
        ])],
        ..empty_table!()
      }
    )
  }

  #[test]
  fn footer_row() {
    assert_table!(
      adoc! {r#"
        [%footer]
        |===
        |one |two
        |1 |2
        |===
      "#},
      Table {
        col_widths: ColWidths::new(vecb![w(1), w(1)]),
        rows: vecb![Row::new(vecb![
          cell!(d: "one", 16..19),
          cell!(d: "two", 21..24),
        ])],
        footer_row: Some(Row::new(vecb![
          cell!(d: "1", 26..27),
          cell!(d: "2", 29..30),
        ])),
        ..empty_table!()
      }
    )
  }

  #[test]
  fn implicit_header_row_only() {
    let table = parse_table!(adoc! {r#"
      |===
      |one |two

      |===
    "#});
    assert!(table.header_row.is_some());
    assert!(table.rows.is_empty());
  }

  #[test]
  fn autowidth_cols() {
    let table = parse_table!(adoc! {r#"
      [%autowidth]
      |===
      |one |two
      |===
    "#});
    assert_eq!(table.col_widths, ColWidths::new(vecb![Auto, Auto]));
  }

  #[test]
  fn no_implicit_header_row_for_multiline_first_cell() {
    let table = parse_table!(adoc! {r#"
      [cols="2*"]
      |===
      |first cell

      first cell continued |second cell
      |===
    "#});
    assert!(table.header_row.is_none());
  }

  #[test]
  fn implicit_header_row_has_no_content_style() {
    let table = parse_table!(adoc! {r#"
      [cols="1a,2l"]
      |===
      |1|2

      |3|4
      |===
    "#});
    assert!(matches!(
      table.header_row.clone().unwrap().cells[0].content,
      CellContent::Default(_)
    ));
    assert!(matches!(
      table.header_row.clone().unwrap().cells[1].content,
      CellContent::Default(_)
    ));
  }

  #[test]
  fn explicit_header_row_has_no_content_style() {
    let table = parse_table!(adoc! {r#"
      [%header,cols="1a,2l"]
      |===
      |1|2
      |3|4
      |===
    "#});
    assert!(matches!(
      table.header_row.clone().unwrap().cells[0].content,
      CellContent::Default(_)
    ));
    assert!(matches!(
      table.header_row.clone().unwrap().cells[1].content,
      CellContent::Default(_)
    ));
  }

  #[test]
  fn explicit_noheader_row_has_content_style() {
    let table = parse_table!(adoc! {r#"
      [%noheader,cols="1a,2l"]
      |===
      |1|2

      |3|4
      |===
    "#});
    assert!(table.header_row.is_none());
    assert!(matches!(
      table.rows[0].cells[0].content,
      CellContent::AsciiDoc(_)
    ));
    assert!(matches!(
      table.rows[0].cells[1].content,
      CellContent::Literal(_)
    ));
  }

  #[test]
  fn table_empty_first_line() {
    assert_table!(
      adoc! {r#"
        |===

        |one |two
        |===
      "#},
      Table {
        col_widths: ColWidths::new(vecb![w(1), w(1)]),
        rows: vecb![Row::new(vecb![
          cell!(d: "one", 7..10),
          cell!(d: "two", 12..15),
        ])],
        ..empty_table!()
      }
    );
  }

  #[test]
  fn test_table_w_title() {
    assert_block!(
      adoc! {r"
        .Simple psv table
        |===
        |A |B |C
        |a |b |c
        |1 |2 |3
        |===
      "},
      Block {
        meta: ChunkMeta {
          title: Some(just!("Simple psv table", 1..17)),
          ..chunk_meta!(0, 1)
        },
        loc: (18..54).into(),
        content: BlockContent::Table(Table {
          col_widths: ColWidths::new(vecb![w(1), w(1), w(1)]),
          rows: vecb![
            Row::new(vecb![
              cell!(d: "A", 24..25),
              cell!(d: "B", 27..28),
              cell!(d: "C", 30..31),
            ]),
            Row::new(vecb![
              cell!(d: "a", 33..34),
              cell!(d: "b", 36..37),
              cell!(d: "c", 39..40),
            ]),
            Row::new(vecb![
              cell!(d: "1", 42..43),
              cell!(d: "2", 45..46),
              cell!(d: "3", 48..49),
            ]),
          ],
          ..empty_table!()
        }),
        context: BlockContext::Table,
      }
    )
  }

  assert_error!(
    no_table_end_delim,
    adoc! {r"
      |===
      |c1, r1
    "},
    error! {r"
       --> test.adoc:1:1
        |
      1 | |===
        | ^^^^ Table never closed, started here
    "}
  );

  assert_error!(
    no_cell_sep,
    adoc! {r"
      |===
      foo
      |===
    "},
    error! {r"
       --> test.adoc:2:1
        |
      2 | foo
        | ^ Expected cell separator `|`
    "}
  );

  assert_error!(
    cell_span_overflow,
    adoc! {r#"
      [cols="1,1"]
      |===
      3+|foo
      |===
    "#},
    error! {r"
       --> test.adoc:3:1
        |
      3 | 3+|foo
        | ^^ Cell column span (3) exceeds number of columns (2)
    "}
  );
}
