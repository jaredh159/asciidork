#![allow(dead_code)]

mod contiguous_lines;
mod delimiter;
mod diagnostic;
mod lexer;
mod line;
mod list_stack;
pub mod parser;
mod tasks;
mod token;
mod utils;

extern crate asciidork_ast as ast;

pub mod prelude {
  pub use crate::parser::Parser;
  pub use bumpalo::Bump;
}

pub use diagnostic::Diagnostic;
pub use parser::Parser;

mod internal {
  pub use crate::contiguous_lines::ContiguousLines;
  pub use crate::delimiter::*;
  pub use crate::diagnostic::*;
  pub use crate::lexer::*;
  pub use crate::line::*;
  pub use crate::list_stack::*;
  pub use crate::parser::*;
  pub use crate::tasks::block_metadata::*;
  pub use crate::tasks::collect_text::*;
  pub use crate::tasks::parse_section::*;
  pub use crate::token::*;
  pub use crate::utils::bump::*;
  pub use ast::*;
  pub type Result<T> = std::result::Result<T, Diagnostic>;
}

pub mod variants {
  pub mod token {
    pub use crate::token::TokenKind::*;
  }
}

#[cfg(test)]
pub mod test;
