use crate::internal::*;

// https://docs.asciidoctor.org/asciidoc/latest/document/header/
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct DocHeader<'bmp> {
  pub title: Option<DocTitle<'bmp>>,
  pub authors: BumpVec<'bmp, Author<'bmp>>,
  pub revision: Option<Revision<'bmp>>,
  pub attrs: BumpVec<'bmp, (String, AttrValue)>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct DocTitle<'bmp> {
  pub heading: InlineNodes<'bmp>,
  pub subtitle: Option<InlineNodes<'bmp>>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
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

impl Json for DocHeader<'_> {
  fn to_json_in(&self, buf: &mut JsonBuf) {
    buf.begin_obj("DocHeader");
    buf.add_option_member("title", self.title.as_ref());
    buf.add_member("authors", &self.authors);
    buf.add_option_member("revision", self.revision.as_ref());
    buf.finish_obj();
  }
}

impl Json for DocTitle<'_> {
  fn to_json_in(&self, buf: &mut JsonBuf) {
    buf.begin_obj("DocTitle");
    buf.add_member("heading", &self.heading);
    buf.add_option_member("subtitle", self.subtitle.as_ref());
    buf.finish_obj();
  }
}

impl Json for Revision<'_> {
  fn to_json_in(&self, buf: &mut JsonBuf) {
    buf.begin_obj("Revision");
    buf.add_member("version", &self.version);
    buf.add_option_member("date", self.date.as_ref());
    buf.add_option_member("remark", self.remark.as_ref());
    buf.finish_obj();
  }
}

impl Json for Author<'_> {
  fn to_json_in(&self, buf: &mut JsonBuf) {
    buf.begin_obj("Author");
    buf.add_member("first_name", &self.first_name);
    buf.add_option_member("middle_name", self.middle_name.as_ref());
    buf.add_member("last_name", &self.last_name);
    buf.add_option_member("email", self.email.as_ref());
    buf.finish_obj();
  }
}
