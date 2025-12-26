use crate::internal::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct InlineNode<'arena> {
  pub content: Inline<'arena>,
  pub loc: SourceLocation,
}

impl<'arena> InlineNode<'arena> {
  pub const fn new(content: Inline<'arena>, loc: SourceLocation) -> Self {
    Self { content, loc }
  }
}

// https://docs.asciidoctor.org/asciidoc/latest/key-concepts/#elements
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Inline<'arena> {
  CurlyQuote(CurlyKind),
  Discarded,
  Macro(MacroNode<'arena>),
  InlinePassthru(InlineNodes<'arena>),
  Newline,
  CalloutNum(Callout),
  CalloutTuck(BumpString<'arena>),
  IndexTerm(IndexTerm<'arena>),
  InlineAnchor(BumpString<'arena>),
  BiblioAnchor(BumpString<'arena>),
  LineBreak,
  LineComment(BumpString<'arena>),
  MultiCharWhitespace(BumpString<'arena>),
  Quote(QuoteKind, InlineNodes<'arena>),
  SpecialChar(SpecialCharKind),
  Symbol(SymbolKind),
  Text(BumpString<'arena>),
  SpacedDashes(u8, AdjacentNewline),
  Span(SpanKind, Option<AttrList<'arena>>, InlineNodes<'arena>),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SpanKind {
  Bold,
  Italic,
  LitMono,
  Mono,
  Highlight,
  Superscript,
  Subscript,
  Text,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum QuoteKind {
  Double,
  Single,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum CurlyKind {
  LeftDouble,
  RightDouble,
  LeftSingle,
  RightSingle,
  LegacyImplicitApostrophe,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SpecialCharKind {
  Ampersand,
  LessThan,
  GreaterThan,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum AdjacentNewline {
  None,
  Leading,
  Trailing,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SymbolKind {
  Copyright,
  Registered,
  Trademark,
  EmDash,
  TripleDash,
  Ellipsis,
  SingleRightArrow,
  DoubleRightArrow,
  SingleLeftArrow,
  DoubleLeftArrow,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct IndexTerm<'arena> {
  pub term_type: IndexTermType<'arena>,
  pub term_ref: IndexTermReference<'arena>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum IndexTermType<'arena> {
  Visible {
    term: InlineNodes<'arena>,
  },
  Concealed {
    primary: InlineNodes<'arena>,
    secondary: Option<InlineNodes<'arena>>,
    tertiary: Option<InlineNodes<'arena>>,
  },
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum IndexTermReference<'arena> {
  None,
  See(BumpString<'arena>),
  SeeAlso(Vec<BumpString<'arena>>),
}

#[test]
fn test_size_of_inline() {
  assert!(std::mem::size_of::<Inline>() <= 248);
}
