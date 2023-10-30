use bumpalo::collections::Vec;

use super::{node::Section, Inline};

#[derive(Debug, PartialEq, Eq)]
pub struct Block<'alloc> {
  pub context: BlockContext<'alloc>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum BlockContext<'alloc> {
  Admonition,
  Audio,
  CalloutList,
  DescriptionList,
  DiscreteHeading,
  Example,
  Image,
  ListItem,
  Listing,
  Literal,
  OrderedList,
  Open,
  PageBreak,
  Paragraph(Vec<'alloc, Inline<'alloc>>),
  Passthrough,
  BlockQuote,
  Section(Section<'alloc>),
  Sidebar,
  Table,
  TableCell,
  ThematicBreak,
  TableOfContents,
  UnorderedList,
  Verse,
  Video,
}
