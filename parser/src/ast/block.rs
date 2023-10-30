use bumpalo::collections::Vec;

use super::{node::Section, Inline};

#[derive(Debug, PartialEq, Eq)]
pub struct Block<'bmp> {
  pub context: BlockContext<'bmp>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum BlockContext<'bmp> {
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
  Paragraph(Vec<'bmp, Inline<'bmp>>),
  Passthrough,
  BlockQuote,
  Section(Section<'bmp>),
  Sidebar,
  Table,
  TableCell,
  ThematicBreak,
  TableOfContents,
  UnorderedList,
  Verse,
  Video,
}
