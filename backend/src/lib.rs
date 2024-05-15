extern crate asciidork_ast as ast;
extern crate asciidork_opts as opts;

mod admonition;
mod backend;

// TODO: maybe move this into ast?
pub use admonition::AdmonitionKind;
pub use backend::Backend;

pub mod prelude {
  pub use super::AdmonitionKind;
  pub use super::Backend;
  pub use ast::DocType;
  pub use opts::{AttributeMissing, Opts};
}
