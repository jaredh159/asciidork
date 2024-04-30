use std::ops::Range;

use super::{DataFormat, TableConfig, TableTokens};
use crate::internal::*;

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

    let config = TableConfig {
      delim_ch: first_token.lexeme.as_bytes()[0],
      format,
      col_specs,
    };

    let (mut tokens, end) = self.table_content(lines, &delim_line)?;
    let mut rows = bvec![in self.bump];
    let mut num_cols = config.col_specs.len();

    if config.col_specs.is_empty() {
      let mut first_row = bvec![in self.bump];
      while let Some(cell) = self.parse_table_cell(&mut tokens, &config, true)? {
        first_row.push(cell);
      }
      num_cols = first_row.len();
      rows.push(Row::new(first_row));
      while tokens.current().is(TokenKind::Newline) {
        tokens.consume_current();
      }
    }

    println!("num_cols: {}", num_cols);

    while let Some(row) = self.parse_table_row(&mut tokens, &config, num_cols)? {
      rows.push(row);
    }

    Ok(Block {
      content: BlockContent::Table(Table { col_specs: config.col_specs, rows }),
      context: BlockContext::Table,
      loc: SourceLocation::new(meta.start, end),
      meta,
    })
  }

  fn parse_table_cell(
    &mut self,
    tokens: &mut TableTokens<'bmp, 'src>,
    conf: &TableConfig,
    counting_cols: bool,
  ) -> Result<Option<Cell<'bmp>>> {
    if tokens.is_empty() {
      println!("finish 6 (empty tokens)");
      return Ok(None);
    }

    if counting_cols && tokens.current().is(TokenKind::Newline) {
      while tokens.current().is(TokenKind::Newline) {
        tokens.consume_current();
      }
      println!("finish 5 (done counting cols)");
      return Ok(None);
    }

    let (spec, start) = match self.consume_cell_start(tokens, conf.format.sep()) {
      Some((spec, start)) => (spec, start),
      None => {
        println!("finish 4 (no cell start)");
        self.err(
          format!(
            "Expected cell separator `{}`",
            char::from(conf.format.sep())
          ),
          tokens.nth(0),
        )?;
        (CellSpec::default(), tokens.current().unwrap().loc.start)
      }
    };

    let mut end = start;
    let mut cell_tokens = bvec![in self.bump];
    // trim leading whitespace
    while tokens.current().is(TokenKind::Whitespace) {
      tokens.consume_current();
    }
    loop {
      if self.starts_cell(tokens, conf.format.sep()) {
        println!("finish 1 (found next cell)");
        return self.finish_cell(spec, cell_tokens, conf, start..end);
      }
      if counting_cols && tokens.current().is(TokenKind::Newline) {
        println!("finish 2 (found newline, counting cols)");
        return self.finish_cell(spec, cell_tokens, conf, start..end);
      }
      let Some(token) = tokens.consume_current() else {
        println!("finish 3 (out of tokens)");
        return self.finish_cell(spec, cell_tokens, conf, start..end);
      };
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
    config: &TableConfig,
    num_cols: usize,
  ) -> Result<Option<Row<'bmp>>> {
    let mut cells = bvec![in self.bump];
    while let Some(cell) = self.parse_table_cell(tokens, config, false)? {
      cells.push(cell);
      if cells.len() == num_cols {
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
      if next_line.src == start_delim.src {
        return Ok((
          TableTokens::new(tokens, self.lexer.loc_src(start..end)),
          next_line.loc().unwrap().end,
        ));
      }
      if !tokens.is_empty() {
        tokens.push(Token {
          kind: TokenKind::Newline,
          lexeme: "\n",
          loc: SourceLocation::new(end, end + 1),
        });
        end += 1;
      }
      if let Some(loc) = next_line.last_loc() {
        end = loc.end;
      }
      next_line.drain_into(&mut tokens);
    }
    self.err_line("Table never closed, started here", start_delim)?;
    Ok((
      TableTokens::new(tokens, self.lexer.loc_src(start..end)),
      end,
    ))
  }

  fn finish_cell(
    &mut self,
    cell_spec: CellSpec,
    mut cell_tokens: BumpVec<'bmp, Token<'src>>,
    _conf: &TableConfig,
    loc: Range<usize>,
  ) -> Result<Option<Cell<'bmp>>> {
    while cell_tokens.last().is_whitespaceish() {
      cell_tokens.pop();
    }
    let inlines = if cell_tokens.is_empty() {
      InlineNodes::new(self.bump)
    } else {
      let cell_line = self.line_from(cell_tokens, loc);
      let mut cell_lines = cell_line.into_lines_in(self.bump);
      self.parse_inlines(&mut cell_lines)?
    };
    let content = match cell_spec.style {
      Some(CellContentStyle::Default) => CellContent::Default(inlines),
      Some(CellContentStyle::Emphasis) => CellContent::Emphasis(inlines),
      Some(CellContentStyle::Header) => CellContent::Header(inlines),
      Some(CellContentStyle::Literal) => CellContent::Literal(inlines),
      Some(CellContentStyle::Monospace) => CellContent::Monospace(inlines),
      Some(CellContentStyle::Strong) => CellContent::Strong(inlines),
      Some(CellContentStyle::AsciiDoc) => todo!("asciidoc"),
      None => CellContent::Default(inlines),
    };
    Ok(Some(Cell { content }))
  }
}

fn ends_table(line: &Line, conf: &TableConfig) -> bool {
  line.src.len() == 4
    && line.num_tokens() == 2
    && line.nth_token(1).is_len(TokenKind::EqualSigns, 3)
    && line.src.as_bytes().first() == Some(&conf.delim_ch)
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
        col_specs: vecb![ColSpec { width: 1 }, ColSpec { width: 1 }],
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
        col_specs: vecb![],
        rows: vecb![Row::new(vecb![
          cell!(d: "a | b", 6..12),
          cell!(d: "c | d |", 14..23),
        ])],
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
        col_specs: vecb![],
        rows: vecb![
          Row::new(vecb![
            cell!(d: "a", 6..7),
            Cell {
              content: CellContent::Default(InlineNodes::new(leaked_bump()))
            }
          ]),
          Row::new(vecb![cell!(d: "c", 11..12), cell!(d: "d", 15..16)])
        ],
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
        col_specs: vecb![],
        rows: vecb![Row::new(vecb![
          cell!(d: "a", 20..21),
          cell!(d: "b", 24..25),
          cell!(d: "c", 29..30),
        ]),],
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
        col_specs: vecb![],
        rows: vecb![Row::new(vecb![
          cell!(d: "a", 5..6),
          cell!(d: "b", 9..10),
          cell!(d: "c", 13..14),
        ]),],
      }
    )
  }

  #[test]
  fn test_parse_table_implicit_num_rows() {
    assert_table!(
      adoc! {r#"
        |===
        |c1, r1|c2, r1

        |c1, r2
        |c2, r2
        |===
      "#},
      Table {
        col_specs: vecb![],
        rows: vecb![
          Row::new(vecb![cell!(d: "c1, r1", 6..12), cell!(d: "c2, r1", 13..19)]),
          Row::new(vecb![
            cell!(d: "c1, r2", 22..28),
            cell!(d: "c2, r2", 30..36),
          ])
        ],
      }
    )
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
          col_specs: vecb![],
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
