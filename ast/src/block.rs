use crate::internal::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Block<'arena> {
  pub meta: ChunkMeta<'arena>,
  pub content: BlockContent<'arena>,
  pub context: BlockContext,
  pub loc: MultiSourceLocation,
}

impl Block<'_> {
  pub fn is_comment(&self) -> bool {
    self.context == BlockContext::Comment
  }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum BlockContent<'arena> {
  Compound(BumpVec<'arena, Block<'arena>>),
  Simple(InlineNodes<'arena>),
  Verbatim,
  Raw,
  Empty(EmptyMetadata<'arena>),
  Table(Table<'arena>),
  Section(Section<'arena>),
  DocumentAttribute(String, AttrValue),
  QuotedParagraph {
    quote: InlineNodes<'arena>,
    attr: SourceString<'arena>,
    cite: Option<SourceString<'arena>>,
  },
  List {
    variant: ListVariant,
    depth: u8,
    items: BumpVec<'arena, ListItem<'arena>>,
  },
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum EmptyMetadata<'arena> {
  Image {
    target: SourceString<'arena>,
    attrs: AttrList<'arena>,
  },
  DiscreteHeading {
    level: u8,
    content: InlineNodes<'arena>,
    id: Option<BumpString<'arena>>,
  },
  Comment(SourceString<'arena>),
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

impl BlockContent<'_> {
  pub fn last_loc(&self) -> Option<SourceLocation> {
    match self {
      BlockContent::Compound(blocks) => blocks.last().and_then(|b| b.content.last_loc()),
      BlockContent::Simple(inline_nodes) => inline_nodes.last_loc(),
      BlockContent::Section(Section { heading, blocks, .. }) => {
        if !blocks.is_empty() {
          blocks.last().and_then(|b| b.content.last_loc())
        } else {
          heading.last_loc()
        }
      }
      BlockContent::Verbatim => todo!(),
      BlockContent::Raw => todo!(),
      BlockContent::Empty(_) => None,
      BlockContent::Table(_) => todo!(),
      BlockContent::DocumentAttribute(_, _) => None,
      BlockContent::QuotedParagraph { attr, cite, .. } => {
        cite.as_ref().map(|c| c.loc).or(Some(attr.loc))
      }
      BlockContent::List { items, .. } => items.last().and_then(|i| i.last_loc()),
    }
  }
}

impl BlockContext {
  pub const fn caption_attr_name(&self) -> Option<&'static str> {
    match self {
      BlockContext::Table => Some("table-caption"),
      BlockContext::Image => Some("figure-caption"),
      BlockContext::Example => Some("example-caption"),
      BlockContext::Listing => Some("listing-caption"),
      _ => None,
    }
  }
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
      "normal" => Some(BlockContext::Paragraph),
      _ => Self::derive_admonition(string),
    }
  }
}
