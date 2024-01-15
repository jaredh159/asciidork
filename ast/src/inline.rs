use std::ops::{Deref, DerefMut};

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

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct InlineNodes<'bmp>(BumpVec<'bmp, InlineNode<'bmp>>);

impl<'bmp> InlineNodes<'bmp> {
  pub fn new(bump: &'bmp Bump) -> Self {
    Self(BumpVec::new_in(bump))
  }
}

impl<'bmp> Deref for InlineNodes<'bmp> {
  type Target = BumpVec<'bmp, InlineNode<'bmp>>;
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl<'bmp> DerefMut for InlineNodes<'bmp> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

impl<'bmp> From<BumpVec<'bmp, InlineNode<'bmp>>> for InlineNodes<'bmp> {
  fn from(vec: BumpVec<'bmp, InlineNode<'bmp>>) -> Self {
    Self(vec)
  }
}

// https://docs.asciidoctor.org/asciidoc/latest/key-concepts/#elements
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Inline<'bmp> {
  Bold(InlineNodes<'bmp>),
  Curly(CurlyKind),
  Discarded,
  Highlight(InlineNodes<'bmp>),
  Macro(MacroNode<'bmp>),
  Italic(InlineNodes<'bmp>),
  InlinePassthrough(InlineNodes<'bmp>),
  JoiningNewline,
  LitMono(SourceString<'bmp>),
  Mono(InlineNodes<'bmp>),
  MultiCharWhitespace(BumpString<'bmp>),
  Quote(QuoteKind, InlineNodes<'bmp>),
  SpecialChar(SpecialCharKind),
  Superscript(InlineNodes<'bmp>),
  Subscript(InlineNodes<'bmp>),
  Text(BumpString<'bmp>),
  TextSpan(AttrList<'bmp>, InlineNodes<'bmp>),
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
