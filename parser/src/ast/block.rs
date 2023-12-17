use bumpalo::collections::Vec;

use super::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Block<'bmp> {
  pub context: BlockContext<'bmp>,
  pub loc: SourceLocation,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum BlockContext<'bmp> {
  Admonition,
  Audio,
  CalloutList,
  DescriptionList,
  DiscreteHeading,
  Example,
  Image {
    target: SourceString<'bmp>,
    attrs: AttrList<'bmp>,
  },
  ListItem,
  Listing,
  Literal,
  OrderedList,
  Open(Vec<'bmp, Block<'bmp>>),
  PageBreak,
  Paragraph(Vec<'bmp, InlineNode<'bmp>>),
  Passthrough,
  BlockQuote,
  Section(Section<'bmp>),
  Sidebar(Vec<'bmp, Block<'bmp>>),
  Table,
  TableCell,
  ThematicBreak,
  TableOfContents,
  UnorderedList,
  Verse,
  Video,
}
