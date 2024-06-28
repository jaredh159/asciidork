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
      let preamble = if blocks.is_empty() {
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
        .and_then(|section| section.blocks.last().map(|block| block.loc)),
      DocContent::Blocks(blocks) => blocks.last().map(|block| block.loc),
    }
  }
}

impl Json for DocContent<'_> {
  fn to_json_in(&self, buf: &mut JsonBuf) {
    buf.begin_obj("DocContent");
    buf.push_str(r#","variant":""#);
    match self {
      DocContent::Sectioned { preamble, sections } => {
        buf.push_str("Sectioned\"");
        buf.add_option_member("preamble", preamble.as_ref());
        buf.add_member("sections", sections);
      }
      DocContent::Blocks(blocks) => {
        buf.push_str("Blocks\"");
        buf.add_member("blocks", blocks);
      }
    }
    buf.finish_obj();
  }
}
