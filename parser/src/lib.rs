#![allow(dead_code)]

mod chunk;
mod contiguous_lines;
mod delimiter;
mod deq;
mod diagnostic;
pub mod include_resolver;
mod lexer;
mod line;
mod list_stack;
mod parse_context;
pub mod parser;
mod substitutions;
mod tasks;
mod token;
mod utils;

extern crate asciidork_ast as ast;
extern crate asciidork_meta as meta;

pub mod prelude {
  pub use crate::diagnostic::{Diagnostic, DiagnosticColor};
  pub use crate::parser::Parser;
  pub use asciidork_ast::Json;
  pub use bumpalo::Bump;
}

pub use diagnostic::{Diagnostic, DiagnosticColor};
pub use parser::Parser;

mod internal {
  pub use crate::chunk::*;
  pub use crate::contiguous_lines::ContiguousLines;
  pub use crate::delimiter::*;
  pub use crate::deq::*;
  pub use crate::diagnostic::*;
  pub use crate::include_resolver::*;
  pub use crate::lexer::*;
  pub use crate::line::*;
  pub use crate::list_stack::*;
  pub use crate::parse_context::*;
  pub use crate::parser::*;
  pub use crate::substitutions::*;
  pub use crate::tasks::collect_text::*;
  pub use crate::tasks::customize_subs;
  pub use crate::token::*;
  pub use crate::utils::bump::*;
  pub use ast::*;
  pub use meta::{Author, DocType, JobSettings, ReadAttr};
  pub use smallvec::SmallVec;
  pub type Result<T> = std::result::Result<T, Diagnostic>;
}

pub mod variants {
  pub mod token {
    pub use crate::token::TokenKind::*;
  }
}
