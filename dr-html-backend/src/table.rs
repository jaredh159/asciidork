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

    let explicit_width = block
      .meta
      .attr_named("width")
      .map(|width| width.strip_suffix('%').unwrap_or(width))
      .and_then(|width| width.parse::<u8>().ok())
      .filter(|width| *width != 100);

    // temp
    classes.push("grid-all");

    if block.meta.has_attr_option("autowidth") {
      classes.push("fit-content");
    } else if explicit_width.is_none() {
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

    if let Some(width) = explicit_width {
      self.html.pop();
      self.push([r#" style="width: "#, &num_str!(width), "%;\">"]);
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

  pub(super) fn open_cell(&mut self, cell: &Cell, section: TableSection) {
    if matches!(section, TableSection::Header) || matches!(cell.content, CellContent::Header(_)) {
      self.push_str("<th");
    } else {
      self.push_str("<td");
    }
    self.push([" class=\"tableblock halign-", cell.h_align.word()]);
    self.push([" valign-", cell.v_align.word()]);
    if cell.col_span > 1 {
      self.push(["\" colspan=\"", &num_str!(cell.col_span)]);
    }
    if cell.row_span > 1 {
      self.push(["\" rowspan=\"", &num_str!(cell.row_span)]);
    }
    self.push_str("\">");

    match (section, &cell.content) {
      (TableSection::Header, _) => {}
      (_, CellContent::Default(_) | CellContent::Header(_)) => {
        self.push_str("<p class=\"tableblock\">")
      }
      (_, CellContent::Emphasis(_)) => self.push_str("<p class=\"tableblock\"><em>"),
      (_, CellContent::Literal(_)) => self.push_str("<div class=\"literal\"><pre>"),
      (_, CellContent::Monospace(_)) => self.push_str("<p class=\"tableblock\"><code>"),
      (_, CellContent::Strong(_)) => self.push_str("<p class=\"tableblock\"><strong>"),
      (_, CellContent::AsciiDoc(_)) => {}
    }
  }

  pub(super) fn close_cell(&mut self, cell: &Cell, section: TableSection) {
    match (section, &cell.content) {
      (TableSection::Header, _) => self.push_str("</th>"),
      (TableSection::Body, CellContent::Header(_)) => self.push_str("</p></th>"),
      (_, CellContent::Emphasis(_)) => self.push_str("</em></p></td>"),
      (_, CellContent::Literal(_)) => self.push_str("</pre></div></td>"),
      (_, CellContent::Monospace(_)) => self.push_str("</code></p></td>"),
      (_, CellContent::Strong(_)) => self.push_str("</strong></p></td>"),
      (_, CellContent::AsciiDoc(_)) => {}
      _ => self.push_str("</p></td>"),
    }
  }
}
