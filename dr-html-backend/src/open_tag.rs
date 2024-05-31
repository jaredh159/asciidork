use crate::internal::*;

pub struct OpenTag {
  buf: String,
  has_classes: bool,
  roles: Option<String>,
}

impl OpenTag {
  pub fn new(elem: &str, attrs: Option<&AttrList>) -> Self {
    let mut tag = OpenTag {
      buf: String::with_capacity(24),
      has_classes: false,
      roles: None,
    };
    tag.buf.push('<');
    tag.buf.push_str(elem);
    if let Some(id) = attrs.as_ref().and_then(|a| a.id.as_ref()) {
      tag.buf.push_str(" id=\"");
      tag.buf.push_str(id);
      tag.buf.push('"');
    }
    if let Some(roles) = attrs.as_ref().map(|a| &a.roles).filter(|r| !r.is_empty()) {
      let mut role_classes = String::with_capacity(roles.len() * 12);
      for role in roles {
        if !role_classes.is_empty() {
          role_classes.push(' ');
        }
        role_classes.push_str(role);
      }
      tag.roles = Some(role_classes);
    }
    tag
  }

  pub fn push_class(&mut self, class: impl AsRef<str>, prefix: Option<&str>) {
    if !self.has_classes {
      self.buf.push_str(" class=\"");
    } else {
      self.buf.push(' ');
    }
    if let Some(prefix) = prefix {
      self.buf.push_str(prefix);
    }
    self.buf.push_str(class.as_ref());
    self.has_classes = true;
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
      Some(value) => self.push_class(value, prefix),
      None => match doc_meta.get(doc_name.unwrap_or(name)) {
        Some(AttrValue::String(s)) => self.push_class(s, prefix),
        _ => {
          if let Some(default_value) = default_value {
            self.push_class(default_value, prefix);
          }
        }
      },
    }
  }

  pub fn finish(mut self) -> String {
    if let Some(roles) = self.roles.take() {
      self.push_class(&roles, None);
    }
    if self.has_classes {
      self.buf.push('"');
    }
    self.buf.push('>');
    self.buf
  }
}
