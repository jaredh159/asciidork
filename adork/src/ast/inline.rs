use super::AttrList;
use super::Macro;

// https://docs.asciidoctor.org/asciidoc/latest/key-concepts/#elements
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Inline {
  Bold(Vec<Inline>),
  Highlight(Vec<Inline>),
  Macro(Macro),
  Italic(Vec<Inline>),
  LitMono(String),
  Mono(Vec<Inline>),
  Superscript(Vec<Inline>),
  Subscript(Vec<Inline>),
  Text(String),
  TextSpan(AttrList, Vec<Inline>),
}

impl Inline {
  pub fn into_string(self) -> String {
    match self {
      Inline::Bold(inlines) => inlines.into_string(),
      Inline::Highlight(inlines) => inlines.into_string(),
      Inline::Italic(inlines) => inlines.into_string(),
      Inline::Macro(_) => String::new(), // this is weird...
      Inline::LitMono(text) => text,
      Inline::Mono(inlines) => inlines.into_string(),
      Inline::Superscript(inlines) => inlines.into_string(),
      Inline::Subscript(inlines) => inlines.into_string(),
      Inline::Text(text) => text,
      Inline::TextSpan(_, inlines) => inlines.into_string(),
    }
  }
}

pub trait Inlines {
  fn into_string(self) -> String;
}

impl Inlines for Vec<Inline> {
  fn into_string(self) -> String {
    self
      .into_iter()
      .map(|inline| inline.into_string())
      .collect()
  }
}
