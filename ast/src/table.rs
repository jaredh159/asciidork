use std::ops::{Deref, DerefMut};
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
  AsciiDoc(DocContent<'bmp>),
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
  pub col_specs: BumpVec<'bmp, ColSpec>, // do i actually need this?
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

#[derive(Debug, PartialEq, Eq)]
pub struct ColWidths<'bmp>(BumpVec<'bmp, ColWidth>);

impl Deref for ColWidths<'_> {
  type Target = [ColWidth];
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl DerefMut for ColWidths<'_> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum DistributedColWidth {
  Percentage(f32),
  Auto(f32),
}

impl<'bmp> ColWidths<'bmp> {
  pub fn new(col_widths: BumpVec<'bmp, ColWidth>) -> Self {
    Self(col_widths)
  }

  pub fn distribute(&self) -> impl ExactSizeIterator<Item = DistributedColWidth> + '_ {
    let mut width_divisor = Option::<u8>::None;
    let mut num_autowidth = 0;
    for cw in self.iter() {
      match cw {
        ColWidth::Proportional(n) | ColWidth::Percentage(n) => {
          width_divisor = Some(width_divisor.map_or(*n, |sum| sum + n))
        }
        ColWidth::Auto => num_autowidth += 1,
      }
    }

    let autowidth = if let Some(width) = width_divisor {
      if (num_autowidth > 0 && width > 100) || num_autowidth == 0 {
        0.0
      } else {
        width_divisor = Some(100);
        (100 - width) as f32 / num_autowidth as f32
      }
    } else {
      100.0 / num_autowidth as f32
    };

    self.iter().map(move |cw| match cw {
      ColWidth::Proportional(n) | ColWidth::Percentage(n) => {
        DistributedColWidth::Percentage(*n as f32 * 100.0 / width_divisor.unwrap() as f32)
      }
      // ColWidth::Percentage(p) => DistributedColWidth::Percentage(*p as f32),
      ColWidth::Auto => DistributedColWidth::Auto(autowidth),
    })
  }
}

#[test]
fn test_distribute_col_width() {
  use ColWidth as CW;
  use DistributedColWidth as DCW;
  let cases: &[(&[CW], &[DCW])] = &[
    (&[CW::Proportional(1)], &[DCW::Percentage(100.0)]),
    (
      &[CW::Auto, CW::Auto, CW::Auto],
      &[
        DCW::Auto(100.0 / 3.0),
        DCW::Auto(100.0 / 3.0),
        DCW::Auto(100.0 / 3.0),
      ],
    ),
    (
      &[
        CW::Proportional(1),
        CW::Proportional(1),
        CW::Proportional(2),
      ],
      &[
        DCW::Percentage(25.0),
        DCW::Percentage(25.0),
        DCW::Percentage(50.0),
      ],
    ),
    (
      &[
        CW::Proportional(2),
        CW::Proportional(1),
        CW::Proportional(3),
      ],
      &[
        DCW::Percentage(100.0 / 3.0), // 33.333...
        DCW::Percentage(100.0 / 6.0), // 16.666...
        DCW::Percentage(50.0),
      ],
    ),
    (
      &[CW::Percentage(15), CW::Percentage(30), CW::Percentage(55)],
      &[
        DCW::Percentage(15.0),
        DCW::Percentage(30.0),
        DCW::Percentage(55.0),
      ],
    ),
    (
      &[CW::Percentage(20), CW::Auto, CW::Auto],
      &[DCW::Percentage(20.0), DCW::Auto(40.0), DCW::Auto(40.0)],
    ),
    (
      &[CW::Proportional(1), CW::Proportional(2), CW::Auto],
      &[DCW::Percentage(1.0), DCW::Percentage(2.0), DCW::Auto(97.0)],
    ),
    (
      &[CW::Proportional(25), CW::Auto, CW::Auto],
      &[DCW::Percentage(25.0), DCW::Auto(37.5), DCW::Auto(37.5)],
    ),
    (
      &[CW::Proportional(60), CW::Proportional(60), CW::Auto],
      &[DCW::Percentage(50.0), DCW::Percentage(50.0), DCW::Auto(0.0)],
    ),
    (
      &[CW::Percentage(60), CW::Percentage(60), CW::Percentage(60)],
      &[
        DCW::Percentage(100.0 / 3.0),
        DCW::Percentage(100.0 / 3.0),
        DCW::Percentage(100.0 / 3.0),
      ],
    ),
  ];
  let bump = Bump::new();
  for (input, expected) in cases.iter() {
    let col_widths = ColWidths::new(BumpVec::from_iter_in((*input).iter().copied(), &bump));
    let actual: Vec<_> = col_widths.distribute().collect();
    assert_eq!(actual, *expected);
  }
}
