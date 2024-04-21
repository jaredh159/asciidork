mod attr_entries;
mod attr_list;
mod block;
mod chunk_meta;
mod doc_content;
mod doc_header;
mod inline;
mod inline_nodes;
mod json;
mod list;
mod r#macro;
mod node;
mod source_location;
mod source_string;
mod toc;

pub use internal::types::*;

mod internal {
  pub(crate) mod types {
    pub use crate::attr_entries::{AttrEntries, AttrEntry};
    pub use crate::attr_list::{AttrList, Named};
    pub use crate::block::{Block, BlockContent, BlockContext, EmptyMetadata};
    pub use crate::chunk_meta::ChunkMeta;
    pub use crate::doc_content::DocContent;
    pub use crate::doc_header::{Author, DocHeader, DocTitle, Revision};
    pub use crate::inline::{CurlyKind, Inline, InlineNode, QuoteKind, SpecialCharKind};
    pub use crate::inline::{CurlyKind::*, QuoteKind::*};
    pub use crate::inline_nodes::InlineNodes;
    pub use crate::json::*;
    pub use crate::list::*;
    pub use crate::node::{Callout, Document, Ref, Section};
    pub use crate::r#macro::{Flow, MacroNode, UrlScheme};
    pub use crate::source_location::SourceLocation;
    pub use crate::source_string::SourceString;
    pub use crate::toc::*;
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
  pub use crate::attr_entries::{AttrEntries, AttrEntry};
  pub use crate::attr_list::{AttrList, Named};
  pub use crate::block::{Block, BlockContent, BlockContext, EmptyMetadata};
  pub use crate::chunk_meta::ChunkMeta;
  pub use crate::doc_content::DocContent;
  pub use crate::doc_header::{Author, DocHeader, DocTitle};
  pub use crate::inline::{CurlyKind, Inline, InlineNode, QuoteKind, SpecialCharKind};
  pub use crate::json::*;
  pub use crate::list::{ListItem, ListItemTypeMeta, ListMarker, ListVariant};
  pub use crate::node::{Callout, Document, Ref, Section};
  pub use crate::source_location::SourceLocation;
  pub use crate::source_string::SourceString;
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
