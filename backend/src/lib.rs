extern crate asciidork_ast as ast;
extern crate asciidork_core as core;

mod admonition;
mod backend;
pub mod utils;

// TODO: maybe move this into ast?
pub use admonition::AdmonitionKind;

pub use backend::Backend;

pub mod prelude {
  pub use super::AdmonitionKind;
  pub use super::Backend;
  pub use core::{AttrValue, DocType};
}
