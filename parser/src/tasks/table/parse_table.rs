use std::ops::Range;

use super::{DataFormat, TableContext, TableTokens};
use crate::internal::*;
use crate::variants::token::*;

impl<'bmp, 'src> Parser<'bmp, 'src> {
  pub(crate) fn parse_table(
    &mut self,
    mut lines: ContiguousLines<'bmp, 'src>,
    meta: ChunkMeta<'bmp>,
  ) -> Result<Block<'bmp>> {
    let delim_line = lines.consume_current().unwrap();
    let first_token = delim_line.current_token().unwrap();
    debug_assert!(first_token.lexeme.len() == 1);

    let col_specs = meta
      .attr_named("cols")
      .map(|cols_attr| self.parse_col_specs(cols_attr))
      .unwrap_or_else(|| bvec![in self.bump]);

    let format = meta
      .attr_named("separator")
      .and_then(|sep| match sep.len() {
        1 => Some(DataFormat::Prefix(sep.as_bytes()[0])),
        _ => None, // err ??
      })
      .unwrap_or(DataFormat::Prefix(b'|'));

    let mut ctx = TableContext {
      delim_ch: first_token.lexeme.as_bytes()[0],
      format,
      num_cols: col_specs.len(),
      counting_cols: col_specs.is_empty(),
      col_specs,
      has_header_row: None,
      can_infer_implicit_header: false,
    };

    if meta.has_attr_option("header") {
      ctx.has_header_row = Some(true);
    } else if meta.has_attr_option("noheader") || lines.num_lines() != 1 {
      ctx.has_header_row = Some(false);
    }

    let (mut tokens, end) = self.table_content(lines, &delim_line)?;
    let mut rows = bvec![in self.bump];
    let mut header_row = None;

    if ctx.counting_cols {
      self.parse_implicit_first_row(&mut tokens, &mut rows, &mut ctx)?;
      if rows.first().is_some()
        && (ctx.has_header_row == Some(true) || ctx.can_infer_implicit_header)
      {
        ctx.has_header_row = Some(true);
        header_row = rows.pop();
      } else if ctx.has_header_row.is_none() {
        ctx.has_header_row = Some(false);
      }
    }

    while let Some(row) = self.parse_table_row(&mut tokens, &mut ctx)? {
      if rows.is_empty()
        && header_row.is_none()
        && (ctx.has_header_row == Some(true) || ctx.can_infer_implicit_header)
      {
        header_row = Some(row);
        ctx.has_header_row = Some(true);
      } else {
        rows.push(row);
        if ctx.has_header_row.is_none() {
          ctx.has_header_row = Some(false);
        }
      }
    }

    Ok(Block {
      content: BlockContent::Table(Table {
        col_specs: ctx.col_specs,
        header_row,
        rows,
        footer_row: None,
      }),
      context: BlockContext::Table,
      loc: SourceLocation::new(meta.start, end),
      meta,
    })
  }

  fn parse_implicit_first_row(
    &mut self,
    tokens: &mut TableTokens<'bmp, 'src>,
    rows: &mut BumpVec<'bmp, Row<'bmp>>,
    ctx: &mut TableContext,
  ) -> Result<()> {
    let mut cells = bvec![in self.bump];
    loop {
      let Some(cell) = self.parse_table_cell(tokens, ctx, cells.len())? else {
        break;
      };
      cells.push(cell);
      if !ctx.counting_cols {
        break;
      }
    }
    ctx.num_cols = cells.len();
    rows.push(Row::new(cells));
    while tokens.current().is(TokenKind::Newline) {
      tokens.consume_current();
    }
    Ok(())
  }

  fn parse_table_cell(
    &mut self,
    tokens: &mut TableTokens<'bmp, 'src>,
    ctx: &mut TableContext,
    col_index: usize,
  ) -> Result<Option<Cell<'bmp>>> {
    if tokens.is_empty() {
      println!("finish 6 (empty tokens)");
      return Ok(None);
    }

    let (spec, mut start) = match self.consume_cell_start(tokens, ctx.format.sep()) {
      Some((spec, start)) => (spec, start),
      None => {
        println!("finish 4 (no cell start)");
        let sep = char::from(ctx.format.sep());
        self.err(format!("Expected cell separator `{}`", sep), tokens.nth(0))?;
        (CellSpec::default(), tokens.current().unwrap().loc.start)
      }
    };

    let mut end = start;
    let mut cell_tokens = bvec![in self.bump];

    // trim leading whitespace
    while tokens.current().is(TokenKind::Whitespace) {
      let trimmed = tokens.consume_current().unwrap();
      start = trimmed.loc.end;
    }

    loop {
      if self.starts_cell(tokens, ctx.format.sep()) {
        println!("finish 1 (found next cell)",);
        return self.finish_cell(spec, cell_tokens, col_index, ctx, start..end);
      }
      let Some(token) = tokens.consume_current() else {
        println!("finish 3 (out of tokens)");
        return self.finish_cell(spec, cell_tokens, col_index, ctx, start..end);
      };

      if token.is(TokenKind::Newline) {
        // once we've seen one newline, we finish the current cell (even if
        // it continues multiple lines) but we're done counting newlines
        // doesn't exactly match asciidoctor docs, but matches behavior
        ctx.counting_cols = false;
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

  fn parse_table_row(
    &mut self,
    tokens: &mut TableTokens<'bmp, 'src>,
    ctx: &mut TableContext,
  ) -> Result<Option<Row<'bmp>>> {
    let mut cells = bvec![in self.bump];
    while let Some(cell) = self.parse_table_cell(tokens, ctx, cells.len())? {
      cells.push(cell);
      if cells.len() == ctx.num_cols {
        break;
      }
    }
    if cells.is_empty() {
      Ok(None)
    } else {
      Ok(Some(Row::new(cells)))
    }
  }

  fn table_content(
    &mut self,
    mut lines: ContiguousLines<'bmp, 'src>,
    start_delim: &Line<'bmp, 'src>,
  ) -> Result<(TableTokens<'bmp, 'src>, usize)> {
    let mut tokens = BumpVec::with_capacity_in(lines.num_tokens(), self.bump);
    let delim_loc = start_delim.last_loc().unwrap();
    let start = delim_loc.end + 1;
    let mut end = delim_loc.end + 1;
    while let Some(line) = lines.consume_current() {
      if line.src == start_delim.src {
        self.restore_lines(lines);
        return Ok((
          TableTokens::new(tokens, self.lexer.loc_src(start..end)),
          line.loc().unwrap().end,
        ));
      }
      if let Some(loc) = line.last_loc() {
        end = loc.end;
      }
      line.drain_into(&mut tokens);
      if !lines.is_empty() {
        tokens.push(Token {
          kind: TokenKind::Newline,
          lexeme: "\n",
          loc: SourceLocation::new(end, end + 1),
        });
        end += 1;
      }
    }
    while let Some(next_line) = self.read_line() {
      if !tokens.is_empty() {
        tokens.push(Token {
          kind: TokenKind::Newline,
          lexeme: "\n",
          loc: SourceLocation::new(end, end + 1),
        });
        end += 1;
      }
      if next_line.src == start_delim.src {
        return Ok((
          TableTokens::new(tokens, self.lexer.loc_src(start..end)),
          next_line.loc().unwrap().end,
        ));
      }
      if let Some(loc) = next_line.last_loc() {
        end = loc.end;
      }
      next_line.drain_into(&mut tokens);
    }
    self.err_line("Table never closed, started here", start_delim)?;
    let loc = self.lexer.loc_src(start..end);
    Ok((TableTokens::new(tokens, loc), end))
  }

  fn finish_cell(
    &mut self,
    cell_spec: CellSpec,
    mut cell_tokens: BumpVec<'bmp, Token<'src>>,
    col_index: usize,
    ctx: &mut TableContext,
    mut loc: Range<usize>,
  ) -> Result<Option<Cell<'bmp>>> {
    dbg!(&cell_tokens);
    let cell_style = cell_spec.style.unwrap_or_else(|| {
      ctx
        .col_specs
        .get(col_index)
        .map_or(CellContentStyle::Default, |colspec| colspec.style)
    });

    if ctx.has_header_row.is_none() {
      let mut ws = SmallVec::<[TokenKind; 12]>::new();
      while cell_tokens.last().is_whitespaceish() {
        let token = cell_tokens.pop().unwrap();
        loc.end = token.loc.start;
        ws.push(token.kind);
      }
      if ws.len() > 1 && ws[ws.len() - 2..] == [Newline, Newline] {
        ctx.can_infer_implicit_header = true;
      }
    } else {
      ctx.can_infer_implicit_header = false;
      while cell_tokens.last().is_whitespaceish() {
        loc.end = cell_tokens.pop().unwrap().loc.start;
      }
    }

    if cell_style == CellContentStyle::AsciiDoc {
      let mut cell_line = self.line_from(cell_tokens, loc.clone());
      cell_line.trim_for_cell(cell_style);
      let cell_parser = self.nest(cell_line.src, loc.start);
      return match cell_parser.parse() {
        Ok(ParseResult { document, warnings }) => {
          self.errors.borrow_mut().extend(warnings);
          Ok(Some(Cell {
            content: CellContent::AsciiDoc(document.content),
          }))
        }
        Err(mut diagnostics) => {
          if !diagnostics.is_empty() && self.strict {
            Err(diagnostics.remove(0))
          } else {
            self.errors.borrow_mut().extend(diagnostics);
            Ok(None)
          }
        }
      };
    }

    let inlines = if cell_tokens.is_empty() {
      InlineNodes::new(self.bump)
    } else {
      let mut cell_line = self.line_from(cell_tokens, loc);
      cell_line.trim_for_cell(cell_style);
      let prev_subs = self.ctx.subs;
      self.ctx.subs = cell_style.into();
      let inlines = self.parse_inlines(&mut cell_line.into_lines_in(self.bump))?;
      self.ctx.subs = prev_subs;
      inlines
    };

    let content = match cell_style {
      CellContentStyle::Default => CellContent::Default(inlines),
      CellContentStyle::Emphasis => CellContent::Emphasis(inlines),
      CellContentStyle::Header => CellContent::Header(inlines),
      CellContentStyle::Literal => CellContent::Literal(inlines),
      CellContentStyle::Monospace => CellContent::Monospace(inlines),
      CellContentStyle::Strong => CellContent::Strong(inlines),
      CellContentStyle::AsciiDoc => unreachable!("Parser::finish_cell() asciidoc"),
    };

    Ok(Some(Cell { content }))
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use test_utils::{assert_eq, *};

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
        col_specs: vecb![ColSpec::default(), ColSpec::default()],
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
        rows: vecb![Row::new(vecb![
          Cell {
            content: CellContent::AsciiDoc(DocContent::Blocks(vecb![Block {
              meta: ChunkMeta::empty(8),
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
              loc: SourceLocation::new(8, 13)
            }]))
          },
          cell!(d: "two", 15..18)
        ]),],
        ..empty_table!()
      }
    )
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
        rows: vecb![
          Row::new(vecb![
            cell!(d: "a", 6..7),
            Cell {
              content: CellContent::Default(InlineNodes::new(leaked_bump()))
            }
          ]),
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
        rows: vecb![
          Row::new(vecb![
            cell!(d: "c1, r1", 6..12),
            Cell {
              content: CellContent::Default(nodes![
                node!("c2,"; 13..16),
                node!(Inline::Newline, 16..17),
                node!("r1"; 17..19),
              ])
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
          ])
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
        col_specs: vecb![ColSpec::default(), ColSpec::default()],
        rows: vecb![Row::new(vecb![
          Cell {
            content: CellContent::Literal(nodes![
              node!("  one"; 21..26),
              node!(Inline::Newline, 26..27),
              node!("  two"; 27..32),
              node!(Inline::Newline, 32..33),
              node!("three"; 33..38),
            ])
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
        col_specs: vecb![ColSpec::default(), ColSpec::default(), ColSpec::default()],
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
          attrs: None,
          title: Some(just!("Simple psv table", 1..17)),
          start: 0
        },
        content: BlockContent::Table(Table {
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
        loc: SourceLocation::new(0, 51),
      }
    )
  }

  test_error!(
    no_table_end_delim,
    adoc! {r"
      |===
      |c1, r1
    "},
    error! {r"
      1: |===
         ^^^^ Table never closed, started here
    "}
  );

  test_error!(
    no_cell_sep,
    adoc! {r"
      |===
      foo
      |===
    "},
    error! {r"
      2: foo
         ^ Expected cell separator `|`
    "}
  );
}
