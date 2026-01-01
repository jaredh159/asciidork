use crate::html::htmlbuf::HtmlBuf;
use ast::{AttrValue, DocumentMeta, prelude::*};
use core::ReadAttr;

// NB: the awkward api here is because we want to make the common path
// of some classes and an id very fast, with minimal allocations, as it
// is used in the hot path of rendering html for many elements

pub struct OpenTag {
  buf: String,
  pub opened_classes: bool,
  append_classes: Option<String>,
  styles: Option<String>,
}

impl HtmlBuf for OpenTag {
  fn htmlbuf(&mut self) -> &mut String {
    &mut self.buf
  }
}

impl OpenTag {
  pub fn new(elem: &str, attrs: &impl AttrData) -> Self {
    Self::new_with_id(true, elem, attrs)
  }

  pub fn without_id(elem: &str, attrs: &impl AttrData) -> Self {
    Self::new_with_id(false, elem, attrs)
  }

  fn new_with_id(id: bool, elem: &str, attrs: &impl AttrData) -> Self {
    let mut tag = Self {
      buf: String::with_capacity(64),
      opened_classes: false,
      append_classes: None,
      styles: None,
    };

    tag.buf.push('<');
    tag.buf.push_str(elem);

    if id && let Some(id) = attrs.id() {
      tag.push_html_attr("id", id);
    }

    let mut roles = attrs.roles();
    if let Some(first_role) = roles.next() {
      let mut append = String::with_capacity(24);
      append.push_str(first_role);
      for role in roles {
        append.push(' ');
        append.push_str(role);
      }
      tag.append_classes = Some(append);
    }
    tag
  }

  pub fn push_str(&mut self, s: &str) {
    self.buf.push_str(s);
  }

  pub fn push_ch(&mut self, c: char) {
    self.buf.push(c);
  }

  pub fn push_class(&mut self, class: impl AsRef<str>) {
    self.push_prefixed_class(class, None);
  }

  pub fn push_classes(&mut self, source: impl Iterator<Item = impl AsRef<str>>) {
    for class in source {
      self.push_class(class);
    }
  }

  pub fn push_opt_class(&mut self, class: Option<impl AsRef<str>>) {
    self.push_opt_prefixed_class(class, None);
  }

  pub fn push_opt_prefixed_class(&mut self, class: Option<impl AsRef<str>>, prefix: Option<&str>) {
    if let Some(class) = class {
      self.push_prefixed_class(class, prefix);
    }
  }

  pub fn push_prefixed_class(&mut self, class: impl AsRef<str>, prefix: Option<&str>) {
    if !self.opened_classes {
      self.buf.push_str(" class=\"");
      self.opened_classes = true;
    } else {
      self.buf.push(' ');
    }
    if let Some(prefix) = prefix {
      self.buf.push_str(prefix);
    }
    self.buf.push_str(class.as_ref());
  }

  pub fn push_resolved_attr_class(
    &mut self,
    name: &str,
    default_value: Option<&str>,
    doc_name: Option<&str>,
    prefix: Option<&str>,
    chunk_meta: &ChunkMeta,
    doc_meta: &DocumentMeta,
  ) {
    match chunk_meta.attrs.named(name) {
      // `topbot` is a special case that gets normalized in html to `frame-ends`
      Some("topbot") if name == "frame" => self.push_prefixed_class("ends", prefix),
      Some(value) => self.push_prefixed_class(value, prefix),
      None => match doc_meta.get(doc_name.unwrap_or(name)) {
        Some(AttrValue::String(s)) if s == "topbot" && name == "frame" => {
          self.push_prefixed_class("ends", prefix);
        }
        Some(AttrValue::String(s)) => self.push_prefixed_class(s, prefix),
        _ => {
          if let Some(default_value) = default_value {
            self.push_prefixed_class(default_value, prefix);
          }
        }
      },
    }
  }

  pub fn push_style(&mut self, style: impl AsRef<str>) {
    if self.styles.is_none() {
      self.styles = Some(style.as_ref().to_string());
    } else {
      self.styles.as_mut().unwrap().push_str("; ");
      self.styles.as_mut().unwrap().push_str(style.as_ref());
    }
  }

  pub fn push_link_attrs(
    &mut self,
    attrs: &AttrList,
    has_link_text: bool,
    blank_window_shorthand: bool,
  ) {
    if !has_link_text {
      self.push_class("bare");
    }
    self.finish_classes();
    if let Some(title) = attrs.named("title") {
      self.push_html_attr("title", title)
    }
    if let Some(window) = attrs.named("window") {
      self.push_html_attr("target", window);
      if window == "_blank" || attrs.has_option("noopener") {
        self.push_str(" rel=\"noopener");
        if attrs.has_option("nofollow") {
          self.push_str(" nofollow\"");
        } else {
          self.push_ch('"');
        }
      }
    } else if blank_window_shorthand {
      self.push_str(" target=\"_blank\" rel=\"noopener\"");
    } else if attrs.has_option("nofollow") {
      self.push_str(" rel=\"nofollow\"");
    }
  }

  pub fn finish_classes(&mut self) {
    if let Some(append_classes) = self.append_classes.take() {
      if !self.opened_classes {
        self.buf.push_str(" class=\"");
        self.opened_classes = true;
      } else {
        self.buf.push(' ');
      }
      self.buf.push_str(&append_classes);
    }
    if self.opened_classes {
      self.buf.push('"');
    }
    self.opened_classes = false;
  }

  pub fn finish(mut self) -> String {
    self.finish_classes();
    if let Some(styles) = self.styles.take() {
      self.push_html_attr("style", &styles);
    }
    self.buf.push('>');
    self.buf
  }
}
