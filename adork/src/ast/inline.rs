use super::AttrList;

// https://docs.asciidoctor.org/asciidoc/latest/key-concepts/#elements
#[derive(Debug, PartialEq, Eq)]
pub enum Inline {
  Bold(Vec<Inline>),
  Italic(Vec<Inline>),
  LitMono(String),
  Mono(Vec<Inline>),
  Superscript(Vec<Inline>),
  Subscript(Vec<Inline>),
  Text(String),
  TextSpan(AttrList, Vec<Inline>),
}
