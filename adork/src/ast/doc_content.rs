use super::{Block, Section};

#[derive(Debug, PartialEq, Eq)]
pub enum DocContent {
  Sectioned {
    preamble: Option<Vec<Block>>,
    sections: Vec<Section>,
  },
  Blocks(Vec<Block>),
}

impl DocContent {
  pub fn is_sectioned(&self) -> bool {
    match self {
      DocContent::Sectioned { .. } => true,
      _ => false,
    }
  }

  pub fn ensure_sectioned(&mut self) {
    if !self.is_sectioned() {
      *self = DocContent::Sectioned {
        preamble: match self {
          DocContent::Blocks(blocks) => match blocks.is_empty() {
            true => None,
            false => Some(blocks.drain(..).collect()),
          },
          _ => None,
        },
        sections: vec![],
      };
    }
  }
}
