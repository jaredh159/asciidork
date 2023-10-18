use bumpalo::collections::String;

use super::AttrList;
use super::Macro;

// https://docs.asciidoctor.org/asciidoc/latest/key-concepts/#elements
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Inline<'alloc> {
  Bold(Vec<Inline<'alloc>>),
  Highlight(Vec<Inline<'alloc>>),
  Macro(Macro<'alloc>),
  Italic(Vec<Inline<'alloc>>),
  LitMono(String<'alloc>),
  Mono(Vec<Inline<'alloc>>),
  Superscript(Vec<Inline<'alloc>>),
  Subscript(Vec<Inline<'alloc>>),
  Text(String<'alloc>),
  TextSpan(AttrList<'alloc>, Vec<Inline<'alloc>>),
}
