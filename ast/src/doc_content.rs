use crate::internal::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum DocContent<'arena> {
  Sectioned {
    preamble: Option<BumpVec<'arena, Block<'arena>>>,
    sections: BumpVec<'arena, Section<'arena>>,
  },
  Blocks(BumpVec<'arena, Block<'arena>>),
}

impl<'arena> DocContent<'arena> {
  pub fn push_block(&mut self, block: Block<'arena>, _bump: &'arena Bump) {
    match self {
      DocContent::Blocks(blocks) => blocks.push(block),
      _ => unreachable!("DocContent::push_block"),
    }
  }

  pub fn push_section(&mut self, section: Section<'arena>, bump: &'arena Bump) {
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

  pub fn ensure_sectioned(&mut self, bump: &'arena Bump) {
    if let DocContent::Blocks(blocks) = self {
      let preamble = if blocks.is_empty() || blocks.iter().all(|b| b.is_comment()) {
        None
      } else {
        Some(std::mem::replace(blocks, BumpVec::new_in(bump)))
      };
      let sections = BumpVec::with_capacity_in(1, bump);
      *self = DocContent::Sectioned { preamble, sections };
    }
  }

  pub const fn blocks(&self) -> Option<&BumpVec<'arena, Block<'arena>>> {
    match self {
      DocContent::Blocks(blocks) => Some(blocks),
      _ => None,
    }
  }

  pub fn last_loc(&self) -> Option<SourceLocation> {
    match self {
      DocContent::Sectioned { sections, .. } => sections
        .last()
        .and_then(|section| section.blocks.last().and_then(|b| b.content.last_loc())),
      DocContent::Blocks(blocks) => blocks.last().and_then(|b| b.content.last_loc()),
    }
  }
}
