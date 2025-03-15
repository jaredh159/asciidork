use serde_json::{Map, Value};

use asciidork_ast::prelude::*;

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct Loc {
  pub line: u32,
  pub column: u32,
  pub file: Option<String>,
}

impl Loc {
  pub const fn new(line: u32, column: u32) -> Self {
    Self { line, column, file: None }
  }

  pub fn incr_column(&mut self) -> Self {
    let mut other = self.clone();
    other.column += 1;
    other
  }

  pub fn from_pos(offset: u32, src: &[u8]) -> Self {
    let (line, column) = line_number_with_offset(offset, src);
    Loc { line, column, file: None }
  }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LocSpan([Loc; 2]);

impl LocSpan {
  pub const fn new(start: Loc, end: Loc) -> Self {
    Self([start, end])
  }

  pub fn from_multi_loc(loc: &MultiSourceLocation, src: &[u8]) -> Self {
    Self::new(
      Loc::from_pos(loc.start_pos, src).incr_column(),
      Loc::from_pos(loc.end_pos, src),
    )
  }

  pub fn from_src_loc(astloc: SourceLocation, src: &[u8]) -> Self {
    Self::new(
      Loc::from_pos(astloc.start, src).incr_column(),
      Loc::from_pos(astloc.end, src),
    )
  }

  pub fn from_src_pair(start: SourceLocation, end: SourceLocation, src: &[u8]) -> Self {
    Self::new(
      Loc::from_pos(start.start, src).incr_column(),
      Loc::from_pos(end.end, src),
    )
  }

  pub fn into_value(self) -> Value {
    let mut start = Map::new();
    start.insert("line".to_string(), Value::Number(self.0[0].line.into()));
    start.insert("col".to_string(), Value::Number(self.0[0].column.into()));
    let mut end = Map::new();
    end.insert("line".to_string(), Value::Number(self.0[1].line.into()));
    end.insert("col".to_string(), Value::Number(self.0[1].column.into()));
    Value::Array(vec![Value::Object(start), Value::Object(end)])
  }
}

// TODO: generalize into core, dedup from source_lexer
// examine how it differs from source_lexer, provide as service
fn line_number_with_offset(byte_offset: u32, src: &[u8]) -> (u32, u32) {
  let mut line_number = 1;
  let mut offset: u32 = 0;
  for idx in 0..byte_offset {
    match src.get(idx as usize) {
      None => break,
      Some(b'\n') => {
        offset = 0;
        line_number += 1;
      }
      _ => offset += 1,
    }
  }
  (line_number, offset)
}
