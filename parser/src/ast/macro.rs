use super::*;
use crate::utils::bump::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Macro<'bmp> {
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
    scheme: UrlScheme,
    target: SourceString<'bmp>,
    attrs: Option<AttrList<'bmp>>,
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
