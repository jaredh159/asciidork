use std::ops::Range;

use crate::internal::*;

#[derive(Debug, Clone)]
struct TableConfig<'bmp> {
  delim_ch: u8,
  format: DataFormat,
  col_specs: BumpVec<'bmp, ColSpec>,
}

#[derive(Debug, Clone, Copy)]
enum DataFormat {
  Prefix(u8),
  Csv(u8),
  Delimited(u8),
}

#[derive(Debug, Clone)]
pub struct TableTokens<'bmp, 'src>(Line<'bmp, 'src>);

impl<'bmp, 'src> TableTokens<'bmp, 'src> {
  pub fn new(tokens: BumpVec<'bmp, Token<'src>>, src: &'src str) -> Self {
    Self(Line::new(tokens, src))
  }

  pub fn discard(&mut self, n: usize) {
    self.0.discard(n);
  }

  pub fn current(&self) -> Option<&Token<'src>> {
    self.0.current_token()
  }

  pub fn current_mut(&mut self) -> Option<&mut Token<'src>> {
    self.0.current_token_mut()
  }

  pub fn nth(&self, n: usize) -> Option<&Token<'src>> {
    self.0.nth_token(n)
  }

  pub fn has_seq_at(&self, kinds: &[TokenKind], offset: usize) -> bool {
    self.0.has_seq_at(kinds, offset)
  }

  pub fn consume_current(&mut self) -> Option<Token<'src>> {
    self.0.consume_current()
  }

  pub fn is_empty(&self) -> bool {
    self.0.is_empty()
  }
}

impl DataFormat {
  pub const fn sep(&self) -> u8 {
    match self {
      DataFormat::Prefix(c) => *c,
      DataFormat::Csv(c) => *c,
      DataFormat::Delimited(c) => *c,
    }
  }
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

    let col_specs = meta
      .attrs
      .as_ref()
      .and_then(|attrs| attrs.named("cols"))
      .map(|cols_attr| self.parse_col_specs(cols_attr))
      .unwrap_or_else(|| bvec![in self.bump]);

    let config = TableConfig {
      delim_ch: first_token.lexeme.as_bytes()[0],
      format: DataFormat::Prefix(b'|'), // todo derive
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
    let Some((spec, start)) = self.consume_cell_start(tokens, conf.format.sep()) else {
      return Ok(None);
    };
    let mut end = start;
    let mut cell_tokens = bvec![in self.bump];
    loop {
      if self.starts_cell(tokens, conf.format.sep()) {
        return self.finish_cell(spec, cell_tokens, conf, start..end);
      }
      let Some(token) = tokens.consume_current() else {
        return self.finish_cell(spec, cell_tokens, conf, start..end);
      };
      if counting_cols && token.is(TokenKind::Newline) {
        return self.finish_cell(spec, cell_tokens, conf, start..end);
      }
      end = token.loc.end;
      cell_tokens.push(token);
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
    while cell_tokens.last().is(TokenKind::Newline) {
      cell_tokens.pop();
    }
    let cell_line = self.line_from(cell_tokens, loc);
    let mut cell_lines = cell_line.into_lines_in(self.bump);
    let inlines = self.parse_inlines(&mut cell_lines)?;
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

  #[test]
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
