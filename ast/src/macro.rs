use crate::internal::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum MacroNode<'arena> {
  Footnote {
    number: u16,
    id: Option<SourceString<'arena>>,
    text: InlineNodes<'arena>,
  },
  Image {
    flow: Flow,
    target: SourceString<'arena>,
    attrs: AttrList<'arena>,
  },
  Keyboard {
    keys: BumpVec<'arena, BumpString<'arena>>,
    keys_src: SourceString<'arena>,
  },
  Link {
    scheme: Option<UrlScheme>,
    target: SourceString<'arena>,
    attrs: Option<AttrList<'arena>>,
    caret: bool,
  },
  Icon {
    target: SourceString<'arena>,
    attrs: AttrList<'arena>,
  },
  Button(SourceString<'arena>),
  Menu(BumpVec<'arena, SourceString<'arena>>),
  Xref {
    id: SourceString<'arena>,
    linktext: Option<InlineNodes<'arena>>,
  },
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum UrlScheme {
  Https,
  Http,
  Ftp,
  Irc,
  Mailto,
  File,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Flow {
  Inline,
  Block,
}

impl Json for Flow {
  fn to_json_in(&self, buf: &mut JsonBuf) {
    buf.push_obj_enum_type("Flow", self);
  }
}

impl Json for UrlScheme {
  fn to_json_in(&self, buf: &mut JsonBuf) {
    buf.push_obj_enum_type("UrlScheme", self);
  }
}

impl Json for MacroNode<'_> {
  fn to_json_in(&self, buf: &mut JsonBuf) {
    buf.begin_obj("MacroNode");
    buf.push_str(r#","variant":""#);
    match self {
      MacroNode::Footnote { id, text, .. } => {
        buf.push_str("Footnote\"");
        buf.add_option_member("id", id.as_ref());
        buf.add_member("text", text);
      }
      MacroNode::Image { flow, target, attrs } => {
        buf.push_str("Image\"");
        buf.add_member("flow", flow);
        buf.add_member("target", target);
        if !attrs.is_empty() {
          buf.add_member("attrs", attrs);
        }
      }
      MacroNode::Keyboard { keys, .. } => {
        buf.push_str("Keyboard\"");
        buf.add_member("keys", keys);
      }
      MacroNode::Link { scheme, target, attrs, .. } => {
        buf.push_str("Link\"");
        buf.add_option_member("scheme", scheme.as_ref());
        buf.add_member("target", target);
        buf.add_option_member("attrs", attrs.as_ref());
      }
      MacroNode::Icon { target, attrs } => {
        buf.push_str("Icon\"");
        buf.add_member("target", target);
        buf.add_member("attrs", attrs);
      }
      MacroNode::Button(s) => {
        buf.push_str("Button\"");
        buf.add_member("text", s);
      }
      MacroNode::Menu(items) => {
        buf.push_str("Menu\"");
        buf.add_member("items", items);
      }
      MacroNode::Xref { id, linktext } => {
        buf.push_str("Xref\"");
        buf.add_member("id", id);
        buf.add_member("linktext", linktext);
      }
    }
    buf.finish_obj();
  }
}
