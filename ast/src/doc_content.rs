use crate::internal::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum DocContent<'arena> {
  Parts(MultiPartBook<'arena>),
  Sections(Sectioned<'arena>),
  Blocks(BumpVec<'arena, Block<'arena>>),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Sectioned<'arena> {
  pub preamble: Option<BumpVec<'arena, Block<'arena>>>,
  pub sections: BumpVec<'arena, Section<'arena>>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct MultiPartBook<'arena> {
  pub preamble: Option<BumpVec<'arena, Block<'arena>>>,
  pub opening_special_sects: BumpVec<'arena, Section<'arena>>,
  pub parts: BumpVec<'arena, Part<'arena>>,
  pub closing_special_sects: BumpVec<'arena, Section<'arena>>,
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
  pub meta: ChunkMeta<'arena>,
  pub text: InlineNodes<'arena>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Section<'arena> {
  pub meta: ChunkMeta<'arena>,
  pub level: u8,
  pub id: Option<BumpString<'arena>>,
  pub heading: InlineNodes<'arena>,
  pub blocks: BumpVec<'arena, Block<'arena>>,
  pub loc: MultiSourceLocation,
}

impl<'arena> Sectioned<'arena> {
  pub fn into_doc_content(self, bump: &'arena Bump) -> DocContent<'arena> {
    if self.sections.is_empty() {
      DocContent::Blocks(self.preamble.unwrap_or(bvec![in bump]))
    } else {
      DocContent::Sections(self)
    }
  }
}

impl<'arena> DocContent<'arena> {
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
      DocContent::Parts(book) => book
        .closing_special_sects
        .last()
        .and_then(|sect| sect.blocks.last().and_then(|b| b.content.last_loc()))
        .or_else(|| {
          book.parts.last().and_then(|sectioned| {
            sectioned
              .sections
              .last()
              .and_then(|sect| sect.blocks.last().and_then(|b| b.content.last_loc()))
          })
        }),
    }
  }
}
