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
  AsciiDoc(Document<'bmp>),
  Default(BumpVec<'bmp, InlineNodes<'bmp>>),
  Emphasis(BumpVec<'bmp, InlineNodes<'bmp>>),
  Header(BumpVec<'bmp, InlineNodes<'bmp>>),
  Literal(InlineNodes<'bmp>),
  Monospace(BumpVec<'bmp, InlineNodes<'bmp>>),
  Strong(BumpVec<'bmp, InlineNodes<'bmp>>),
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

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ColWidth {
  Proportional(u8),
  Percentage(u8),
  Auto,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ColSpec {
  pub width: ColWidth,
  pub h_align: HorizontalAlignment,
  pub v_align: VerticalAlignment,
  pub style: CellContentStyle,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Table<'bmp> {
  pub col_widths: ColWidths<'bmp>,
  pub header_row: Option<Row<'bmp>>,
  pub rows: BumpVec<'bmp, Row<'bmp>>,
  pub footer_row: Option<Row<'bmp>>,
}

impl Default for ColSpec {
  fn default() -> Self {
    Self {
      width: ColWidth::Proportional(1),
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
  pub col_span: u8,
  pub row_span: u8,
  pub h_align: HorizontalAlignment,
  pub v_align: VerticalAlignment,
}

impl<'bmp> Cell<'bmp> {
  pub fn new(content: CellContent<'bmp>, cell_spec: CellSpec, col_spec: Option<&ColSpec>) -> Self {
    Self {
      content,
      col_span: cell_spec.col_span.unwrap_or(1),
      row_span: cell_spec.row_span.unwrap_or(1),
      h_align: cell_spec
        .h_align
        .unwrap_or(col_spec.map_or(HorizontalAlignment::Left, |cs| cs.h_align)),
      v_align: cell_spec
        .v_align
        .unwrap_or(col_spec.map_or(VerticalAlignment::Top, |cs| cs.v_align)),
    }
  }
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

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TableSection {
  Header,
  Body,
  Footer,
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

impl VerticalAlignment {
  pub const fn word(&self) -> &'static str {
    match self {
      Self::Top => "top",
      Self::Middle => "middle",
      Self::Bottom => "bottom",
    }
  }
}

impl HorizontalAlignment {
  pub const fn word(&self) -> &'static str {
    match self {
      Self::Left => "left",
      Self::Center => "center",
      Self::Right => "right",
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
