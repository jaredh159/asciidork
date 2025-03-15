use miniserde::Serialize;

use asciidork_parser::prelude::Diagnostic;

#[derive(Debug, Eq, PartialEq, Clone, Serialize)]
pub struct DiagnosticError {
  pub full_line: String,
  pub message: String,
  pub line_num: u32,
  pub column_num_start: u32,
  pub column_num_end: u32,
  pub source_file: String,
}

impl From<Diagnostic> for DiagnosticError {
  fn from(diagnostic: Diagnostic) -> Self {
    Self {
      full_line: diagnostic.line.clone(),
      message: diagnostic.message.clone(),
      line_num: diagnostic.line_num,
      column_num_start: diagnostic.underline_start,
      column_num_end: diagnostic.underline_start + diagnostic.underline_width,
      source_file: diagnostic.source_file.file_name().to_string(),
    }
  }
}
