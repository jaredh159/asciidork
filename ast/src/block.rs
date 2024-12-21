use crate::internal::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Block<'arena> {
  pub meta: ChunkMeta<'arena>,
  pub content: BlockContent<'arena>,
  pub context: BlockContext,
}

impl Block<'_> {
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

// json

impl Json for Block<'_> {
  fn to_json_in(&self, buf: &mut JsonBuf) {
    buf.begin_obj("Block");
    if !self.meta.is_empty() {
      buf.add_member("meta", &self.meta);
    }
    buf.push_str(r#","context":""#);
    buf.push_simple_variant(self.context);
    buf.push('"');
    if self.content != BlockContent::Empty(EmptyMetadata::None) {
      buf.add_member("content", &self.content);
    }
    buf.push('}');
  }
}

impl Json for BlockContext {
  fn to_json_in(&self, buf: &mut JsonBuf) {
    buf.push_obj_enum_type("BlockContext", self);
  }
}

impl Json for EmptyMetadata<'_> {
  fn to_json_in(&self, buf: &mut JsonBuf) {
    buf.begin_obj("EmptyMetadata");
    buf.push_str(r#","variant":""#);
    match self {
      EmptyMetadata::Image { target, attrs } => {
        buf.push_str("Image\"");
        buf.add_member("target", target);
        if !attrs.is_empty() {
          buf.add_member("attrs", attrs);
        }
      }
      EmptyMetadata::DiscreteHeading { level, content, id } => {
        buf.push_str("DiscreteHeading\"");
        buf.add_member("level", level);
        buf.add_option_member("id", id.as_ref());
        buf.add_member("content", content);
      }
      EmptyMetadata::None => buf.push_str("None\""),
    }
    buf.finish_obj();
  }
}

impl Json for BlockContent<'_> {
  fn to_json_in(&self, buf: &mut JsonBuf) {
    buf.begin_obj("BlockContent");
    buf.push_str(r#","variant":""#);
    match self {
      BlockContent::Compound(blocks) => {
        buf.push_str("Compound\"");
        buf.add_member("children", blocks);
      }
      BlockContent::Simple(nodes) => {
        buf.push_str("Simple\"");
        buf.add_member("children", nodes);
      }
      BlockContent::Verbatim => todo!(),
      BlockContent::Raw => todo!(),
      BlockContent::Empty(meta) => {
        buf.push_str("Empty\"");
        if *meta != EmptyMetadata::None {
          buf.add_member("meta", meta);
        }
      }
      BlockContent::Table(_) => todo!(),
      BlockContent::Section(section) => {
        buf.push_str("Section\"");
        buf.add_member("section", section);
      }
      BlockContent::DocumentAttribute(key, value) => {
        buf.push_str("DocumentAttribute\"");
        buf.add_member("key", &key.as_str());
        buf.add_member("value", &value.str());
      }
      BlockContent::QuotedParagraph { quote, attr, cite } => {
        buf.push_str("QuotedParagraph\"");
        buf.add_member("quote", quote);
        buf.add_member("attr", attr);
        buf.add_option_member("cite", cite.as_ref());
      }
      BlockContent::List { variant, depth, items } => {
        buf.push_str("List\"");
        buf.add_member("list_variant", variant);
        buf.add_member("depth", depth);
        buf.add_member("items", items);
      }
    }
    buf.finish_obj();
  }
}
