use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

use crate::internal::*;

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
  pub const fn supports_special_section(&self, special_sect: SpecialSection) -> bool {
    matches!(
      (self, special_sect),
      (
        DocType::Article,
        SpecialSection::Abstract
          | SpecialSection::Appendix
          | SpecialSection::Glossary
          | SpecialSection::Bibliography
          | SpecialSection::Index
      ) | (
        DocType::Book,
        SpecialSection::Abstract
          | SpecialSection::Colophon
          | SpecialSection::Dedication
          | SpecialSection::Acknowledgments
          | SpecialSection::Preface
          | SpecialSection::Appendix
          | SpecialSection::Glossary
          | SpecialSection::Bibliography
          | SpecialSection::Index
      )
    )
  }
}
