use asciidork_core::{file, Path, SafeMode};
use ast::{prelude::*, ReadAttr};

use crate::{html::OpenTag, utils, Backend};

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

  fn open_element(&mut self, element: &str, classes: &[&str], attrs: &impl AttrData) {
    let mut open_tag = OpenTag::new(element, attrs);
    classes.iter().for_each(|c| open_tag.push_class(c));
    self.push_open_tag(open_tag);
  }

  fn push_open_tag(&mut self, tag: OpenTag) {
    self.push_str(&tag.finish());
  }
}

// pub fn push_img_path(buf: &mut String, target: &str, doc_meta: &ast::DocumentMeta) {
//   if let Some(imagesdir) = doc_meta.str("imagesdir") {
//     let mut path = Path::new_specifying_separator(imagesdir, '/');
//     path.push(target);
//     push_url_encoded(buf, &path.to_string());
//   } else {
//     push_url_encoded(buf, target);
//   }
// }

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
    std::mem::take(&mut self.alt_htmlbuf())
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

pub trait HtmlBufBackend: Backend + HtmlBuf {
  fn push_icon_uri(&mut self, name: &str, prefix: Option<&str>) {
    // PERF: we could work to prevent all these allocations w/ some caching
    // these might get rendered many times in a given document
    let icondir = self.doc_meta().string_or("iconsdir", "./images/icons");
    let ext = self.doc_meta().string_or("icontype", "png");
    self.push([&icondir, "/", prefix.unwrap_or(""), name, ".", &ext]);
  }

  fn render_image(&mut self, target: &str, attrs: &AttrList, is_block: bool) {
    let format = attrs.named("format").or_else(|| file::ext(target));
    let is_svg = matches!(format, Some("svg" | "SVG"));
    if is_svg && attrs.has_option("interactive") && self.doc_meta().safe_mode != SafeMode::Secure {
      self.render_interactive_svg(target, attrs);
    }
    self.push_str(r#"<img src=""#);
    self.push_img_path(target);
    self.push_str(r#"" alt=""#);
    if let Some(alt) = attrs.named("alt").or_else(|| attrs.str_positional_at(0)) {
      self.push_str_attr_escaped(alt);
    } else if let Some(Some(nodes)) = attrs.positional.first() {
      for s in nodes.plain_text() {
        self.push_str_attr_escaped(s);
      }
    } else {
      let alt = file::stem(target).replace(['-', '_'], " ");
      self.push_str_attr_escaped(&alt);
    }
    self.push_ch('"');
    self.push_named_or_pos_attr("width", 1, attrs);
    self.push_named_or_pos_attr("height", 2, attrs);
    if !is_block {
      self.push_named_attr("title", attrs);
    }
    self.push_ch('>');
  }

  fn render_interactive_svg(&mut self, target: &str, attrs: &AttrList) {
    self.push_str(r#"<object type="image/svg+xml" data=""#);
    self.push_img_path(target);
    self.push_ch('"');
    self.push_named_or_pos_attr("width", 1, attrs);
    self.push_named_or_pos_attr("height", 2, attrs);
    self.push_ch('>');
    if let Some(fallback) = attrs.named("fallback") {
      self.push_str(r#"<img src=""#);
      self.push_img_path(fallback);
      self.push_ch('"');
      self.push_named_or_pos_attr("alt", 0, attrs);
      self.push_ch('>');
    } else if let Some(alt) = attrs.named("alt").or_else(|| attrs.str_positional_at(0)) {
      self.push([r#"<span class="alt">"#, alt, "</span>"]);
    }
    self.push_str("</object>");
  }

  fn push_img_path(&mut self, target: &str) {
    if let Some(imagesdir) = self.doc_meta().str("imagesdir") {
      let mut path = Path::new_specifying_separator(imagesdir, '/');
      path.push(target);
      self.push_url_encoded(&path.to_string());
    } else {
      self.push_url_encoded(target);
    }
  }

  fn render_missing_xref(
    &mut self,
    target: &SourceString,
    kind: XrefKind,
    doc_title: Option<&DocTitle>,
  ) {
    if target == "#" || Some(target.src.as_str()) == self.doc_meta().str("asciidork-docfilename") {
      let doctitle = doc_title
        .and_then(|t| t.attrs.named("reftext"))
        .unwrap_or_else(|| self.doc_meta().str("doctitle").unwrap_or("[^top]"))
        .to_string();
      self.push_str(&doctitle);
    } else if utils::xref::is_interdoc(target, kind) {
      let href = utils::xref::href(target, self.doc_meta(), kind, false);
      self.push_str(utils::xref::remove_leading_hash(&href));
    } else {
      self.push(["[", target.strip_prefix('#').unwrap_or(target), "]"]);
    }
  }
}

impl<T> HtmlBufBackend for T where T: Backend + HtmlBuf {}
