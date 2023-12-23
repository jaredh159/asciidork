#![allow(dead_code)]

mod contiguous_lines;
mod delimiter;
mod diagnostic;
mod lexer;
mod line;
pub mod parser;
mod tasks;
mod token;
mod utils;

#[cfg(test)]
pub mod test;

pub use diagnostic::Diagnostic;
pub use parser::Parser;

mod internal {
  pub use crate::contiguous_lines::ContiguousLines;
  pub use crate::delimiter::*;
  pub use crate::diagnostic::*;
  pub use crate::lexer::*;
  pub use crate::line::*;
  pub use crate::parser::*;
  pub use crate::tasks::collect_text::*;
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
