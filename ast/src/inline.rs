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
  Bold(BumpVec<'bmp, InlineNode<'bmp>>),
  Curly(CurlyKind),
  Discarded,
  Highlight(BumpVec<'bmp, InlineNode<'bmp>>),
  Macro(MacroNode<'bmp>),
  Italic(BumpVec<'bmp, InlineNode<'bmp>>),
  InlinePassthrough(BumpVec<'bmp, InlineNode<'bmp>>),
  JoiningNewline,
  LitMono(SourceString<'bmp>),
  Mono(BumpVec<'bmp, InlineNode<'bmp>>),
  MultiCharWhitespace(BumpString<'bmp>),
  Quote(QuoteKind, BumpVec<'bmp, InlineNode<'bmp>>),
  SpecialChar(SpecialCharKind),
  Superscript(BumpVec<'bmp, InlineNode<'bmp>>),
  Subscript(BumpVec<'bmp, InlineNode<'bmp>>),
  Text(BumpString<'bmp>),
  TextSpan(AttrList<'bmp>, BumpVec<'bmp, InlineNode<'bmp>>),
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
