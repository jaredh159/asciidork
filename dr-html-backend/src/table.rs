use crate::asciidoctor_html::num_str;
use crate::internal::*;

impl AsciidoctorHtml {
  pub(super) fn open_table_element(&mut self, block: &Block) {
    let mut classes = SmallVec::<[&str; 6]>::from_slice(&["tableblock"]);
    match block.meta.attr_named("frame") {
      Some("ends") | Some("topbot") => classes.push("frame-ends"),
      Some("none") => classes.push("frame-none"),
      Some("sides") => classes.push("frame-sides"),
      _ => classes.push("frame-all"),
    }
    // temp
    classes.push("grid-all");
    if block.meta.has_attr_option("autowidth") {
      classes.push("fit-content");
    } else {
      classes.push("stretch");
    }
    if let Some(float) = block.meta.attr_named("float") {
      classes.push(float);
    }
    if let Some(stripes) = block.meta.attr_named("stripes") {
      match stripes {
        "even" => classes.push("stripes-even"),
        "odd" => classes.push("stripes-odd"),
        "all" => classes.push("stripes-all"),
        "hover" => classes.push("stripes-hover"),
        "none" => classes.push("stripes-none"),
        _ => {}
      }
    }
    self.open_element("table", &classes, block.meta.attrs.as_ref());

    if let Some(mut width) = block.meta.attr_named("width") {
      if let Some(trimmed) = width.strip_suffix('%') {
        width = trimmed;
      }
      self.html.pop();
      self.push([r#" style="width: "#, width, "%;\">"]);
    }
  }

  pub(super) fn table_caption(&mut self, block: &Block) {
    if !self.alt_html.is_empty() {
      self.push_str(r#"<caption class="title">"#);
      if let Some(caption) = block.meta.attr_named("caption") {
        self.push_str(caption);
      } else if !self.doc_attrs.is_unset("table-caption") {
        self.table_caption_num += 1;
        self.push(["Table ", &num_str!(self.table_caption_num), ". "]);
      }
      let title = std::mem::take(&mut self.alt_html);
      self.push([&title, "</caption>"]);
    }
  }
}
