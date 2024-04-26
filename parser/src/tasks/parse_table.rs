use crate::internal::*;

impl<'bmp, 'src> Parser<'bmp, 'src> {
  pub(crate) fn parse_table(
    &mut self,
    mut lines: ContiguousLines<'bmp, 'src>,
    meta: ChunkMeta<'bmp>,
  ) -> Result<Block<'bmp>> {
    let delim = lines.consume_current().unwrap();

    let col_specs = meta
      .attrs
      .as_ref()
      .and_then(|attrs| attrs.named("cols"))
      .map(|cols_attr| self.parse_col_specs(cols_attr))
      .unwrap_or_else(|| bvec![in self.bump]);

    let mut expected_cols = if col_specs.is_empty() {
      None
    } else {
      Some(col_specs.len() as u8)
    };

    let end = self.gather_table_lines(&mut lines, &delim)?;

    let mut rows = BumpVec::new_in(self.bump);
    while let Some(row) = self.parse_table_row(&mut lines, &mut expected_cols)? {
      rows.push(row);
    }

    Ok(Block {
      content: BlockContent::Table(Table { col_specs, rows }),
      context: BlockContext::Table,
      loc: SourceLocation::new(meta.start, end),
      meta,
    })
  }

  fn parse_table_row(
    &mut self,
    lines: &mut ContiguousLines<'bmp, 'src>,
    expected_cols: &mut Option<u8>,
  ) -> Result<Option<Row<'bmp>>> {
    if lines.is_empty() {
      return Ok(None);
    }
    if let Some(num_cols) = expected_cols {
      let cells = self.parse_table_cells(lines, Some(*num_cols))?;
      return Ok(Some(Row { cells }));
    }
    let Some(line) = lines.consume_current() else {
      return Ok(None);
    };
    let mut implicit_row = line.into_lines_in(self.bump);
    let cells = self.parse_table_cells(&mut implicit_row, None)?;
    *expected_cols = Some(cells.len() as u8);
    Ok(Some(Row { cells }))
  }

  fn parse_table_cells(
    &mut self,
    lines: &mut ContiguousLines<'bmp, 'src>,
    max: Option<u8>,
  ) -> Result<BumpVec<'bmp, Cell<'bmp>>> {
    let mut cells = bvec![in self.bump];
    let (sep_kind, sep_char) = (TokenKind::Pipe, b'|'); // todo, configurable

    let mut count = 0;
    loop {
      count += 1;
      if cells.len() == max.unwrap_or(u8::MAX) as usize {
        return Ok(cells);
      }
      let Some(mut line) = lines.consume_current() else {
        return Ok(cells);
      };
      if count > 50 {
        panic!("infinite loop");
      }
      if line.is_empty() {
        // dbg!(&lines);
        // panic!("empty line");
        continue;
      }

      let first_token = line.current_token();
      if first_token.is(sep_kind) {
        line.discard(1);
      } else {
        self.err(
          format!("Expected cell separator `{}`", char::from(sep_char)),
          first_token,
        )?;
      }
      let mut cell_lines = line.into_lines_in(self.bump);
      let inlines = self.parse_inlines(&mut cell_lines)?;
      cells.push(Cell {
        content: CellContent::Default(inlines),
      });
    }
  }

  fn gather_table_lines(
    &mut self,
    lines: &mut ContiguousLines<'bmp, 'src>,
    start_delim: &Line<'bmp, 'src>,
  ) -> Result<usize> {
    let mut end = start_delim.last_loc().unwrap().end;
    for line in lines.iter() {
      if let Some(loc) = line.last_loc() {
        end = loc.end;
      }
      if line.src == start_delim.src {
        return Ok(end);
      }
    }
    let mut more_lines = BumpVec::new_in(self.bump);
    while let Some(next_line) = self.read_line() {
      if let Some(loc) = next_line.last_loc() {
        end = loc.end;
      }
      if next_line.src == start_delim.src {
        lines.extend(more_lines);
        return Ok(end);
      } else {
        more_lines.push(next_line);
      }
    }
    self.err_line("Table never closed, started here", start_delim)?;
    Ok(end)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use test_utils::{assert_eq, *};

  #[test]
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
          Row {
            cells: vecb![
              Cell {
                content: CellContent::Default(just!("c1, r1", 19..25))
              },
              Cell {
                content: CellContent::Default(just!("c2, r1", 27..33))
              },
            ]
          },
          Row {
            cells: vecb![
              Cell {
                content: CellContent::Default(just!("c1, r2", 36..42))
              },
              Cell {
                content: CellContent::Default(just!("c2, r2", 44..50))
              },
            ]
          }
        ],
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
