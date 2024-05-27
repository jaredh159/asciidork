use asciidork_meta::DocType;

#[derive(Debug, Default, Clone, Copy)]
pub struct Opts {
  pub doc_type: DocType,
  pub strict: bool,
}

impl Opts {
  pub fn embedded() -> Self {
    Self {
      doc_type: DocType::Inline,
      ..Self::default()
    }
  }
}
