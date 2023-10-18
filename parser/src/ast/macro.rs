use bumpalo::collections::String;
// use bumpalo::Bump;

use super::AttrList;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Macro<'alloc> {
  Footnote(Option<String<'alloc>>, AttrList<'alloc>),
  Image(String<'alloc>, AttrList<'alloc>),
  Keyboard(AttrList<'alloc>),
  Link(UrlScheme, String<'alloc>, AttrList<'alloc>),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum UrlScheme {
  Https,
  Http,
  Ftp,
  Irc,
  Mailto,
}
