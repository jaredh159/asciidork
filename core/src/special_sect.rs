use std::str::FromStr;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum SpecialSection {
  Abstract,
  Colophon,
  Dedication,
  Acknowledgments,
  Preface,
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
      Self::Appendix => "appendix",
      Self::Glossary => "glossary",
      Self::Bibliography => "bibliography",
      Self::Index => "index",
    }
  }

  pub const fn supports_subsections(&self) -> bool {
    match self {
      Self::Abstract => true,
      Self::Colophon => false,
      Self::Dedication => false,
      Self::Acknowledgments => true,
      Self::Preface => true,
      Self::Appendix => true,
      Self::Glossary => false,
      Self::Bibliography => false,
      Self::Index => true,
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
      "appendix" => Self::Appendix,
      "glossary" => Self::Glossary,
      "bibliography" => Self::Bibliography,
      "index" => Self::Index,
      _ => return Err(()),
    })
  }
}
