use crate::internal::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Block<'bmp> {
  pub title: Option<SourceString<'bmp>>,
  pub attrs: Option<AttrList<'bmp>>,
  pub content: BlockContent<'bmp>,
  pub context: BlockContext,
  pub loc: SourceLocation,
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
  QuotedParagraph {
    quote: InlineNodes<'bmp>,
    attr: SourceString<'bmp>,
    cite: Option<SourceString<'bmp>>,
  },
  List {
    variant: ListVariant,
    depth: u8,
    items: BumpVec<'bmp, ListItem<'bmp>>,
  },
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
  Section, // TODO: do i need this? we have a different node...
  Sidebar,
  Table,
  TableCell,
  ThematicBreak,
  TableOfContents,
  UnorderedList,
  Verse,
  Video,
}

impl<'bmp> BlockContent<'bmp> {
  pub fn last_loc(&self) -> Option<SourceLocation> {
    match self {
      BlockContent::Compound(blocks) => blocks.last().map(|b| b.loc),
      BlockContent::Simple(inline_nodes) => inline_nodes.last_loc(),
      BlockContent::Verbatim => todo!(),
      BlockContent::Raw => todo!(),
      BlockContent::Empty(_) => todo!(),
      BlockContent::Table => todo!(),
      BlockContent::DocumentAttribute(_, _) => None,
      BlockContent::QuotedParagraph { attr, cite, .. } => {
        cite.as_ref().map(|c| c.loc).or(Some(attr.loc))
      }
      BlockContent::List { items, .. } => items.last().and_then(|i| i.last_loc()),
    }
  }
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
      "listing" => Some(BlockContext::Listing),
      "literal" => Some(BlockContext::Literal),
      _ => Self::derive_admonition(string),
    }
  }
}
