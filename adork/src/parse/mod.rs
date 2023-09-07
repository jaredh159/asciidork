mod attr_list;
mod author;
mod block;
pub mod diagnostic;
mod doc_attrs;
mod doc_header;
mod inline;
mod parser;
mod revision_line;
mod utils;

pub use parser::Parser;

type Result<T> = std::result::Result<T, diagnostic::Diagnostic>;
