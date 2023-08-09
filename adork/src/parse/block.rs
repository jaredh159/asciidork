use super::Result;
use crate::parse::Parser;
use crate::tok;

impl Parser {
  pub(super) fn parse_block(&self, _block: tok::Block) -> Result<()> {
    todo!("parse block")
  }
}

// tests

#[cfg(test)]
mod tests {
  // use super::*;
  // use crate::t::*;

  #[test]
  fn test_read_line() {
    //
  }
}
