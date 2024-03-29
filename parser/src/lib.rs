#![allow(dead_code)]

mod chunk;
mod contiguous_lines;
mod delimiter;
mod diagnostic;
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

pub mod prelude {
  pub use crate::parser::Parser;
  pub use bumpalo::Bump;
}

pub use diagnostic::Diagnostic;
pub use parser::Parser;

mod internal {
  pub use crate::chunk::*;
  pub use crate::contiguous_lines::ContiguousLines;
  pub use crate::delimiter::*;
  pub use crate::diagnostic::*;
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
  pub type Result<T> = std::result::Result<T, Diagnostic>;
}

pub mod variants {
  pub mod token {
    pub use crate::token::TokenKind::*;
  }
}

#[cfg(test)]
pub mod test;
