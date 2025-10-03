use crate::html5s::{num_str, Newlines};
use crate::internal::*;

impl Html5s {
  pub(super) fn open_table_element(&mut self, block: &Block) {
    let mut tag = OpenTag::new("table", &NoAttrs);
    // tag.push_class("tableblock"); // tablenew

    tag.push_resolved_attr_class(
      "frame",
      Some("all"),
      Some("table-frame"),
      Some("frame-"),
      &block.meta,
      &self.doc_meta,
    );

    tag.push_resolved_attr_class(
      "grid",
      Some("all"),
      Some("table-grid"),
      Some("grid-"),
      &block.meta,
      &self.doc_meta,
    );

    let explicit_width = block
      .meta
      .attrs
      .named("width")
      .map(|width| width.strip_suffix('%').unwrap_or(width))
      .and_then(|width| width.parse::<u8>().ok())
      .filter(|width| *width != 100);

    if block.meta.attrs.has_option("autowidth") && explicit_width.is_none() {
      tag.push_class("fit-content");
    } else if explicit_width.is_none() {
      tag.push_class("stretch");
    }

    if let Some(float) = block.meta.attrs.named("float") {
      tag.push_class(float);
    }

    tag.push_resolved_attr_class(
      "stripes",
      None,
      Some("table-stripes"),
      Some("stripes-"),
      &block.meta,
      &self.doc_meta,
    );

    if let Some(width) = explicit_width {
      tag.push_style(format!("width: {width}%;"));
    }

    self.push_open_tag(tag);
  }

  pub(super) fn open_cell(&mut self, cell: &Cell, section: TableSection) {
    if matches!(section, TableSection::Header) || matches!(cell.content, CellContent::Header(_)) {
      self.push_str("<th");
    } else {
      self.push_str("<td");
    }
    // tablenew swap
    // self.push([" class=\"tableblock halign-", cell.h_align.word()]);
    self.push([" class=\"halign-", cell.h_align.word()]);

    self.push([" valign-", cell.v_align.word()]);
    if cell.col_span > 1 {
      self.push(["\" colspan=\"", &num_str!(cell.col_span)]);
    }
    if cell.row_span > 1 {
      self.push(["\" rowspan=\"", &num_str!(cell.row_span)]);
    }
    self.push_str("\">");

    match &cell.content {
      CellContent::AsciiDoc(_) => {} // self.push_str("<div class=\"content\">"),
      CellContent::Literal(_) => {
        self.newlines = Newlines::Preserve;
        self.push_str("<div class=\"literal\"><pre>");
      }
      _ => {}
    }
  }

  pub(super) fn close_cell(&mut self, cell: &Cell, section: TableSection) {
    match (section, &cell.content) {
      (TableSection::Header, _) | (_, CellContent::Header(_)) => {
        if self.html.as_bytes().last() == Some(&b' ') {
          self.html.pop();
        }
        self.push_str("</th>");
      }
      (_, CellContent::Literal(_)) => {
        self.newlines = self.default_newlines;
        self.push_str("</pre></div></td>");
      }
      (_, CellContent::AsciiDoc(_)) => self.push_str("</td>"), // tablenew
      _ => self.push_str("</td>"),
    }
  }

  pub(super) fn open_cell_paragraph(&mut self, cell: &Cell, section: TableSection) {
    if cell.content.has_multiple_paras() {
      self.push_str("<p>");
    }
    match (section, &cell.content) {
      (TableSection::Header, _) => {}
      // tablenew changed all these
      (_, CellContent::Emphasis(_)) => self.push_str("<em>"),
      (_, CellContent::Monospace(_)) => self.push_str("<code>"),
      (_, CellContent::Strong(_)) => self.push_str("<strong>"),
      // _ => self.push_str("<p class=\"tableblock\">"), // tablenew
      _ => {} // tablenew
    }
  }

  pub(super) fn close_cell_paragraph(&mut self, cell: &Cell, section: TableSection) {
    match (section, &cell.content) {
      (TableSection::Header, _) => self.push_str(" "),
      // tablenew changed all these
      (_, CellContent::Emphasis(_)) => self.push_str("</em>"),
      (_, CellContent::Monospace(_)) => self.push_str("</code>"),
      (_, CellContent::Strong(_)) => self.push_str("</strong>"),
      // _ => self.push_str("</p>"),
      _ => {} // tablenew
    }
    if cell.content.has_multiple_paras() {
      self.push_str("</p>");
    }
  }
}
