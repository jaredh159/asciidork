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

// variant: ordered | unordered
// principle: InlineNodes
// list has items: Vec<ListItem>
// list item has `marker`, like `*` (asg shows list having it too...)

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum BlockContent<'bmp> {
  Compound(BumpVec<'bmp, Block<'bmp>>),
  Simple(InlineNodes<'bmp>),
  Verbatim,
  Raw,
  Empty(EmptyMetadata<'bmp>),
  Table,
  DocumentAttribute(String, AttrEntry),
  QuotedParagraph {
    quote: InlineNodes<'bmp>,
    attr: SourceString<'bmp>,
    cite: Option<SourceString<'bmp>>,
  },
  List {
    variant: ListVariant,
    items: BumpVec<'bmp, ListItem<'bmp>>,
  },
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ListItem<'bmp> {
  pub marker: SourceString<'bmp>,
  pub principle: InlineNodes<'bmp>,
  pub blocks: BumpVec<'bmp, Block<'bmp>>,
}

impl<'bmp> ListItem<'bmp> {
  pub fn loc_start(&self) -> usize {
    self.marker.loc.start
  }
  pub fn loc_end(&self) -> Option<usize> {
    self.principle.last_loc_end()
  }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ListVariant {
  Ordered,
  Unordered,
}

impl ListVariant {
  pub fn to_context(&self) -> BlockContext {
    match self {
      ListVariant::Ordered => BlockContext::OrderedList,
      ListVariant::Unordered => BlockContext::UnorderedList,
    }
  }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum EmptyMetadata<'bmp> {
  Image {
    target: SourceString<'bmp>,
    attrs: AttrList<'bmp>,
  },
  None, // weird..., or good?
}

#[derive(Copy, Debug, PartialEq, Eq, Clone)]
pub enum BlockContext {
  AdmonitionCaution,
  AdmonitionImportant,
  AdmonitionNote,
  AdmonitionTip,
  AdmonitionWarning,
  Audio,
  BlockQuote,
  CalloutList,
  Comment,
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
  QuotedParagraph,
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
      "quote" => Some(BlockContext::BlockQuote),
      _ => Self::derive_admonition(string),
    }
  }
}
