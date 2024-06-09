use std::collections::HashSet;

use super::DataFormat;
use crate::internal::*;

#[derive(Debug, Clone)]
pub struct TableContext<'bmp> {
  pub delim_ch: u8,
  pub format: DataFormat,
  pub cell_separator: char,
  pub embeddable_cell_separator: Option<char>,
  pub cell_separator_tokenkind: Option<TokenKind>,
  pub col_specs: BumpVec<'bmp, ColSpec>,
  pub num_cols: usize,
  pub counting_cols: bool,
  pub header_row: HeaderRow,
  pub header_reparse_cells: BumpVec<'bmp, ParseCellData<'bmp>>,
  pub autowidths: bool,
  pub phantom_cells: HashSet<(usize, usize)>,
  pub effective_row_idx: usize,
  pub dsv_last_consumed: DsvLastConsumed,
  pub table: Table<'bmp>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeaderRow {
  Unknown,
  FoundImplicit,
  FoundNone,
  ExplicitlySet,
  ExplicitlyUnset,
}

impl HeaderRow {
  pub const fn known_to_exist(&self) -> bool {
    matches!(self, HeaderRow::FoundImplicit | HeaderRow::ExplicitlySet)
  }

  pub const fn is_unknown(&self) -> bool {
    matches!(self, HeaderRow::Unknown)
  }
}

#[derive(Debug, Clone)]
pub struct ParseCellData<'bmp> {
  pub cell_tokens: BumpVec<'bmp, Token<'bmp>>,
  pub loc: SourceLocation,
  pub cell_spec: CellSpec,
  pub col_spec: Option<ColSpec>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DsvLastConsumed {
  Delimiter,
  Newline,
  Other,
}

impl<'bmp> TableContext<'bmp> {
  pub fn add_phantom_cells(&mut self, cell: &Cell, col: usize) {
    if cell.row_span == 0 && cell.col_span == 0 {
      return;
    }
    for row_offset in 0..cell.row_span {
      for col_offset in 0..cell.col_span {
        if row_offset == 0 && col_offset == 0 {
          continue;
        }
        self.phantom_cells.insert((
          self.effective_row_idx + row_offset as usize,
          col + col_offset as usize,
        ));
      }
    }
  }

  pub fn row_phantom_cells(&self) -> usize {
    self
      .phantom_cells
      .iter()
      .filter(|(row, _)| *row == self.effective_row_idx)
      .count()
  }

  pub fn effective_row_cols(&self) -> usize {
    let mut cols = self.num_cols;
    for (row, _) in &self.phantom_cells {
      if *row == self.effective_row_idx {
        cols -= 1;
      }
    }
    cols
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use test_utils::{assert_eq, *};

  #[test]
  fn test_table_context() {
    let mut ctx = TableContext {
      delim_ch: b'|',
      format: DataFormat::Prefix('|'),
      cell_separator: '|',
      embeddable_cell_separator: None,
      cell_separator_tokenkind: Some(TokenKind::Pipe),
      col_specs: vecb![],
      num_cols: 3,
      counting_cols: false,
      header_row: HeaderRow::Unknown,
      header_reparse_cells: vecb![],
      autowidths: false,
      phantom_cells: HashSet::new(),
      effective_row_idx: 0,
      dsv_last_consumed: DsvLastConsumed::Other,
      table: Table {
        col_widths: ColWidths::new(vecb![]),
        header_row: None,
        rows: vecb![],
        footer_row: None,
      },
    };

    assert_eq!(ctx.effective_row_cols(), 3);

    let cell = Cell {
      row_span: 2,
      col_span: 2,
      ..empty_cell!()
    };
    ctx.add_phantom_cells(&cell, 1);
    let mut expected = HashSet::new();
    expected.insert((0, 2));
    expected.insert((1, 1));
    expected.insert((1, 2));
    assert_eq!(ctx.phantom_cells, expected);
    assert_eq!(ctx.effective_row_cols(), 2);

    ctx.effective_row_idx = 1;
    assert_eq!(ctx.effective_row_cols(), 1);

    ctx.num_cols = 2;
    ctx.phantom_cells.clear();
    ctx.effective_row_idx = 0;

    let cell = Cell { row_span: 2, ..empty_cell!() };
    ctx.add_phantom_cells(&cell, 1);
    ctx.effective_row_idx = 1;
    assert_eq!(ctx.effective_row_cols(), 1);

    ctx.num_cols = 4;
    ctx.phantom_cells.clear();
    ctx.effective_row_idx = 1;

    let cell = Cell { row_span: 3, ..empty_cell!() };
    ctx.add_phantom_cells(&cell, 3);
    let expected = vec![(2, 3), (3, 3)];
    assert_eq!(ctx.phantom_cells, expected.into_iter().collect());
  }
}
