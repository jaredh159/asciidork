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
  AttributeReference(BumpString<'arena>),
  Bold(InlineNodes<'arena>),
  CurlyQuote(CurlyKind),
  Discarded,
  Highlight(InlineNodes<'arena>),
  Macro(MacroNode<'arena>),
  Italic(InlineNodes<'arena>),
  InlinePassthrough(InlineNodes<'arena>),
  IncludeBoundary(IncludeBoundaryKind, u16),
  Newline,
  CalloutNum(Callout),
  CalloutTuck(BumpString<'arena>),
  LegacyInlineAnchor(BumpString<'arena>),
  LineBreak,
  LineComment(BumpString<'arena>),
  LitMono(SourceString<'arena>),
  Mono(InlineNodes<'arena>),
  MultiCharWhitespace(BumpString<'arena>),
  Quote(QuoteKind, InlineNodes<'arena>),
  SpecialChar(SpecialCharKind),
  Superscript(InlineNodes<'arena>),
  Subscript(InlineNodes<'arena>),
  Text(BumpString<'arena>),
  TextSpan(AttrList<'arena>, InlineNodes<'arena>),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum QuoteKind {
  Double,
  Single,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum IncludeBoundaryKind {
  Begin,
  End,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
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

// json

impl<'arena> Json for InlineNode<'arena> {
  fn to_json_in(&self, buf: &mut JsonBuf) {
    buf.begin_obj("InlineNode");
    buf.add_member("content", &self.content);
    buf.finish_obj();
  }
}

impl Json for QuoteKind {
  fn to_json_in(&self, buf: &mut JsonBuf) {
    buf.push_obj_enum_type("QuoteKind", self);
  }
}

impl Json for SpecialCharKind {
  fn to_json_in(&self, buf: &mut JsonBuf) {
    buf.push_obj_enum_type("SpecialCharKind", self);
  }
}

impl Json for CurlyKind {
  fn to_json_in(&self, buf: &mut JsonBuf) {
    buf.push_obj_enum_type("CurlyKind", self);
  }
}

impl<'arena> Json for Inline<'arena> {
  fn to_json_in(&self, buf: &mut JsonBuf) {
    buf.begin_obj("Inline");
    buf.push_str(r#","variant":""#);
    match self {
      Inline::IncludeBoundary(..) => todo!(),
      Inline::AttributeReference(s) => {
        buf.push_str("AttributeReference\"");
        buf.add_member("name", s);
      }
      Inline::Bold(nodes) => {
        buf.push_str("Bold\"");
        buf.add_member("children", nodes);
      }
      Inline::CurlyQuote(kind) => {
        buf.push_str("CurlyQuote\"");
        buf.add_member("kind", kind);
      }
      Inline::Discarded => buf.push_str("Discarded\""),
      Inline::Highlight(nodes) => {
        buf.push_str("Highlight\"");
        buf.add_member("children", nodes);
      }
      Inline::Macro(m) => {
        buf.push_str("Macro\"");
        buf.add_member("macro", m);
      }
      Inline::LegacyInlineAnchor(attrs) => {
        buf.push_str("LegacyInlineAnchor\"");
        buf.add_member("attrs", attrs);
      }
      Inline::Italic(nodes) => {
        buf.push_str("Italic\"");
        buf.add_member("children", nodes);
      }
      Inline::InlinePassthrough(nodes) => {
        buf.push_str("InlinePassthrough\"");
        buf.add_member("children", nodes);
      }
      Inline::Newline => buf.push_str("JoiningNewline\""),
      Inline::CalloutNum(callout) => {
        buf.push_str("CalloutNum\"");
        buf.add_member("callout", callout);
      }
      Inline::CalloutTuck(_) => buf.push_str("CalloutTuck\""),
      Inline::LineBreak => buf.push_str("LineBreak\""),
      Inline::LineComment(_) => buf.push_str("LineComment\""),
      Inline::LitMono(text) => {
        buf.push_str("LitMono\"");
        buf.add_member("text", text);
      }
      Inline::Mono(nodes) => {
        buf.push_str("Mono\"");
        buf.add_member("children", nodes);
      }
      Inline::MultiCharWhitespace(ws) => {
        buf.push_str("MultiCharWhitespace\"");
        buf.add_member("length", &ws.len());
      }
      Inline::Quote(kind, nodes) => {
        buf.push_str("Quote\"");
        buf.add_member("kind", kind);
        buf.add_member("children", nodes);
      }
      Inline::SpecialChar(kind) => {
        buf.push_str("SpecialChar\"");
        buf.add_member("kind", kind);
      }
      Inline::Superscript(nodes) => {
        buf.push_str("Superscript\"");
        buf.add_member("children", nodes);
      }
      Inline::Subscript(nodes) => {
        buf.push_str("Subscript\"");
        buf.add_member("children", nodes);
      }
      Inline::Text(s) => {
        buf.push_str("Text\"");
        buf.add_member("text", s);
      }
      Inline::TextSpan(attrs, nodes) => {
        buf.push_str("TextSpan\"");
        buf.add_member("attrs", attrs);
        buf.add_member("children", nodes);
      }
    }
    buf.finish_obj();
  }
}
