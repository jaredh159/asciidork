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
    target: SourceString<'arena>,
    linktext: Option<InlineNodes<'arena>>,
    kind: XrefKind,
  },
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum XrefKind {
  Shorthand,
  Macro,
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
