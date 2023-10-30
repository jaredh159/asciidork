use bumpalo::collections::Vec;

use super::block::{Block, BlockContext};
use super::node::Section;

#[derive(Debug, PartialEq, Eq)]
pub enum DocContent<'bmp> {
  Sectioned {
    preamble: Option<Vec<'bmp, Block<'bmp>>>,
    sections: Vec<'bmp, Section<'bmp>>,
  },
  Blocks(Vec<'bmp, Block<'bmp>>),
}

impl<'bmp> DocContent<'bmp> {
  pub fn push_block(&mut self, block: Block<'bmp>) {
    match block.context {
      BlockContext::Section(section) => {
        self.ensure_sectioned();
        match self {
          DocContent::Sectioned { sections, .. } => sections.push(section),
          _ => unreachable!(),
        }
      }
      _ => match self {
        DocContent::Blocks(blocks) => blocks.push(block),
        _ => unreachable!(),
      },
    }
  }

  pub fn is_sectioned(&self) -> bool {
    matches!(self, DocContent::Sectioned { .. })
  }

  pub fn ensure_sectioned(&mut self) {
    todo!("ensure_sectioned")
  }
}
