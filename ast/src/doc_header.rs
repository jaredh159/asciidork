use crate::internal::*;

// https://docs.asciidoctor.org/asciidoc/latest/document/header/
#[derive(Debug, PartialEq, Eq)]
pub struct DocHeader<'bmp> {
  pub title: Option<DocTitle<'bmp>>,
  pub authors: BumpVec<'bmp, Author<'bmp>>,
  pub revision: Option<Revision<'bmp>>,
  pub attrs: AttrEntries,
}

#[derive(Debug, PartialEq, Eq)]
pub struct DocTitle<'bmp> {
  pub heading: InlineNodes<'bmp>,
  pub subtitle: Option<InlineNodes<'bmp>>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Revision<'bmp> {
  pub version: BumpString<'bmp>,
  pub date: Option<BumpString<'bmp>>,
  pub remark: Option<BumpString<'bmp>>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Author<'bmp> {
  pub first_name: BumpString<'bmp>,
  pub middle_name: Option<BumpString<'bmp>>,
  pub last_name: BumpString<'bmp>,
  pub email: Option<BumpString<'bmp>>,
}

impl<'bmp> Author<'bmp> {
  pub fn fullname(&self) -> String {
    let mut name = String::with_capacity(
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
