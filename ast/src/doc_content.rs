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
  pub fn push_block(&mut self, block: Block<'bmp>, _bump: &'bmp Bump) {
    match self {
      DocContent::Blocks(blocks) => blocks.push(block),
      _ => todo!("¯\\_(ツ)_/¯ not sure what to do here..."),
    }
  }

  pub fn push_section(&mut self, section: Section<'bmp>, bump: &'bmp Bump) {
    match self {
      DocContent::Sectioned { sections, .. } => sections.push(section),
      _ => {
        self.ensure_sectioned(bump);
        self.push_section(section, bump);
      }
    }
  }

  pub const fn is_sectioned(&self) -> bool {
    matches!(self, DocContent::Sectioned { .. })
  }

  pub fn ensure_sectioned(&mut self, bump: &'bmp Bump) {
    if let DocContent::Blocks(blocks) = self {
      let preamble = std::mem::replace(blocks, BumpVec::new_in(bump));
      let sections = BumpVec::with_capacity_in(1, bump);
      *self = DocContent::Sectioned { preamble: Some(preamble), sections };
    }
  }

  pub const fn blocks(&self) -> Option<&BumpVec<'bmp, Block<'bmp>>> {
    match self {
      DocContent::Blocks(blocks) => Some(blocks),
      _ => None,
    }
  }
}
