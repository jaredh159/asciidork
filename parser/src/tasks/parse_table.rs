use crate::internal::*;

#[derive(Debug, Clone)]
struct TableConfig<'bmp> {
  delim_start: u8,
  format: DataFormat,
  col_specs: BumpVec<'bmp, ColSpec>,
}

#[derive(Debug, Clone, Copy)]
enum DataFormat {
  Prefix(u8),
  Csv(u8),
  Delimited(u8),
}

impl<'bmp, 'src> Parser<'bmp, 'src> {
  pub(crate) fn parse_table(
    &mut self,
    mut lines: ContiguousLines<'bmp, 'src>,
    meta: ChunkMeta<'bmp>,
  ) -> Result<Block<'bmp>> {
    let delim_line = lines.consume_current().unwrap();
    let first_token = delim_line.current_token().unwrap();
    debug_assert!(first_token.lexeme.len() == 1);

    self.restore_lines(lines);

    let col_specs = meta
      .attrs
      .as_ref()
      .and_then(|attrs| attrs.named("cols"))
      .map(|cols_attr| self.parse_col_specs(cols_attr))
      .unwrap_or_else(|| bvec![in self.bump]);

    let config = TableConfig {
      delim_start: first_token.lexeme.as_bytes()[0],
      format: DataFormat::Prefix(b'|'), // todo derive
      col_specs,
    };

    let mut rows = bvec![in self.bump];

    let mut num_cols = config.col_specs.len();
    if config.col_specs.is_empty() {
      let mut first_row = bvec![in self.bump];
      while let Some(cell) = self.parse_table_cell(&config, true)? {
        first_row.push(cell);
      }
      num_cols = first_row.len();
      rows.push(Row::new(first_row));
    }

    while let Some(row) = self.parse_table_row(&config, num_cols)? {
      rows.push(row);
    }

    Ok(Block {
      content: BlockContent::Table(Table { col_specs: config.col_specs, rows }),
      context: BlockContext::Table,
      loc: SourceLocation::new(meta.start, 999), // _end
      meta,
    })
  }

  fn parse_table_cell(
    &mut self,
    config: &TableConfig,
    counting_cols: bool,
  ) -> Result<Option<Cell<'bmp>>> {
    _ = config;
    while let Some(mut lines) = self.read_lines() {
      while let Some(mut line) = lines.consume_current() {
        dbg!(line.src);
        let mut cell_tokens = bvec![in self.bump];
        let _cell_start = line.loc().expect("maybe?").start;
        // parse an optional cell spec
        // parse prefix
        // then, parse until:
        //  - we hit the beginning of another cell
        //  - or we hit end of line (if counting lines)
        //  - or we hit the end of the table
        //  - or we run out of lines
        loop {
          let Some(token) = line.consume_current() else {
            // we hit the end of a line
            if counting_cols {
              return Ok(None);
            } else {
              break; // read another line
            }
          };

          dbg!(&token);
          cell_tokens.push(token);

          todo!("parse_table_cell()")
        }
      } // consume_current LINE
    } // read_lines()
    Ok(None)
  }

  fn parse_table_row(
    &mut self,
    config: &TableConfig,
    num_cols: usize,
  ) -> Result<Option<Row<'bmp>>> {
    let mut cells = bvec![in self.bump];
    while let Some(cell) = self.parse_table_cell(config, true)? {
      cells.push(cell);
      if cells.len() == num_cols {
        break;
      }
    }
    if cells.is_empty() {
      Ok(None)
    } else {
      // err?
      Ok(Some(Row::new(cells)))
    }
  }

  // fn parse_table_row(
  //   &mut self,
  //   lines: &mut ContiguousLines<'bmp, 'src>,
  //   expected_cols: &mut Option<u8>,
  // ) -> Result<Option<Row<'bmp>>> {
  //   if lines.is_empty() {
  //     return Ok(None);
  //   }
  //   if let Some(num_cols) = expected_cols {
  //     let cells = self.parse_table_cells(lines, Some(*num_cols))?;
  //     return Ok(Some(Row { cells }));
  //   }
  //   let Some(line) = lines.consume_current() else {
  //     return Ok(None);
  //   };
  //   let mut implicit_row = line.into_lines_in(self.bump);
  //   let cells = self.parse_table_cells(&mut implicit_row, None)?;
  //   *expected_cols = Some(cells.len() as u8);
  //   Ok(Some(Row { cells }))
  // }

  // fn parse_table_cells(
  //   &mut self,
  //   lines: &mut ContiguousLines<'bmp, 'src>,
  //   max: Option<u8>,
  // ) -> Result<BumpVec<'bmp, Cell<'bmp>>> {
  //   dbg!(&lines);
  //   let mut cells = bvec![in self.bump];
  //   let (sep_kind, sep_char) = (TokenKind::Pipe, b'|'); // todo, configurable
  //   loop {
  //     if cells.len() == max.unwrap_or(u8::MAX) as usize {
  //       return Ok(cells);
  //     }
  //     let Some(mut line) = lines.consume_current() else {
  //       return Ok(cells);
  //     };
  //     if line.is_empty() {
  //       continue;
  //     }
  //     let first_token = line.current_token();
  //     if first_token.is(sep_kind) {
  //       line.discard(1);
  //     } else if cells.is_empty() {
  //       self.err(
  //         format!("Expected cell separator `{}`", char::from(sep_char)),
  //         first_token,
  //       )?;
  //     }
  //     // let mut cell_lines = line.into_lines_in(self.bump);
  //     // while !cell_lines.is_empty() {
  //     dbg!(line.src);
  //     lines.restore_if_nonempty(line);
  //     let inlines = self.parse_inlines_until(lines, &[sep_kind])?;
  //     // inlines.discard_trailing_newline();
  //     // inlines.
  //     cells.push(Cell {
  //       content: CellContent::Default(inlines),
  //     });
  //     // }
  //   }
  // }

  // fn gather_table_lines(
  //   &mut self,
  //   lines: &mut ContiguousLines<'bmp, 'src>,
  //   start_delim: &Line<'bmp, 'src>,
  // ) -> Result<usize> {
  //   let mut end = start_delim.last_loc().unwrap().end;
  //   for line in lines.iter() {
  //     if let Some(loc) = line.last_loc() {
  //       end = loc.end;
  //     }
  //     if line.src == start_delim.src {
  //       return Ok(end);
  //     }
  //   }
  //   let mut more_lines = BumpVec::new_in(self.bump);
  //   while let Some(next_line) = self.read_line() {
  //     if let Some(loc) = next_line.last_loc() {
  //       end = loc.end;
  //     }
  //     if next_line.src == start_delim.src {
  //       lines.extend(more_lines);
  //       return Ok(end);
  //     } else {
  //       more_lines.push(next_line);
  //     }
  //   }
  //   self.err_line("Table never closed, started here", start_delim)?;
  //   Ok(end)
  // }
}

#[cfg(test)]
mod tests {
  use super::*;
  use test_utils::{assert_eq, *};

  // #[test]
  fn test_parse_table() {
    let input = adoc! {r#"
      [cols="1,1"]
      |===
      |c1, r1
      |c2, r1

      |c1, r2
      |c2, r2
      |===
    "#};

    let block = parse_single_block!(input);
    let table = match block.content {
      BlockContent::Table(table) => table,
      _ => panic!("unexpected block content"),
    };
    assert_eq!(
      table,
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

  // #[test]
  fn test_parse_table_implicit_num_rows() {
    let input = adoc! {r#"
      |===
      |c1, r1|c2, r1

      |c1, r2
      |c2, r2
      |===
    "#};

    let block = parse_single_block!(input);
    let table = match block.content {
      BlockContent::Table(table) => table,
      _ => panic!("unexpected block content"),
    };
    assert_eq!(
      table,
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

  // test_error!(
  //   no_table_end_delim,
  //   adoc! {r"
  //     |===
  //     |c1, r1
  //   "},
  //   error! {r"
  //     1: |===
  //        ^^^^ Table never closed, started here
  //   "}
  // );

  // test_error!(
  //   no_cell_sep,
  //   adoc! {r"
  //     |===
  //     foo
  //     |===
  //   "},
  //   error! {r"
  //     2: foo
  //        ^ Expected cell separator `|`
  //   "}
  // );
}
