#![allow(dead_code)]

use crate::internal::*;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum HorizontalAlignment {
  Left,
  Center,
  Right,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum VerticalAlignment {
  Top,
  Middle,
  Bottom,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum CellContent<'bmp> {
  AsciiDoc(BumpVec<'bmp, Block<'bmp>>),
  Default(InlineNodes<'bmp>),
  Emphasis(InlineNodes<'bmp>),
  Header(InlineNodes<'bmp>),
  Literal(InlineNodes<'bmp>),
  Monospace(InlineNodes<'bmp>),
  Strong(InlineNodes<'bmp>),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Table<'bmp> {
  // pub num_cols: u8,
  pub col_specs: BumpVec<'bmp, ColSpec>,
  pub rows: BumpVec<'bmp, Row<'bmp>>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ColSpec {
  pub width: u8,
  // h_align: HorizontalAlignment,
  // v_align: VerticalAlignment,
  // style: CellStyle,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Cell<'bmp> {
  pub content: CellContent<'bmp>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Row<'bmp> {
  pub cells: BumpVec<'bmp, Cell<'bmp>>,
}

// struct Table<'bmp> {
//   header_row: Option<Row<'bmp>>,
//   rows: BumpVec<'bmp, Row<'bmp>>,
//   footer_row: Option<Row<'bmp>>,
// }
