use asciidork_meta::DocType;

#[derive(Debug, Default, Clone, Copy)]
pub struct Opts {
  pub doc_type: DocType,
  pub attribute_missing: AttributeMissing,
  pub strict: bool,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Default)]
pub enum AttributeMissing {
  Warn,
  Drop,
  #[default]
  Skip,
  // dr. also has "drop-line", i'd rather not support it
}

impl Opts {
  pub fn embedded() -> Self {
    Self {
      doc_type: DocType::Inline,
      ..Self::default()
    }
  }
}
