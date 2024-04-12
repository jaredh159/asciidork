use crate::internal::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Block<'bmp> {
  pub meta: ChunkMeta<'bmp>,
  pub content: BlockContent<'bmp>,
  pub context: BlockContext,
  pub loc: SourceLocation,
}

impl<'bmp> Block<'bmp> {
  pub fn has_attr_option(&self, name: &str) -> bool {
    self
      .meta
      .attrs
      .as_ref()
      .map_or(false, |attrs| attrs.has_option(name))
  }

  pub fn named_attr(&self, name: &str) -> Option<&str> {
    self.meta.attrs.as_ref().and_then(|attrs| attrs.named(name))
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
  Section(Section<'bmp>),
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
  DiscreteHeading {
    level: u8,
    content: InlineNodes<'bmp>,
    id: Option<BumpString<'bmp>>,
  },
  None,
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

impl<'bmp> BlockContent<'bmp> {
  pub fn last_loc(&self) -> Option<SourceLocation> {
    match self {
      BlockContent::Compound(blocks) => blocks.last().map(|b| b.loc),
      BlockContent::Simple(inline_nodes) => inline_nodes.last_loc(),
      BlockContent::Section(Section { heading, blocks, .. }) => {
        if !blocks.is_empty() {
          blocks.last().map(|b| b.loc)
        } else {
          heading.last_loc()
        }
      }
      BlockContent::Verbatim => todo!(),
      BlockContent::Raw => todo!(),
      BlockContent::Empty(_) => None,
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
      "listing" | "source" => Some(BlockContext::Listing),
      "literal" => Some(BlockContext::Literal),
      "pass" => Some(BlockContext::Passthrough),
      "comment" => Some(BlockContext::Comment),
      "verse" => Some(BlockContext::Verse),
      "example" => Some(BlockContext::Example),
      _ => Self::derive_admonition(string),
    }
  }
}
