use crate::internal::*;

impl<'bmp, 'src> Parser<'bmp, 'src> {
  pub(super) fn parse_col_specs(&mut self, cols_attr: &str) -> BumpVec<'bmp, ColSpec> {
    let bump = self.bump;
    BumpVec::from_iter_in(
      cols_attr.split(',').map(|col| self.parse_col_spec(col)),
      bump,
    )
  }

  fn parse_col_spec(&mut self, col_attr: &str) -> ColSpec {
    ColSpec { width: col_attr.parse().unwrap_or(1) }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use test_utils::{assert_eq, *};

  #[test]
  fn test_parse_col_specs() {
    let cases: &[(&str, &[ColSpec])] = &[
      ("1", &[ColSpec { width: 1 }]),
      ("1,2", &[ColSpec { width: 1 }, ColSpec { width: 2 }]),
    ];
    let mut parser = Parser::new(leaked_bump(), "");
    for (input, expected) in cases {
      let cols = parser.parse_col_specs(input);
      assert_eq!(cols, *expected);
    }
  }
}
