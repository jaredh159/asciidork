use crate::internal::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct InlineNode<'bmp> {
  pub content: Inline<'bmp>,
  pub loc: SourceLocation,
}

impl<'bmp> InlineNode<'bmp> {
  pub fn new(content: Inline<'bmp>, loc: SourceLocation) -> Self {
    Self { content, loc }
  }
}

// https://docs.asciidoctor.org/asciidoc/latest/key-concepts/#elements
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Inline<'bmp> {
  Bold(Vec<'bmp, InlineNode<'bmp>>),
  Curly(CurlyKind),
  Discarded,
  Highlight(Vec<'bmp, InlineNode<'bmp>>),
  Macro(MacroNode<'bmp>),
  Italic(Vec<'bmp, InlineNode<'bmp>>),
  InlinePassthrough(Vec<'bmp, InlineNode<'bmp>>),
  JoiningNewline,
  LitMono(SourceString<'bmp>),
  Mono(Vec<'bmp, InlineNode<'bmp>>),
  MultiCharWhitespace(String<'bmp>),
  Quote(QuoteKind, Vec<'bmp, InlineNode<'bmp>>),
  SpecialChar(SpecialCharKind),
  Superscript(Vec<'bmp, InlineNode<'bmp>>),
  Subscript(Vec<'bmp, InlineNode<'bmp>>),
  Text(String<'bmp>),
  TextSpan(AttrList<'bmp>, Vec<'bmp, InlineNode<'bmp>>),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum QuoteKind {
  Double,
  Single,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum CurlyKind {
  LeftDouble,
  RightDouble,
  LeftSingle,
  RightSingle,
  LegacyImplicitApostrophe,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum SpecialCharKind {
  Ampersand,
  LessThan,
  GreaterThan,
}
