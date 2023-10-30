#![allow(dead_code)]

mod ast; // will be it's own crate at some point...
mod block;
mod diagnostic;
mod lexer;
mod line;
pub mod parser;
mod source_location;
mod tasks;
mod token;

pub use diagnostic::Diagnostic;
pub use parser::Parser;

type Result<T> = std::result::Result<T, Diagnostic>;
