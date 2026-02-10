extern crate asciidork_ast as ast;
extern crate asciidork_core as core;

mod backend;
pub mod html;
pub mod time;
pub mod utils;

pub use backend::Backend;

pub mod prelude {
  pub use super::Backend;
  pub use core::{AttrValue, DocType};
}

#[cfg(debug_assertions)]
#[macro_use]
pub mod test;
