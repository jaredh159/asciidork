use ast::prelude::*;

use crate::html::OpenTag;

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
    self.push([" ", name, "=\""]);
    self.push_str_attr_escaped(value);
    self.push_ch('"');
  }

  fn push_html_attr_nodes(&mut self, name: &'static str, nodes: &ast::InlineNodes<'_>) {
    self.push([" ", name, "=\""]);
    if let Some(single) = nodes.single_text() {
      self.push_str_attr_escaped(single);
    } else {
      for s in nodes.plain_text() {
        self.push_str_attr_escaped(s);
      }
    }
    self.push_ch('"');
  }

  fn push_named_attr(&mut self, name: &'static str, attrs: &AttrList) {
    if let Some(nodes) = attrs.named.get(name) {
      self.push_html_attr_nodes(name, nodes);
    }
  }

  fn push_named_or_pos_attr(&mut self, name: &'static str, pos: usize, attrs: &AttrList) {
    if let Some(value) = attrs.named(name).or_else(|| attrs.str_positional_at(pos)) {
      self.push_html_attr(name, value);
    }
  }

  fn open_element(&mut self, element: &str, classes: &[&str], attrs: &impl AttrData) {
    let mut open_tag = OpenTag::new(element, attrs);
    classes.iter().for_each(|c| open_tag.push_class(c));
    self.push_open_tag(open_tag);
  }

  fn open_element_opt(&mut self, element: &str, classes: &[&str], attrs: Option<&impl AttrData>) {
    if let Some(attrs) = attrs {
      let mut open_tag = OpenTag::new(element, attrs);
      classes.iter().for_each(|c| open_tag.push_class(c));
      self.push_open_tag(open_tag);
    } else {
      self.push(["<", element, ">"]);
    }
  }

  fn push_open_tag(&mut self, tag: OpenTag) {
    self.push_str(&tag.finish());
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

pub trait AltHtmlBuf: HtmlBuf {
  fn alt_htmlbuf(&mut self) -> &mut String;
  /// (htmlbuf, alt_htmlbuf)
  fn buffers(&mut self) -> (&mut String, &mut String);

  fn push_buffered(&mut self) {
    let buffer = self.take_buffer();
    self.push_str(&buffer);
  }

  fn take_buffer(&mut self) -> String {
    std::mem::take(self.alt_htmlbuf())
  }

  fn swap_take_buffer(&mut self) -> String {
    let (html, alt_html) = self.buffers();
    std::mem::swap(alt_html, html);
    std::mem::take(alt_html)
  }

  fn start_buffering(&mut self) {
    self.swap_buffers();
  }

  fn stop_buffering(&mut self) {
    self.swap_buffers();
  }

  fn swap_buffers(&mut self) {
    let (html, alt_html) = self.buffers();
    std::mem::swap(html, alt_html);
  }
}
