use super::AttrList;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Macro {
  Footnote(Option<String>, AttrList),
  Image(String, AttrList),
  Keyboard(AttrList),
  Link(UrlScheme, String, AttrList),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum UrlScheme {
  Https,
  Http,
  Ftp,
  Irc,
  Mailto,
}
