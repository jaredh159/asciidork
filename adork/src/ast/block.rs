use super::{Inline, Section};

#[derive(Debug, PartialEq, Eq)]
pub struct Block {
  pub context: BlockContext,
}

#[derive(Debug, PartialEq, Eq)]
pub enum BlockContext {
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
  Paragraph(Vec<Inline>),
  Passthrough,
  BlockQuote,
  Section(Section),
  Sidebar,
  Table,
  TableCell,
  ThematicBreak,
  TableOfContents,
  UnorderedList,
  Verse,
  Video,
}
