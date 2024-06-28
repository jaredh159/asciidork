use std::ops::{Deref, DerefMut};

use crate::internal::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ColWidths<'arena>(BumpVec<'arena, ColWidth>);

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

impl<'arena> From<BumpVec<'arena, ColWidth>> for ColWidths<'arena> {
  fn from(col_widths: BumpVec<'arena, ColWidth>) -> Self {
    Self(col_widths)
  }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum DistributedColWidth {
  Percentage(f32),
  Auto(f32),
}

impl<'arena> ColWidths<'arena> {
  pub fn new(col_widths: BumpVec<'arena, ColWidth>) -> Self {
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
