use crate::internal::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum MacroNode<'bmp> {
  Footnote {
    id: Option<SourceString<'bmp>>,
    text: InlineNodes<'bmp>,
  },
  Image {
    flow: Flow,
    target: SourceString<'bmp>,
    attrs: AttrList<'bmp>,
  },
  Keyboard {
    keys: BumpVec<'bmp, BumpString<'bmp>>,
    keys_src: SourceString<'bmp>,
  },
  Link {
    scheme: Option<UrlScheme>,
    target: SourceString<'bmp>,
    attrs: Option<AttrList<'bmp>>,
  },
  Pass {
    target: Option<SourceString<'bmp>>,
    content: InlineNodes<'bmp>,
  },
  Icon {
    target: SourceString<'bmp>,
    attrs: AttrList<'bmp>,
  },
  Button(SourceString<'bmp>),
  Menu(BumpVec<'bmp, SourceString<'bmp>>),
  Xref {
    id: SourceString<'bmp>,
    linktext: Option<InlineNodes<'bmp>>,
  },
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum UrlScheme {
  Https,
  Http,
  Ftp,
  Irc,
  Mailto,
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
      MacroNode::Footnote { id, text } => {
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
      MacroNode::Link { scheme, target, attrs } => {
        buf.push_str("Link\"");
        buf.add_option_member("scheme", scheme.as_ref());
        buf.add_member("target", target);
        buf.add_option_member("attrs", attrs.as_ref());
      }
      MacroNode::Pass { target, content } => {
        buf.push_str("Pass\"");
        buf.add_option_member("target", target.as_ref());
        buf.add_member("content", content);
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
