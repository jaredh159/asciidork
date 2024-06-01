use crate::internal::*;

// NB: the awkward api here is because we want to make the common path
// of some classes and an id very fast, with minimal allocations, as it
// is used in the hot path of rendering html for almost every element

pub struct OpenTag {
  buf: String,
  opened_classes: bool,
  append_classes: Option<String>,
  styles: Option<String>,
}

impl OpenTag {
  pub fn new(elem: &str, attrs: Option<&AttrList>) -> Self {
    Self::new_with_id(true, elem, attrs)
  }

  pub fn without_id(elem: &str, attrs: Option<&AttrList>) -> Self {
    Self::new_with_id(false, elem, attrs)
  }

  fn new_with_id(id: bool, elem: &str, attrs: Option<&AttrList>) -> Self {
    let mut tag = Self {
      buf: String::with_capacity(64),
      opened_classes: false,
      append_classes: None,
      styles: None,
    };

    tag.buf.push('<');
    tag.buf.push_str(elem);

    if id {
      if let Some(id) = attrs.as_ref().and_then(|a| a.id.as_ref()) {
        tag.buf.push_str(" id=\"");
        tag.buf.push_str(id);
        tag.buf.push('"');
      }
    }

    if let Some(mut roles) = attrs
      .as_ref()
      .map(|a| &a.roles)
      .filter(|r| !r.is_empty())
      .map(|r| r.iter())
    {
      let mut append = String::with_capacity(roles.len() * 12);
      append.push_str(roles.next().unwrap());
      for role in roles {
        append.push(' ');
        append.push_str(role);
      }
      tag.append_classes = Some(append);
    }
    tag
  }

  pub fn push_class(&mut self, class: impl AsRef<str>) {
    self.push_prefixed_class(class, None);
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
    match chunk_meta.attr_named(name) {
      Some(value) => self.push_prefixed_class(value, prefix),
      None => match doc_meta.get(doc_name.unwrap_or(name)) {
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

  pub fn finish(mut self) -> String {
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
    if let Some(styles) = self.styles.take() {
      self.buf.push_str(" style=\"");
      self.buf.push_str(&styles);
      self.buf.push('"');
    }
    self.buf.push('>');
    self.buf
  }
}
