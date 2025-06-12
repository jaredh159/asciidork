mod attr_list;
mod block;
mod chunk_meta;
mod col_widths;
mod doc_content;
mod document;
mod inline;
mod inline_nodes;
mod list;
mod r#macro;
mod multi_attr_list;
mod multi_source_location;
mod node;
mod source_location;
mod source_string;
mod table;
mod toc;

pub use internal::types::*;

mod internal {
  pub(crate) mod types {
    pub use crate::attr_list::{AttrData, AttrList, Named};
    pub use crate::block::{Block, BlockContent, BlockContext, EmptyMetadata};
    pub use crate::chunk_meta::ChunkMeta;
    pub use crate::col_widths::*;
    pub use crate::doc_content::*;
    pub use crate::document::{DocHeader, DocTitle, Document};
    pub use crate::inline::{AdjacentNewline, CurlyKind::*, QuoteKind::*, SymbolKind};
    pub use crate::inline::{CurlyKind, Inline, InlineNode, QuoteKind, SpecialCharKind};
    pub use crate::inline_nodes::InlineNodes;
    pub use crate::list::*;
    pub use crate::multi_attr_list::{MultiAttrList, NoAttrs};
    pub use crate::multi_source_location::MultiSourceLocation;
    pub use crate::node::{Anchor, Callout};
    pub use crate::r#macro::{Flow, MacroNode, PluginMacro, UrlScheme, XrefKind};
    pub use crate::source_location::SourceLocation;
    pub use crate::source_string::SourceString;
    pub use crate::table::*;
    pub use crate::toc::*;
    pub use asciidork_core::{AttrValue, DocumentMeta, ReadAttr, SpecialSection};
    pub use smallvec::SmallVec;
  }

  pub use types::*;

  // bump helpers
  pub use bumpalo::collections::String as BumpString;
  pub use bumpalo::collections::Vec as BumpVec;
  pub use bumpalo::vec as bvec;
  pub use bumpalo::Bump;
}

pub mod prelude {
  pub use crate::attr_list::{AttrData, AttrList, Named};
  pub use crate::block::{Block, BlockContent, BlockContext, EmptyMetadata};
  pub use crate::chunk_meta::ChunkMeta;
  pub use crate::col_widths::*;
  pub use crate::doc_content::*;
  pub use crate::document::{DocHeader, DocTitle, Document};
  pub use crate::inline::{CurlyKind, Inline, InlineNode, QuoteKind, SpecialCharKind, SymbolKind};
  pub use crate::list::{ListItem, ListItemTypeMeta, ListMarker, ListVariant};
  pub use crate::multi_attr_list::{MultiAttrList, NoAttrs};
  pub use crate::multi_source_location::MultiSourceLocation;
  pub use crate::node::{Anchor, Callout};
  pub use crate::r#macro::{PluginMacro, UrlScheme, XrefKind};
  pub use crate::source_location::SourceLocation;
  pub use crate::source_string::SourceString;
  pub use crate::table::*;
  pub use crate::toc::*;
}

pub mod short {
  pub mod block {
    pub use crate::block::BlockContent as Content;
    pub use crate::block::BlockContext as Context;
  }
}

pub mod variants {
  pub mod inline {
    pub use crate::inline::Inline::*;
  }
  pub mod r#macro {
    pub use crate::r#macro::MacroNode::*;
  }
}
