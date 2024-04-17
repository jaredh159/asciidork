use crate::internal::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct InlineNode<'bmp> {
  pub content: Inline<'bmp>,
  pub loc: SourceLocation,
}

impl<'bmp> InlineNode<'bmp> {
  pub const fn new(content: Inline<'bmp>, loc: SourceLocation) -> Self {
    Self { content, loc }
  }
}

// https://docs.asciidoctor.org/asciidoc/latest/key-concepts/#elements
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Inline<'bmp> {
  AttributeReference(BumpString<'bmp>),
  Bold(InlineNodes<'bmp>),
  CurlyQuote(CurlyKind),
  Discarded,
  Highlight(InlineNodes<'bmp>),
  Macro(MacroNode<'bmp>),
  Italic(InlineNodes<'bmp>),
  InlinePassthrough(InlineNodes<'bmp>),
  JoiningNewline,
  CalloutNum(Callout),
  CalloutTuck(BumpString<'bmp>),
  LineBreak,
  LineComment(BumpString<'bmp>),
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

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum SpecialCharKind {
  Ampersand,
  LessThan,
  GreaterThan,
}

// json

impl<'bmp> Json for InlineNode<'bmp> {
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

impl<'bmp> Json for Inline<'bmp> {
  fn to_json_in(&self, buf: &mut JsonBuf) {
    buf.begin_obj("Inline");
    buf.push_str(r#","variant":""#);
    match self {
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
      Inline::Italic(nodes) => {
        buf.push_str("Italic\"");
        buf.add_member("children", nodes);
      }
      Inline::InlinePassthrough(nodes) => {
        buf.push_str("InlinePassthrough\"");
        buf.add_member("children", nodes);
      }
      Inline::JoiningNewline => buf.push_str("JoiningNewline\""),
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
