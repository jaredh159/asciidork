use std::collections::HashMap;

use crate::parse::inline::Inline;

// A document represents the top-level block element in AsciiDoc. It consists of an optional document header and either a) one or more sections preceded by an optional preamble or b) a sequence of top-level blocks only.
#[derive(Debug, PartialEq, Eq)]
pub struct Document {
  pub doctype: DocType,
  pub header: Option<DocHeader>,
  pub content: DocContent,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum DocType {
  Article,
  Book,
  ManPage,
  Inline,
}

type TodoType = Option<String>;

// If a document has a header, no content blocks are permitted above it. In other words, the document must start with a document header if it has one.
// https://docs.asciidoctor.org/asciidoc/latest/document/header/
#[derive(Debug, PartialEq, Eq)]
pub struct DocHeader {
  pub title: Option<DocTitle>,
  pub authors: Vec<Author>,
  pub revision: Option<Revision>,
  pub attrs: HashMap<String, String>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct DocTitle {
  pub heading: Vec<Inline>,
  pub subtitle: Option<Vec<Inline>>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Author {
  pub first_name: String,
  pub middle_name: Option<String>,
  pub last_name: String,
  pub email: Option<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Revision {
  number: String,
  date: String,
  remark: String,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Heading {
  // tod
}

#[derive(Debug, PartialEq, Eq)]
pub enum DocContent {
  Sectioned {
    preamble: Option<Block>,
    sections: Vec<Section>,
  },
  Blocks(Vec<Block>),
}

#[derive(Debug, PartialEq, Eq)]
pub struct SectionedDocumentContent {
  pub preamble: Option<Block>,
  pub sections: Vec<Section>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Section {
  pub level: u8,
  pub blocks: Vec<Block>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Context {
  Preamble,
  Section,
  Listing,
  Paragraph,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Block {
  Paragraph(ParagraphBlock),
  // Listing(ListingBlock),
  // Preamble(PreambleBlock),
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParagraphBlock {
  pub inlines: Vec<Inline>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Preamble {
  pub blocks: Vec<Block>,
}
