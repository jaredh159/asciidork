use bumpalo::collections::String;
// use bumpalo::Bump;

use super::AttrList;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Macro<'bmp> {
  Footnote(Option<String<'bmp>>, AttrList<'bmp>),
  Image(String<'bmp>, AttrList<'bmp>),
  Keyboard(AttrList<'bmp>),
  Link(UrlScheme, String<'bmp>, AttrList<'bmp>),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum UrlScheme {
  Https,
  Http,
  Ftp,
  Irc,
  Mailto,
}
