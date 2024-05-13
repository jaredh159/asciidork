use std::collections::HashSet;

use super::DataFormat;
use crate::internal::*;

#[derive(Debug, Clone)]
pub struct TableContext<'bmp> {
  pub delim_ch: u8,
  pub format: DataFormat,
  pub col_specs: BumpVec<'bmp, ColSpec>,
  pub num_cols: usize,
  pub counting_cols: bool,
  pub has_header_row: Option<bool>,
  pub autowidths: bool,
  pub can_infer_implicit_header: bool,
  pub phantom_cells: HashSet<(usize, usize)>,
  pub effective_row_idx: usize,
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
      format: DataFormat::Prefix(b'|'),
      col_specs: vecb![],
      num_cols: 3,
      counting_cols: false,
      has_header_row: None,
      autowidths: false,
      can_infer_implicit_header: false,
      phantom_cells: HashSet::new(),
      effective_row_idx: 0,
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
