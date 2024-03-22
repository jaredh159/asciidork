#[derive(Debug, Default, Clone, Copy)]
pub struct Opts {
  pub doc_type: DocType,
  pub attribute_missing: AttributeMissing,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum AttributeMissing {
  Warn,
  Drop,
  Skip,
  // dr. also has "drop-line", i'd rather not support it
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum DocType {
  Article,
  Book,
  Manpage,
  Inline,
}

impl DocType {
  // https://docs.asciidoctor.org/asciidoc/latest/sections/styles/
  pub fn supports_special_section(&self, name: &str) -> bool {
    matches!(
      (self, name),
      (
        DocType::Article,
        "abstract" | "appendix" | "glossary" | "bibliography" | "index"
      ) | (
        DocType::Book,
        "abstract"
          | "colophon"
          | "dedication"
          | "acknowledgments"
          | "preface"
          | "partintro"
          | "appendix"
          | "glossary"
          | "bibliography"
          | "index",
      )
    )
  }
}

impl Default for AttributeMissing {
  fn default() -> Self {
    Self::Skip
  }
}

impl Default for DocType {
  fn default() -> Self {
    Self::Article
  }
}

impl Opts {
  pub fn embedded() -> Self {
    Self {
      doc_type: DocType::Inline,
      ..Self::default()
    }
  }
}
