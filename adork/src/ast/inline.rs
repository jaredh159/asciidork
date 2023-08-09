// https://docs.asciidoctor.org/asciidoc/latest/key-concepts/#elements
#[derive(Debug, PartialEq, Eq)]
pub enum Inline {
  Text(String),
  Bold(Vec<Inline>),
  Italic(Vec<Inline>),
  Mono(String),
  LitMono(String),
}
