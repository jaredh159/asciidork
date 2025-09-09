use crate::internal::*;

pub trait HtmlBuf {
  fn htmlbuf(&mut self) -> &mut String;

  fn push_str_attr_escaped(&mut self, s: &str) {
    for c in s.chars() {
      match c {
        '"' => self.htmlbuf().push_str("&quot;"),
        '\'' => self.htmlbuf().push_str("&#8217;"),
        '&' => self.htmlbuf().push_str("&amp;"),
        '<' => self.htmlbuf().push_str("&lt;"),
        '>' => self.htmlbuf().push_str("&gt;"),
        _ => self.htmlbuf().push(c),
      }
    }
  }

  #[allow(unused)]
  fn push_url_encoded(&mut self, s: &str) {
    push_url_encoded(self.htmlbuf(), s);
  }

  fn push_str(&mut self, s: &str) {
    self.htmlbuf().push_str(s);
  }

  fn push_ch(&mut self, c: char) {
    self.htmlbuf().push(c);
  }

  fn push<const N: usize>(&mut self, strs: [&str; N]) {
    for s in strs {
      self.push_str(s);
    }
  }

  fn push_html_attr(&mut self, name: &'static str, value: &str) {
    self.push([r#" "#, name, r#"=""#]);
    self.push_str_attr_escaped(value);
    self.push_ch('"');
  }

  fn push_named_attr(&mut self, name: &'static str, attrs: &AttrList) {
    if let Some(value) = attrs.named(name) {
      self.push_html_attr(name, value);
    }
  }

  fn push_named_or_pos_attr(&mut self, name: &'static str, pos: usize, attrs: &AttrList) {
    if let Some(value) = attrs.named(name).or_else(|| attrs.str_positional_at(pos)) {
      self.push_html_attr(name, value);
    }
  }
}

pub fn push_img_path(buf: &mut String, target: &str, doc_meta: &DocumentMeta) {
  if let Some(imagesdir) = doc_meta.str("imagesdir") {
    let mut path = Path::new_specifying_separator(imagesdir, '/');
    path.push(target);
    push_url_encoded(buf, &path.to_string());
  } else {
    push_url_encoded(buf, target);
  }
}

fn push_url_encoded(buf: &mut String, s: &str) {
  for c in s.chars() {
    match c {
      ' ' => buf.push_str("%20"),
      _ => buf.push(c),
    }
  }
}
