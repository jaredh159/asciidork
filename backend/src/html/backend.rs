use roman_numerals_fn::to_roman_numeral;
use std::collections::HashSet;

use asciidork_core::{file, DocType, Path, SafeMode};
use ast::{prelude::*, AttrValue, ReadAttr, SpecialSection};

use crate::{
  html::{HtmlBuf, OpenTag},
  utils, Backend,
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
}

pub trait HtmlBackend: Backend + HtmlBuf {
  fn state(&self) -> &BackendState;
  fn state_mut(&mut self) -> &mut BackendState;

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

    for idx in 0..=level_idx {
      if appendix && idx == 0 {
        self.push_ch((b'A' + sect_nums[idx] as u8) as char);
      } else {
        self.push_str(&sect_nums[idx].to_string());
      }
      self.push_ch('.');
    }
    self.push_ch(' ');

    self.state_mut().section_nums = sect_nums;
  }

  fn push_section_heading_prefix(&mut self, level: u8, special_sect: Option<SpecialSection>) {
    if self.should_number_section(level, special_sect) {
      if level == 1 && self.doc_meta().get_doctype() == DocType::Book {
        if let Some(chapter_signifier) = self.doc_meta().string("chapter-signifier") {
          self.push([&chapter_signifier, " "]);
        }
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
    return (class, list_type);
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EphemeralState {
  VisitingSimpleTermDescription,
  InDescListDesc,
  IsSourceBlock,
  InBibliography,
  InGlossaryList,
  InAppendix,
}
