use crate::internal::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Block<'bmp> {
  pub title: Option<SourceString<'bmp>>,
  pub attrs: Option<AttrList<'bmp>>,
  pub content: BlockContent<'bmp>,
  pub context: BlockContext,
  pub loc: SourceLocation,
}

impl<'bmp> Block<'bmp> {
  pub fn empty(b: &'bmp Bump) -> Self {
    Block {
      title: None,
      attrs: None,
      context: BlockContext::Paragraph,
      content: BlockContent::Simple(InlineNodes::new(b)),
      loc: SourceLocation::new(0, 0),
    }
  }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum BlockContent<'bmp> {
  Compound(BumpVec<'bmp, Block<'bmp>>),
  Simple(InlineNodes<'bmp>),
  Verbatim,
  Raw,
  Empty(EmptyMetadata<'bmp>),
  Table,
  DocumentAttribute(String, AttrEntry),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum EmptyMetadata<'bmp> {
  Image {
    target: SourceString<'bmp>,
    attrs: AttrList<'bmp>,
  },
}

#[derive(Copy, Debug, PartialEq, Eq, Clone)]
pub enum BlockContext {
  AdmonitionCaution,
  AdmonitionImportant,
  AdmonitionNote,
  AdmonitionTip,
  AdmonitionWarning,
  Audio,
  CalloutList,
  DescriptionList,
  DiscreteHeading,
  DocumentAttributeDecl,
  Example,
  Image,
  ListItem,
  Listing,
  Literal,
  OrderedList,
  Open,
  PageBreak,
  Paragraph,
  Passthrough,
  BlockQuote,
  Section,
  Sidebar,
  Table,
  TableCell,
  ThematicBreak,
  TableOfContents,
  UnorderedList,
  Verse,
  Video,
}

impl BlockContext {
  pub fn derive_admonition(string: &str) -> Option<Self> {
    match string {
      "CAUTION" => Some(BlockContext::AdmonitionCaution),
      "IMPORTANT" => Some(BlockContext::AdmonitionImportant),
      "NOTE" => Some(BlockContext::AdmonitionNote),
      "TIP" => Some(BlockContext::AdmonitionTip),
      "WARNING" => Some(BlockContext::AdmonitionWarning),
      _ => None,
    }
  }

  pub fn derive(string: &str) -> Option<Self> {
    match string {
      "sidebar" => Some(BlockContext::Sidebar),
      _ => Self::derive_admonition(string),
    }
  }
}
