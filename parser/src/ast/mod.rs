mod attr_list;
mod block;
mod doc_content;
mod doc_header;
mod inline;
mod r#macro;
mod node;
mod source_location;
mod source_string;

pub use attr_list::{AttrList, Named};
pub use block::*;
pub use doc_content::DocContent;
pub use doc_header::{Author, DocHeader, DocTitle, Revision};
pub use inline::{CurlyKind, Inline, InlineNode, QuoteKind, SpecialCharKind};
pub use inline::{CurlyKind::*, Inline::*, QuoteKind::*};
pub use node::{Document, Section};
pub use r#macro::{Macro::*, *};
pub use source_location::SourceLocation;
pub use source_string::SourceString;

pub mod short {
  pub mod block {
    pub use crate::ast::block::BlockContent as Content;
    pub use crate::ast::block::BlockContext as Context;
  }
}
