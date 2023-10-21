use bumpalo::collections::{String, Vec};

use super::AttrList;
use super::Macro;

// https://docs.asciidoctor.org/asciidoc/latest/key-concepts/#elements
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Inline<'alloc> {
  Bold(Vec<'alloc, Inline<'alloc>>),
  Highlight(Vec<'alloc, Inline<'alloc>>),
  Macro(Macro<'alloc>),
  Italic(Vec<'alloc, Inline<'alloc>>),
  LitMono(String<'alloc>),
  Mono(Vec<'alloc, Inline<'alloc>>),
  Superscript(Vec<'alloc, Inline<'alloc>>),
  Subscript(Vec<'alloc, Inline<'alloc>>),
  Text(String<'alloc>),
  TextSpan(AttrList<'alloc>, Vec<'alloc, Inline<'alloc>>),
}
