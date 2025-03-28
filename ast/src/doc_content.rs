use crate::internal::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum DocContent<'arena> {
  Parts(BumpVec<'arena, Part<'arena>>),
  Sections(Sectioned<'arena>),
  Blocks(BumpVec<'arena, Block<'arena>>),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Sectioned<'arena> {
  pub preamble: Option<BumpVec<'arena, Block<'arena>>>,
  pub sections: BumpVec<'arena, Section<'arena>>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Part<'arena> {
  pub title: PartTitle<'arena>,
  pub intro: Option<BumpVec<'arena, Block<'arena>>>,
  pub sections: BumpVec<'arena, Section<'arena>>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct PartTitle<'arena> {
  pub id: Option<BumpString<'arena>>,
  pub attrs: MultiAttrList<'arena>,
  pub text: InlineNodes<'arena>,
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
      DocContent::Sections(Sectioned { sections, .. }) => sections.push(section),
      DocContent::Parts(_parts) => todo!("push section into book parts"),
      DocContent::Blocks(blocks) => {
        eprintln!("push_section: converting blocks to sections");
        let preamble = if blocks.is_empty() || blocks.iter().all(|b| b.is_comment()) {
          None
        } else {
          Some(std::mem::replace(blocks, BumpVec::new_in(bump)))
        };
        let sections = bvec![in bump; section];
        *self = DocContent::Sections(Sectioned { preamble, sections });
      }
    }
  }

  pub const fn is_sectioned(&self) -> bool {
    matches!(self, DocContent::Sections(_))
  }

  pub const fn blocks(&self) -> Option<&BumpVec<'arena, Block<'arena>>> {
    match self {
      DocContent::Blocks(blocks) => Some(blocks),
      _ => None,
    }
  }

  pub fn last_loc(&self) -> Option<SourceLocation> {
    match self {
      DocContent::Sections(Sectioned { sections, .. }) => sections
        .last()
        .and_then(|section| section.blocks.last().and_then(|b| b.content.last_loc())),
      DocContent::Blocks(blocks) => blocks.last().and_then(|b| b.content.last_loc()),
      DocContent::Parts(parts) => parts.last().and_then(|sectioned| {
        sectioned
          .sections
          .last()
          .and_then(|section| section.blocks.last().and_then(|b| b.content.last_loc()))
      }),
    }
  }
}
