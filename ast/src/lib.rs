mod attr_list;
mod block;
mod doc_content;
mod doc_header;
mod inline;
mod r#macro;
mod node;
mod source_location;
mod source_string;

pub use internal::types::*;

mod internal {
  pub(crate) mod types {
    pub use crate::attr_list::{AttrList, Named};
    pub use crate::block::{Block, BlockContent, BlockContext, EmptyMetadata};
    pub use crate::doc_content::DocContent;
    pub use crate::doc_header::{AttrEntries, AttrEntry, Author, DocHeader, DocTitle, Revision};
    pub use crate::inline::{CurlyKind, Inline, InlineNode, QuoteKind, SpecialCharKind};
    pub use crate::inline::{CurlyKind::*, QuoteKind::*};
    pub use crate::node::{Document, Section};
    pub use crate::r#macro::{Flow, MacroNode, UrlScheme};
    pub use crate::source_location::SourceLocation;
    pub use crate::source_string::SourceString;
  }

  pub use types::*;

  // bump helpers
  pub use bumpalo::collections::String;
  pub use bumpalo::collections::Vec;
  pub use bumpalo::vec as bvec;
  pub use bumpalo::Bump;
  pub use std::string::String as StdString;
}

pub mod prelude {
  pub use crate::attr_list::{AttrList, Named};
  pub use crate::block::{Block, EmptyMetadata};
  pub use crate::doc_content::DocContent;
  pub use crate::doc_header::{AttrEntries, AttrEntry};
  pub use crate::inline::{InlineNode, SpecialCharKind};
  pub use crate::node::{Document, Section};
  pub use crate::source_location::SourceLocation;
  pub use crate::source_string::SourceString;
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
