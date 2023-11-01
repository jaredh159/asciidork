use bumpalo::collections::{String, Vec};

use super::AttrList;
use super::Macro;
use super::SourceLocation;

// https://docs.asciidoctor.org/asciidoc/latest/key-concepts/#elements
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Inline<'bmp> {
  Bold(Vec<'bmp, Inline<'bmp>>),
  Highlight(Vec<'bmp, Inline<'bmp>>),
  Macro(Macro<'bmp>),
  Italic(Vec<'bmp, Inline<'bmp>>),
  LitMono(String<'bmp>),
  Mono(Vec<'bmp, Inline<'bmp>>),
  Superscript(Vec<'bmp, Inline<'bmp>>),
  Subscript(Vec<'bmp, Inline<'bmp>>),
  Text(String<'bmp>),
  TextSpan(AttrList<'bmp>, Vec<'bmp, Inline<'bmp>>),
}
