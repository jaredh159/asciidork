#![allow(dead_code)]

use crate::internal::*;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub enum HorizontalAlignment {
  #[default]
  Left,
  Center,
  Right,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub enum VerticalAlignment {
  #[default]
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

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub enum CellContentStyle {
  AsciiDoc,
  #[default]
  Default,
  Emphasis,
  Header,
  Literal,
  Monospace,
  Strong,
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

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct CellSpec {
  pub duplication: Option<u8>,
  pub col_span: Option<u8>,
  pub row_span: Option<u8>,
  pub h_align: Option<HorizontalAlignment>,
  pub v_align: Option<VerticalAlignment>,
  pub style: Option<CellContentStyle>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Cell<'bmp> {
  pub content: CellContent<'bmp>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Row<'bmp> {
  pub cells: BumpVec<'bmp, Cell<'bmp>>,
}

impl<'bmp> Row<'bmp> {
  pub fn new(cells: BumpVec<'bmp, Cell<'bmp>>) -> Self {
    Self { cells }
  }
}
