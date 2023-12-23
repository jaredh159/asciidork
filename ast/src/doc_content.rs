use crate::internal::*;

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
      BlockContext::Section => {
        self.ensure_sectioned();
        todo!("push_block: section")
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
