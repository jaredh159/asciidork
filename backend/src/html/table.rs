use ast::prelude::*;

use crate::html::OpenTag;

use std::fmt::Write;

pub fn open_cell(
  html: &mut String,
  cell: &Cell,
  section: &TableSection,
  semantic_class: Option<&str>,
) {
  if matches!(section, TableSection::Header) || matches!(cell.content, CellContent::Header(_)) {
    html.push_str("<th class=\"");
  } else {
    html.push_str("<td class=\"");
  }
  if let Some(semantic_class) = semantic_class {
    html.push_str(semantic_class);
    html.push(' ');
  }
  html.push_str("halign-");
  html.push_str(cell.h_align.word());
  html.push_str(" valign-");
  html.push_str(cell.v_align.word());
  // html.push([" class=\"tableblock halign-", cell.h_align.word()]);
  // html.push([" valign-", cell.v_align.word()]);
  if cell.col_span > 1 {
    html.push_str(r#"" colspan=""#);
    html.push_str(&crate::num_str!(cell.col_span));
    //   html.push(["\" colspan=\"", &crate::num_str!(cell.col_span)]);
  }
  if cell.row_span > 1 {
    html.push_str(r#"" rowspan=""#);
    html.push_str(&crate::num_str!(cell.row_span));
    //   html.push(["\" rowspan=\"", &crate::num_str!(cell.row_span)]);
  }
  html.push_str("\">");
}

pub fn push_colgroup(html: &mut String, table: &Table, block: &Block) {
  html.push_str("<colgroup>");
  let autowidth = block.meta.attrs.has_option("autowidth");
  for width in table.col_widths.distribute() {
    html.push_str("<col");
    if !autowidth {
      if let DistributedColWidth::Percentage(width) = width {
        if width.fract() == 0.0 {
          write!(html, r#" style="width: {width}%;""#).unwrap();
        } else {
          let width_s = format!("{width:.4}");
          let width_s = width_s.trim_end_matches('0');
          write!(html, r#" style="width: {width_s}%;""#).unwrap();
        }
      }
    }
    html.push('>');
  }
  html.push_str("</colgroup>");
}

pub fn finish_open_table_tag(tag: &mut OpenTag, block: &Block, doc_meta: &DocumentMeta) {
  tag.push_resolved_attr_class(
    "frame",
    Some("all"),
    Some("table-frame"),
    Some("frame-"),
    &block.meta,
    &doc_meta,
  );

  tag.push_resolved_attr_class(
    "grid",
    Some("all"),
    Some("table-grid"),
    Some("grid-"),
    &block.meta,
    &doc_meta,
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
    &doc_meta,
  );

  if let Some(width) = explicit_width {
    tag.push_style(format!("width: {width}%;"));
  }
}
