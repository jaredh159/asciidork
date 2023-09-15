use super::AttrList;

#[derive(Debug, PartialEq, Eq)]
pub enum Macro {
  Keyboard(AttrList),
  Image(String, AttrList),
  Footnote(Option<String>, AttrList),
}
