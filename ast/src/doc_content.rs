use crate::internal::*;

#[derive(Debug, PartialEq, Eq)]
pub enum DocContent<'bmp> {
  Sectioned {
    preamble: Option<BumpVec<'bmp, Block<'bmp>>>,
    sections: BumpVec<'bmp, Section<'bmp>>,
  },
  Blocks(BumpVec<'bmp, Block<'bmp>>),
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

  pub fn push_section(&mut self, section: Section<'bmp>) {
    match self {
      DocContent::Sectioned { sections, .. } => sections.push(section),
      _ => unreachable!(),
    }
  }

  pub fn is_sectioned(&self) -> bool {
    matches!(self, DocContent::Sectioned { .. })
  }

  pub fn ensure_sectioned(&mut self) {
    todo!("ensure_sectioned")
  }
}
