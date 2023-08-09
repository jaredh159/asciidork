use super::{Block, DocContent, DocHeader, Inline};

#[derive(Debug, PartialEq, Eq)]
pub struct Section {
  level: u8,
  heading: Heading,
  blocks: Vec<Block>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Heading {
  inlines: Vec<Inline>,
}

// https://docs.asciidoctor.org/asciidoc/latest/key-concepts/#document
#[derive(Debug, PartialEq, Eq)]
pub struct Document {
  pub header: Option<DocHeader>,
  pub content: DocContent,
}
