use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Default)]
pub enum DocType {
  #[default]
  Article,
  Book,
  Manpage,
  Inline,
}

impl DocType {
  pub const fn to_str(&self) -> &'static str {
    match self {
      Self::Article => "article",
      Self::Book => "book",
      Self::Manpage => "manpage",
      Self::Inline => "inline",
    }
  }
}

impl Display for DocType {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.to_str())
  }
}

impl FromStr for DocType {
  type Err = &'static str;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Ok(match s {
      "article" => Self::Article,
      "book" => Self::Book,
      "manpage" => Self::Manpage,
      "inline" => Self::Inline,
      _ => return Err("Invalid doctype: expected `article`, `book`, `manpage`, or `inline`"),
    })
  }
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
