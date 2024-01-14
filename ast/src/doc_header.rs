use crate::internal::*;

// https://docs.asciidoctor.org/asciidoc/latest/document/header/
#[derive(Debug, PartialEq, Eq)]
pub struct DocHeader<'bmp> {
  pub title: Option<DocTitle<'bmp>>,
  pub authors: Vec<'bmp, Author<'bmp>>,
  pub revision: Option<Revision<'bmp>>,
  // 👍 thurs jared: make non optional up at the doc level, maybe with
  // an empty entries that gets handed out if doc header not present
  pub attrs: AttrEntries,
}

#[derive(Debug, PartialEq, Eq)]
pub struct DocTitle<'bmp> {
  pub heading: Vec<'bmp, InlineNode<'bmp>>,
  pub subtitle: Option<Vec<'bmp, InlineNode<'bmp>>>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Revision<'bmp> {
  pub version: String<'bmp>,
  pub date: Option<String<'bmp>>,
  pub remark: Option<String<'bmp>>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Author<'bmp> {
  pub first_name: String<'bmp>,
  pub middle_name: Option<String<'bmp>>,
  pub last_name: String<'bmp>,
  pub email: Option<String<'bmp>>,
}

impl<'bmp> Author<'bmp> {
  pub fn fullname(&self) -> StdString {
    let mut name = StdString::with_capacity(
      self.first_name.len()
        + self.last_name.len()
        + self.middle_name.as_ref().map_or(0, |s| s.len())
        + 2,
    );
    name.push_str(&self.first_name);
    if let Some(middle_name) = &self.middle_name {
      name.push(' ');
      name.push_str(middle_name);
    }
    name.push(' ');
    name.push_str(&self.last_name);
    name
  }
}
