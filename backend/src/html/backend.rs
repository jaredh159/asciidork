use std::collections::HashSet;
use std::{cell::RefCell, rc::Rc};

use roman_numerals_fn::to_roman_numeral;

use asciidork_core::{DocType, JobAttr, Path, SafeMode, file, iff};
use ast::{AttrValue, ReadAttr, SpecialSection, prelude::*};

use crate::{
  html::{HtmlBuf, OpenTag},
  utils,
};

#[derive(Debug, Default)]
pub struct BackendState {
  pub section_nums: [u16; 5],
  pub section_num_levels: isize,
  pub ephemeral: HashSet<EphemeralState>,
  pub appendix_caption_num: u8,
  pub book_part_num: usize,
  pub desc_list_depth: u8,
  pub interactive_list_stack: Vec<bool>,
  pub in_asciidoc_table_cell: bool,
  pub xref_depth: u8,
  #[allow(clippy::type_complexity)]
  pub footnotes: Rc<RefCell<Vec<(Option<String>, String)>>>,
}

pub trait HtmlBackend: HtmlBuf {
  fn state(&self) -> &BackendState;
  fn state_mut(&mut self) -> &mut BackendState;
  fn doc_meta(&self) -> &DocumentMeta;

  fn set_html_job_attrs(attrs: &mut asciidork_core::JobAttrs) {
    attrs.insert_unchecked("backend", JobAttr::readonly("html5"));
    attrs.insert_unchecked("backend-html5", JobAttr::readonly(true));
    attrs.insert_unchecked("basebackend", JobAttr::readonly("html"));
    attrs.insert_unchecked("basebackend-html", JobAttr::readonly(true));
  }

  fn open_doc_head(&mut self, meta: &DocumentMeta) {
    self.push_str(r#"<!DOCTYPE html><html"#);
    if !meta.is_true("nolang") {
      self.push([r#" lang=""#, meta.str_or("lang", "en"), "\""]);
    }
    self.push_str("><head>");
  }

  fn meta_tags(&mut self, meta: &DocumentMeta) {
    let encoding = meta.str_or("encoding", "UTF-8");
    self.push([r#"<meta charset=""#, encoding, r#"">"#]);
    self.push_str(r#"<meta http-equiv="X-UA-Compatible" content="IE=edge">"#);
    self.push_str(r#"<meta name="viewport" content="width=device-width, initial-scale=1.0">"#);
    if !meta.is_true("reproducible") {
      self.push_str(r#"<meta name="generator" content="Asciidork">"#);
    }
    if let Some(appname) = meta.str("app-name") {
      self.push([r#"<meta name="application-name" content=""#, appname, "\">"]);
    }
    if let Some(desc) = meta.str("description") {
      self.push([r#"<meta name="description" content=""#, desc, "\">"]);
    }
    if let Some(keywords) = meta.str("keywords") {
      self.push([r#"<meta name="keywords" content=""#, keywords, "\">"]);
    }
    if let Some(copyright) = meta.str("copyright") {
      self.push([r#"<meta name="copyright" content=""#, copyright, "\">"]);
    }
  }

  fn open_body(&mut self, document: &Document) {
    self.push_str("<body");
    if let Some(custom_id) = document.meta.str("css-signature") {
      self.push([r#" id=""#, custom_id, "\""]);
    }
    self.push([r#" class=""#, document.meta.get_doctype().to_str()]);
    match document.toc.as_ref().map(|toc| &toc.position) {
      Some(TocPosition::Left) => self.push_str(" toc2 toc-left"),
      Some(TocPosition::Right) => self.push_str(" toc2 toc-right"),
      _ => {}
    }
    self.push_str("\">");
  }

  fn enter_table_section(&mut self, section: TableSection) {
    match section {
      TableSection::Header => self.push_str("<thead>"),
      TableSection::Body => self.push_str("<tbody>"),
      TableSection::Footer => self.push_str("<tfoot>"),
    }
  }

  fn exit_table_section(&mut self, section: TableSection) {
    match section {
      TableSection::Header => self.push_str("</thead>"),
      TableSection::Body => self.push_str("</tbody>"),
      TableSection::Footer => self.push_str("</tfoot>"),
    }
  }

  fn visit_inline_specialchar(&mut self, char: &SpecialCharKind) {
    match char {
      SpecialCharKind::Ampersand => self.push_str("&amp;"),
      SpecialCharKind::LessThan => self.push_str("&lt;"),
      SpecialCharKind::GreaterThan => self.push_str("&gt;"),
    }
  }

  fn visit_curly_quote(&mut self, kind: CurlyKind) {
    match kind {
      CurlyKind::LeftDouble => self.push_str("&#8221;"),
      CurlyKind::RightDouble => self.push_str("&#8220;"),
      CurlyKind::LeftSingle => self.push_str("&#8216;"),
      CurlyKind::RightSingle => self.push_str("&#8217;"),
      CurlyKind::LegacyImplicitApostrophe => self.push_str("&#8217;"),
    }
  }

  fn enter_xref(&mut self, target: &SourceString, kind: XrefKind) {
    let state = self.state_mut();
    state.xref_depth += 1;
    if state.xref_depth == 1 {
      self.push([
        "<a href=\"",
        &utils::xref::href(target, self.doc_meta(), kind, true),
        "\">",
      ]);
    }
  }

  fn exit_xref(&mut self) {
    let state = self.state_mut();
    state.xref_depth -= 1;
    if state.xref_depth == 0 {
      self.push_str("</a>");
    }
  }

  fn enter_xref_text(&mut self, is_biblio: bool) {
    if is_biblio {
      self.push_ch('[');
    }
  }

  fn exit_xref_text(&mut self, is_biblio: bool) {
    if is_biblio {
      self.push_ch(']');
    }
  }
  fn section_nums(&mut self) -> &[u16; 5] {
    &self.state().section_nums
  }

  fn section_nums_mut(&mut self) -> &mut [u16; 5] {
    &mut self.state_mut().section_nums
  }

  fn section_num_levels(&self) -> isize {
    self.state().section_num_levels
  }

  fn ephemeral_state(&self) -> &HashSet<EphemeralState> {
    &self.state().ephemeral
  }

  fn ephemeral_state_mut(&mut self) -> &mut HashSet<EphemeralState> {
    &mut self.state_mut().ephemeral
  }

  fn appendix_caption_num(&self) -> u8 {
    self.state().appendix_caption_num
  }

  fn appendix_caption_num_mut(&mut self) -> &mut u8 {
    &mut self.state_mut().appendix_caption_num
  }

  fn book_part_num(&self) -> usize {
    self.state().book_part_num
  }

  fn book_part_num_mut(&mut self) -> &mut usize {
    &mut self.state_mut().book_part_num
  }

  fn on_toc_exit(&mut self) {
    let state = self.state_mut();
    state.section_nums = [0; 5];
    state.appendix_caption_num = 0;
    state.book_part_num = 0;
  }

  fn push_icon_uri(&mut self, name: &str, prefix: Option<&str>) {
    // PERF: we could work to prevent all these allocations w/ some caching
    // these might get rendered many times in a given document
    let icondir = self.doc_meta().string_or("iconsdir", "./images/icons");
    let ext = self.doc_meta().string_or("icontype", "png");
    self.push([&icondir, "/", prefix.unwrap_or(""), name, ".", &ext]);
  }

  fn render_image(&mut self, target: &str, attrs: &AttrList, img_kind: &ImageKind, is_block: bool) {
    match img_kind {
      ImageKind::Standard => {
        let format = attrs.named("format").or_else(|| file::ext(target));
        let is_svg = matches!(format, Some("svg" | "SVG"));
        if is_svg
          && attrs.has_option("interactive")
          && self.doc_meta().safe_mode != SafeMode::Secure
        {
          return self.render_interactive_svg(target, attrs);
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
      ImageKind::InlineSvg(Some(inline_svg)) => self.push_str(inline_svg),
      ImageKind::InlineSvg(None) => {
        self.push_str(r#"<span class="alt">"#);
        if let Some(alt) = attrs.named("alt").or_else(|| attrs.str_positional_at(0)) {
          self.push_str(alt);
        } else {
          let stem = asciidork_core::file::stem(target);
          self.push_str(&stem.replace(['_', '-'], " "));
        }
        self.push_str(r#"</span>"#);
      }
    }
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

  fn enter_link_macro(
    &mut self,
    target: &SourceString,
    attrs: Option<&AttrList>,
    scheme: Option<UrlScheme>,
    resolving_xref: bool,
    has_link_text: bool,
    blank_window_shorthand: bool,
  ) {
    if resolving_xref
      || self
        .state()
        .ephemeral
        .contains(&EphemeralState::InTableOfContents)
    {
      return;
    }
    let mut tag = if let Some(attrs) = attrs {
      OpenTag::new("a", attrs)
    } else {
      OpenTag::new("a", &NoAttrs)
    };
    tag.push_str(" href=\"");
    if matches!(scheme, Some(UrlScheme::Mailto)) {
      tag.push_str("mailto:");
    }
    tag.push_specialchar_escaped(target);
    tag.push_ch('"');

    if let Some(attrs) = attrs {
      tag.push_link_attrs(attrs, has_link_text, blank_window_shorthand);
    }

    if attrs.is_none() && (!has_link_text && !matches!(scheme, Some(UrlScheme::Mailto))) {
      tag.push_class("bare")
    }

    self.push_open_tag(tag);
  }

  fn exit_link_macro(&mut self, target: &SourceString, resolving_xref: bool, has_link_text: bool) {
    if resolving_xref
      || self
        .state()
        .ephemeral
        .contains(&EphemeralState::InTableOfContents)
    {
      return;
    }
    if has_link_text {
      self.push_str("</a>");
      return;
    }
    if self.doc_meta().is_true("hide-uri-scheme") {
      self.push_specialchar_escaped(file::remove_uri_scheme(target));
    } else {
      self.push_specialchar_escaped(target);
    }
    self.push_str("</a>");
  }

  fn enter_mailto_macro(
    &mut self,
    address: &SourceString,
    subject: Option<&SourceString>,
    body: Option<&SourceString>,
    attrs: Option<&AttrList>,
    has_link_text: bool,
  ) {
    let mut a_tag = if let Some(attrs) = attrs {
      OpenTag::new("a", attrs)
    } else {
      OpenTag::new("a", &NoAttrs)
    };
    a_tag.push([" href=\"mailto:", address]);
    if let Some(subject) = subject {
      a_tag.push_str("?subject=");
      a_tag.push_url_encoded(subject);
    }
    if let Some(body) = body {
      a_tag.push_str(iff!(subject.is_some(), "&amp;body=", "?body="));
      a_tag.push_url_encoded(body);
    }
    a_tag.push_ch('"');
    self.push_open_tag(a_tag);
    if !has_link_text {
      self.push_str(address);
    }
  }

  fn visit_menu_macro(&mut self, items: &[SourceString]) {
    let mut items = items.iter();
    self.push_str(r#"<span class="menuseq"><span class="menu">"#);
    self.push_str(items.next().unwrap());
    self.push_str("</span>");

    let last_idx = items.len() - 1;
    for (idx, item) in items.enumerate() {
      self.push_str(r#"&#160;&#9656;<span class=""#);
      if idx == last_idx {
        self.push(["menuitem\">", item, "</span>"]);
      } else {
        self.push(["submenu\">", item, "</span>"]);
      }
    }
    self.push_str("</span>");
  }

  fn prev_footnote_ref_num(&self, id: Option<&SourceString>) -> Option<String> {
    self
      .state()
      .footnotes
      .borrow()
      .iter()
      .enumerate()
      .filter(|(_, (prev, _))| {
        prev.is_some() && prev.as_ref().map(|s| s.as_str()) == id.map(|s| &**s)
      })
      .map(|(i, _)| (i + 1).to_string())
      .next()
  }

  fn push_appendix_caption(&mut self) {
    if let Some(appendix_caption) = self.doc_meta().string("appendix-caption") {
      self.push([&appendix_caption, " "]);
    }

    let letter = (self.state().appendix_caption_num + b'A') as char;
    self.push_ch(letter);
    self.state_mut().appendix_caption_num += 1;

    if self.doc_meta().is_false("appendix-caption") {
      self.push_str(". ");
    } else {
      self.push_str(": ");
    }
  }

  fn push_section_number_prefix(&mut self, level: u8) {
    debug_assert!(level > 0 && level < 6);
    let appendix = self.state().ephemeral.contains(&EphemeralState::InAppendix);
    let mut sect_nums = self.state().section_nums;

    let level_idx = (level - 1) as usize;
    sect_nums[level_idx] += 1;
    sect_nums
      .iter_mut()
      .skip(level_idx + 1)
      .for_each(|n| *n = 0);

    // for idx in 0..=level_idx {
    for (idx, sect_num) in sect_nums.iter().enumerate().take(level_idx + 1) {
      if appendix && idx == 0 {
        // self.push_ch((b'A' + sect_nums[idx] as u8) as char);
        self.push_ch((b'A' + *sect_num as u8) as char);
      } else {
        // self.push_str(&sect_nums[idx].to_string());
        self.push_str(&sect_num.to_string());
      }
      self.push_ch('.');
    }
    self.push_ch(' ');

    self.state_mut().section_nums = sect_nums;
  }

  fn push_section_heading_prefix(&mut self, level: u8, special_sect: Option<SpecialSection>) {
    if self.should_number_section(level, special_sect) {
      if level == 1
        && self.doc_meta().get_doctype() == DocType::Book
        && let Some(chapter_signifier) = self.doc_meta().string("chapter-signifier")
      {
        self.push([&chapter_signifier, " "]);
      }
      self.push_section_number_prefix(level);
    }
  }

  fn should_number_section(&self, level: u8, special_sect: Option<SpecialSection>) -> bool {
    let Some(sectnums) = self.doc_meta().get("sectnums") else {
      return false;
    };
    if self.section_num_levels() < level as isize {
      return false;
    }
    match sectnums {
      AttrValue::String(val) if val == "all" => true,
      AttrValue::Bool(true) => {
        if let Some(special_sect) = special_sect {
          self
            .doc_meta()
            .get_doctype()
            .supports_special_section(special_sect)
        } else {
          true
        }
      }
      _ => false,
    }
  }

  fn push_part_prefix(&mut self) {
    if self.doc_meta().is_true("partnums") {
      let book_part_num = self.book_part_num_mut();
      *book_part_num += 1;
      let part_num = *book_part_num;
      let largest_representable_roman = 3999;
      if part_num <= largest_representable_roman {
        if let Some(part_signifier) = self.doc_meta().string("part-signifier") {
          self.push([&part_signifier, " "]);
        }
        let roman = &to_roman_numeral(part_num as u16).unwrap();
        self.push_str(roman);
        self.push_str(": ");
      }
    }
  }

  fn start_enter_ordered_list(&mut self, block: &Block, depth: u8) -> (&'static str, &'static str) {
    let non_dd_depth = depth - self.state().desc_list_depth;
    self.state_mut().interactive_list_stack.push(false);
    let custom = block.meta.attrs.ordered_list_custom_number_style();
    let list_type = custom
      .and_then(super::list::type_from_class)
      .unwrap_or_else(|| super::list::type_from_depth(non_dd_depth));
    let class = custom.unwrap_or_else(|| super::list::class_from_depth(non_dd_depth));
    (class, list_type)
  }

  fn finish_enter_ordered_list(
    &mut self,
    class: &str,
    list_type: &str,
    block: &Block,
    items: &[ListItem],
  ) {
    self.push([r#"<ol class=""#, class, "\""]);

    if list_type != "1" {
      self.push([" type=\"", list_type, "\""]);
    }

    if let Some(attr_start) = block.meta.attrs.named("start") {
      self.push([" start=\"", attr_start, "\""]);
    } else {
      match items[0].marker {
        ListMarker::Digits(1) => {}
        ListMarker::Digits(n) => {
          // TODO: asciidoctor documents that this is OK,
          // but it doesn't actually work, and emits a warning
          self.push([" start=\"", &crate::num_str!(n), "\""]);
        }
        _ => {}
      }
    }

    if block.meta.attrs.has_option("reversed") {
      self.push_str(" reversed>");
    } else {
      self.push_str(">");
    }
  }

  fn start_enter_unordered_list(&mut self, wrap: &str, block: &Block) -> (OpenTag, OpenTag) {
    let custom = block.meta.attrs.unordered_list_custom_marker_style();
    let interactive = block.meta.attrs.has_option("interactive");
    self.state_mut().interactive_list_stack.push(interactive);
    let mut div = OpenTag::new(wrap, &block.meta.attrs);
    let mut ul = OpenTag::new("ul", &NoAttrs);
    div.push_class("ulist");
    if self
      .state()
      .ephemeral
      .contains(&EphemeralState::InBibliography)
      || block.meta.attrs.special_sect() == Some(SpecialSection::Bibliography)
    {
      div.push_class("bibliography");
      ul.push_class("bibliography");
    }
    if let Some(custom) = custom {
      div.push_class(custom);
      ul.push_class(custom);
    }
    (div, ul)
  }

  fn push_callout_number_img(&mut self, num: u8) {
    let n_str = &crate::num_str!(num);
    self.push_str(r#"<img src=""#);
    self.push_icon_uri(n_str, Some("callouts/"));
    self.push([r#"" alt=""#, n_str, r#"">"#]);
  }

  fn push_enter_discrete_heading(&mut self, level: u8, id: Option<&str>, block: &Block) {
    let level_str = crate::num_str!(level + 1);
    if let Some(id) = id {
      self.push(["<h", &level_str, r#" id=""#, id, "\""]);
    } else {
      self.push(["<h", &level_str]);
    }
    self.push_str(r#" class="discrete"#);
    for role in block.meta.attrs.roles() {
      self.push_ch(' ');
      self.push_str(role);
    }
    self.push_str("\">");
  }

  fn push_exit_discrete_heading(&mut self, level: u8) {
    self.push(["</h", &crate::num_str!(level + 1), ">"]);
  }

  fn enter_toc_node(&mut self, node: &TocNode) {
    self.push_str("<li><a href=\"#");
    if let Some(id) = &node.id {
      self.push_str(id);
    }
    self.push_str("\">");
    if node.special_sect == Some(SpecialSection::Appendix) {
      self.state_mut().section_nums = [0; 5];
      self
        .state_mut()
        .ephemeral
        .insert(EphemeralState::InAppendix);
      self.push_appendix_caption();
    } else if node.level == 0 {
      self.push_part_prefix();
    } else {
      self.push_section_heading_prefix(node.level, node.special_sect);
    }
  }

  fn exit_toc_node(&mut self, node: &TocNode) {
    if node.special_sect == Some(SpecialSection::Appendix) {
      self.state_mut().section_nums = [0; 5];
      self
        .state_mut()
        .ephemeral
        .remove(&EphemeralState::InAppendix);
    }
    self.push_str("</li>");
  }

  fn enter_section_state(&mut self, section: &Section) {
    match section.meta.attrs.special_sect() {
      Some(SpecialSection::Appendix) => {
        self.state_mut().section_nums = [0; 5];
        self
          .state_mut()
          .ephemeral
          .insert(EphemeralState::InAppendix);
      }
      Some(SpecialSection::Bibliography) => {
        self
          .state_mut()
          .ephemeral
          .insert(EphemeralState::InBibliography);
      }
      _ => {}
    };
  }

  fn exit_section_state(&mut self, section: &Section) {
    match section.meta.attrs.special_sect() {
      Some(SpecialSection::Appendix) => {
        self.state_mut().section_nums = [0; 5];
        self
          .state_mut()
          .ephemeral
          .remove(&EphemeralState::InAppendix);
      }
      Some(SpecialSection::Bibliography) => {
        self
          .state_mut()
          .ephemeral
          .remove(&EphemeralState::InBibliography);
      }
      _ => {}
    };
  }

  fn enter_section_heading(&mut self, section: &Section, accessible: bool) {
    let level_str = crate::num_str!(section.level + 1);
    if let Some(id) = &section.id {
      self.push(["<h", &level_str, r#" id=""#, id, "\">"]);
      if self.doc_meta().is_true("sectanchors") {
        self.push([r##"<a class="anchor" href="#"##, id]);
        if accessible {
          self.push_str(r#"" aria-hidden="true"#);
        }
        self.push_str("\"></a>");
      }
      if self.doc_meta().is_true("sectlinks") {
        self.push([r##"<a class="link" href="#"##, id, "\">"]);
      }
    } else {
      self.push(["<h", &level_str, ">"]);
    }
    if section.meta.attrs.special_sect() == Some(SpecialSection::Appendix) {
      self.push_appendix_caption();
    } else {
      self.push_section_heading_prefix(section.level, section.meta.attrs.special_sect());
    }
  }

  fn exit_section_heading(&mut self, section: &Section) {
    let level_str = crate::num_str!(section.level + 1);
    if self.doc_meta().is_true("sectlinks") && section.id.is_some() {
      self.push_str("</a>");
    }
    self.push(["</h", &level_str, ">"]);
  }

  fn standalone(&self) -> bool {
    self.doc_meta().get_doctype() != DocType::Inline
      && !self.state().in_asciidoc_table_cell
      && !self.doc_meta().embedded
  }

  fn render_title(&mut self, document: &Document, attrs: &DocumentMeta) {
    self.push_str(r#"<title>"#);
    if let Some(title) = attrs.str("title") {
      self.push_str(title);
    } else if let Some(title) = document.title() {
      for s in title.main.plain_text() {
        self.push_str(s);
      }
    } else {
      self.push_str("Untitled");
    }
    self.push_str(r#"</title>"#);
  }

  fn render_embedded_stylesheet(&mut self, default_css: &str) {
    if self.doc_meta().str("stylesheet") == Some("") {
      self.push(["<style>", default_css, "</style>"]);
    } else if let Some(css) = self.doc_meta().string("_asciidork_resolved_custom_css") {
      self.push(["<style>", &css, "</style>"]);
    }
  }

  fn render_meta_authors(&mut self, authors: &[asciidork_core::Author]) {
    if authors.is_empty() {
      return;
    }
    self.push_str(r#"<meta name="author" content=""#);
    for (index, author) in authors.iter().enumerate() {
      if index > 0 {
        self.push_str(", ");
      }
      // TODO: escape/sanitize, w/ tests, see asciidoctor
      self.push_str(&author.fullname());
    }
    self.push_str(r#"">"#);
  }

  fn render_favicon(&mut self, meta: &DocumentMeta) {
    match meta.get("favicon") {
      Some(AttrValue::String(path)) => {
        let ext = crate::utils::file_ext(path).unwrap_or("ico");
        self.push_str(r#"<link rel="icon" type="image/"#);
        self.push([ext, r#"" href=""#, path, "\">"]);
      }
      Some(AttrValue::Bool(true)) => {
        self.push_str(r#"<link rel="icon" type="image/x-icon" href="favicon.ico">"#);
      }
      _ => {}
    }
  }

  fn render_doc_title(&self) -> bool {
    !self.doc_meta().is_true("noheader") && self.doc_meta().show_doc_title()
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EphemeralState {
  VisitingSimpleTermDescription,
  InTableOfContents,
  InHorizontalDescList,
  InQandaDescList,
  InDescListDesc,
  IsSourceBlock,
  InBibliography,
  InGlossaryList,
  InAppendix,
}
