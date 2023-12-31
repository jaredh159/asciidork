use crate::internal::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum MacroNode<'bmp> {
  Footnote {
    id: Option<SourceString<'bmp>>,
    text: Vec<'bmp, InlineNode<'bmp>>,
  },
  Image {
    flow: Flow,
    target: SourceString<'bmp>,
    attrs: AttrList<'bmp>,
  },
  Keyboard {
    keys: Vec<'bmp, String<'bmp>>,
    keys_src: SourceString<'bmp>,
  },
  Link {
    scheme: Option<UrlScheme>,
    target: SourceString<'bmp>,
    attrs: Option<AttrList<'bmp>>,
  },
  Pass {
    target: Option<SourceString<'bmp>>,
    content: Vec<'bmp, InlineNode<'bmp>>,
  },
  Icon {
    target: SourceString<'bmp>,
    attrs: AttrList<'bmp>,
  },
  Button(SourceString<'bmp>),
  Menu(Vec<'bmp, SourceString<'bmp>>),
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
