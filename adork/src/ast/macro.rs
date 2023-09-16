use super::AttrList;

#[derive(Debug, PartialEq, Eq)]
pub enum Macro {
  Footnote(Option<String>, AttrList),
  Image(String, AttrList),
  Keyboard(AttrList),
}
