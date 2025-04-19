use std::str::FromStr;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum SpecialSection {
  Abstract,
  Colophon,
  Dedication,
  Acknowledgments,
  Preface,
  PartIntro,
  Appendix,
  Glossary,
  Bibliography,
  Index,
}

impl SpecialSection {
  pub const fn to_str(&self) -> &'static str {
    match self {
      Self::Abstract => "abstract",
      Self::Colophon => "colophon",
      Self::Dedication => "dedication",
      Self::Acknowledgments => "acknowledgments",
      Self::Preface => "preface",
      Self::PartIntro => "partintro",
      Self::Appendix => "appendix",
      Self::Glossary => "glossary",
      Self::Bibliography => "bibliography",
      Self::Index => "index",
    }
  }
}

impl FromStr for SpecialSection {
  type Err = ();
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Ok(match s {
      "abstract" => Self::Abstract,
      "colophon" => Self::Colophon,
      "dedication" => Self::Dedication,
      "acknowledgments" => Self::Acknowledgments,
      "preface" => Self::Preface,
      "partintro" => Self::PartIntro,
      "appendix" => Self::Appendix,
      "glossary" => Self::Glossary,
      "bibliography" => Self::Bibliography,
      "index" => Self::Index,
      _ => return Err(()),
    })
  }
}
