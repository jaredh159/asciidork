#![allow(dead_code)]

mod ast; // will be it's own crate at some point...
mod block;
mod evaluator;
mod lexer;
mod line;
pub mod parser;
mod source_location;
mod tasks;
mod token;

pub use parser::Parser;

#[derive(Debug)]
pub struct Diagnostic; // temp

type Result<T> = std::result::Result<T, Diagnostic>;
