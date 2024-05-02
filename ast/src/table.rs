#![allow(dead_code)]

use std::str::FromStr;

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
  pub col_specs: BumpVec<'bmp, ColSpec>, // do i actually need this?
  pub header_row: Option<Row<'bmp>>,
  pub rows: BumpVec<'bmp, Row<'bmp>>,
  pub footer_row: Option<Row<'bmp>>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ColSpec {
  pub width: u8,
  pub h_align: HorizontalAlignment,
  pub v_align: VerticalAlignment,
  pub style: CellContentStyle,
}

impl Default for ColSpec {
  fn default() -> Self {
    Self {
      width: 1,
      h_align: HorizontalAlignment::default(),
      v_align: VerticalAlignment::default(),
      style: CellContentStyle::default(),
    }
  }
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

impl FromStr for HorizontalAlignment {
  type Err = ();
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "<" => Ok(Self::Left),
      "^" => Ok(Self::Center),
      ">" => Ok(Self::Right),
      _ => Err(()),
    }
  }
}

impl FromStr for VerticalAlignment {
  type Err = ();
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "<" => Ok(Self::Top),
      "^" => Ok(Self::Middle),
      ">" => Ok(Self::Bottom),
      _ => Err(()),
    }
  }
}

impl FromStr for CellContentStyle {
  type Err = ();
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "a" => Ok(Self::AsciiDoc),
      "d" => Ok(Self::Default),
      "e" => Ok(Self::Emphasis),
      "h" => Ok(Self::Header),
      "l" => Ok(Self::Literal),
      "m" => Ok(Self::Monospace),
      "s" => Ok(Self::Strong),
      _ => Err(()),
    }
  }
}
