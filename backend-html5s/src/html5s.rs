#![allow(dead_code)]
#![allow(unused_variables)]

use std::{cell::RefCell, rc::Rc};

use std::collections::HashSet;
use std::fmt::Write;

use crate::internal::*;
use EphemeralState::*;

#[derive(Debug, Default)]
pub struct Html5s {
  doc_meta: DocumentMeta,
  html: String,
  alt_html: String,
  in_source_block: bool,
  fig_caption_num: usize,
  table_caption_num: usize,
  example_caption_num: usize,
  listing_caption_num: usize,
  newlines: Newlines,
  default_newlines: Newlines,
  xref_depth: u8,
  section_level_stack: Vec<u8>,
  autogen_conum: u8,
  state: BackendState,
  // maybe move me into html;w
  #[allow(clippy::type_complexity)]
  footnotes: Rc<RefCell<Vec<(Option<String>, String)>>>,
  in_asciidoc_table_cell: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Newlines {
  #[default]
  Preserve,
  JoinWithBreak,
}

impl Backend for Html5s {
  type Output = String;
  type Error = std::convert::Infallible;
  const OUTFILESUFFIX: &'static str = ".html";

  fn doc_meta(&self) -> &DocumentMeta {
    &self.doc_meta
  }

  fn set_job_attrs(attrs: &mut asciidork_core::JobAttrs) {
    todo!()
  }

  fn enter_document(&mut self, document: &Document) {
    self.doc_meta = document.meta.clone();
    utils::set_backend_attrs::<Self>(&mut self.doc_meta);
    self.state.section_num_levels = document.meta.isize("sectnumlevels").unwrap_or(3);
  }

  fn exit_document(&mut self, document: &Document) {}

  fn enter_header(&mut self) {}

  fn exit_header(&mut self) {}

  fn enter_content(&mut self) {}

  fn exit_content(&mut self) {}

  fn enter_footer(&mut self) {
    if !self.footnotes.borrow().is_empty() && !self.in_asciidoc_table_cell {
      self.render_footnotes();
    }
  }

  fn exit_footer(&mut self) {}

  fn enter_toc(&mut self, toc: &TableOfContents, macro_block: Option<&Block>) {
    let id = &macro_block
      .and_then(|b| b.meta.attrs.id().map(|id| id.to_string()))
      .unwrap_or("toc".to_string());
    self.push([r#"<nav id=""#, id, r#"" class=""#]); // tocnew
    self.push_str(&self.doc_meta.string_or("toc-class", "toc"));
    self.push_str(r#"" role="doc-toc"#); // tocnew
    if matches!(toc.position, TocPosition::Left | TocPosition::Right) {
      self.push_ch('2'); // `toc2` roughly means "toc-aside", per dr src
    }
    let level = self.section_level_stack.last().copied().unwrap_or(0) + 2;
    self.push([r#""><h"#, &num_str!(level), r#" id=""#, id, r#"-title">"#]);
    self.push_str(&toc.title);
    self.push([r#"</h"#, &num_str!(level), ">"]);
  }

  fn exit_toc(&mut self, _toc: &TableOfContents) {
    self.push_str("</nav>"); // tocnew `nav`
    self.on_toc_exit();
  }

  fn enter_toc_level(&mut self, level: u8, _nodes: &[TocNode]) {
    self.push(["<ol class=\"toc-list level-", &num_str!(level), "\">"]);
  }

  fn exit_toc_level(&mut self, _level: u8, _nodes: &[TocNode]) {
    self.push_str("</ol>");
  }

  fn enter_toc_node(&mut self, node: &TocNode) {
    self.push_str("<li><a href=\"#");
    if let Some(id) = &node.id {
      self.push_str(id);
    }
    self.push_str("\">");
    if node.special_sect == Some(SpecialSection::Appendix) {
      self.state.section_nums = [0; 5];
      self.state.ephemeral.insert(InAppendix);
      self.push_appendix_caption();
    } else if node.level == 0 {
      self.push_part_prefix();
    } else {
      self.push_section_heading_prefix(node.level, node.special_sect);
    }
  }

  fn exit_toc_node(&mut self, node: &TocNode) {
    if node.special_sect == Some(SpecialSection::Appendix) {
      self.state.section_nums = [0; 5];
      self.state.ephemeral.remove(&InAppendix);
    }
    self.push_str("</li>");
  }

  fn exit_toc_content(&mut self) {
    self.push_str("</a>");
  }

  fn visit_document_attribute_decl(&mut self, name: &str, value: &AttrValue) {
    if name == "hardbreaks-option" {
      if value.is_true() {
        self.default_newlines = Newlines::JoinWithBreak;
        self.newlines = Newlines::JoinWithBreak;
      } else {
        self.default_newlines = Newlines::default();
        self.newlines = Newlines::default();
      }
    }
    _ = self.doc_meta.insert_doc_attr(name, value.clone());
  }

  fn enter_preamble(&mut self, doc_has_title: bool, blocks: &[Block]) {
    if doc_has_title {
      self.push_str(r#"<section id="preamble" aria-label="Preamble">"#);
    }
  }

  fn exit_preamble(&mut self, doc_has_title: bool, _blocks: &[Block]) {
    if doc_has_title {
      self.push_str("</section>");
    }
  }

  fn enter_document_title(&mut self) {
    if self.render_doc_title() {
      self.push_str("<h1>")
    } else {
      self.start_buffering();
    }
  }

  fn exit_document_title(&mut self) {
    if self.render_doc_title() {
      self.push_str("</h1>");
    } else {
      self.swap_take_buffer(); // discard
    }
    // self.render_document_authors();
  }

  fn enter_section(&mut self, section: &Section) {
    self.section_level_stack.push(section.level);
    let mut section_tag = OpenTag::without_id("section", &section.meta.attrs);
    section_tag.push_class("doc-section");
    section_tag.push_class(&format!("level-{}", section.level));
    // section_tag.push_class(backend::html::util::section_class(section));
    self.push_open_tag(section_tag);
    match section.meta.attrs.special_sect() {
      Some(SpecialSection::Appendix) => {
        self.state.section_nums = [0; 5];
        self.state.ephemeral.insert(InAppendix)
      }
      Some(SpecialSection::Bibliography) => self.state.ephemeral.insert(InBibliography),
      _ => true,
    };
  }

  fn exit_section(&mut self, section: &Section) {
    self.push_str("</section>");
    match section.meta.attrs.special_sect() {
      Some(SpecialSection::Appendix) => {
        self.state.section_nums = [0; 5];
        self.state.ephemeral.remove(&InAppendix)
      }
      Some(SpecialSection::Bibliography) => self.state.ephemeral.remove(&InBibliography),
      _ => true,
    };
    self.section_level_stack.pop();
  }

  fn enter_section_heading(&mut self, section: &Section) {
    let level_str = num_str!(section.level + 1);
    if let Some(id) = &section.id {
      self.push(["<h", &level_str, r#" id=""#, id, "\">"]);
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
    let level_str = num_str!(section.level + 1);
    self.push(["</h", &level_str, ">"]);
    if section.level == 1 {
      // self.push_str(r#"<div class="sectionbody">"#);
    }
  }

  fn enter_book_part(&mut self, part: &Part) {
    let mut section_tag = OpenTag::without_id("section", &part.title.meta.attrs);
    section_tag.push_class("doc-section");
    section_tag.push_class("level-0");
    // section_tag.push_class(&format!("level-{}", section.level));
    // section_tag.push_class(backend::html::util::section_class(section));
    self.push_open_tag(section_tag);
  }
  fn exit_book_part(&mut self, part: &Part) {
    self.push_str("</section>");
  }

  fn enter_book_part_title(&mut self, title: &PartTitle) {
    self.push_str("<h1");
    if let Some(id) = &title.id {
      self.push([r#" id=""#, id]); //, "\""]);
    }
    self.push_str("\">");
    self.push_part_prefix();
  }

  fn exit_book_part_title(&mut self, _title: &PartTitle) {
    self.push_str("</h1>");
  }

  fn enter_book_part_intro(&mut self, part: &Part) {
    if part.title.meta.title.is_some() {
      self.push_str(r#"<section class="open-block partintro">"#);
      self.push_str(r#"<h6 class="block-title">"#);
    } else {
      self.push_str(r#"<div class="open-block partintro">"#);
    }
  }

  fn exit_book_part_intro(&mut self, part: &Part) {
    if part.title.meta.title.is_some() {
      self.push_str("</section>");
    } else {
      self.push_str("</div>");
    }
  }

  fn enter_book_part_intro_content(&mut self, part: &Part) {
    if part.title.meta.title.is_some() {
      self.push_str("</h6>");
    }
    self.push_str(r#"<div class="content">"#);
  }

  fn exit_book_part_intro_content(&mut self, _part: &Part) {
    self.push_str("</div>");
  }

  fn enter_paragraph_block(&mut self, block: &Block) {
    if self.doc_meta.get_doctype() == DocType::Inline {
      return;
    }
    if block.has_title() {
      self.push_str(r#"<section class="paragraph">"#);
      self.render_buffered_block_title(block, true);
    }
    // if block.meta.attrs.special_sect() == Some(SpecialSection::Abstract) {
    //   self.push_str("<blockquote>");
    // } else {
    // self.push_str("<p>");
    // dbg!(&self.state.ephemeral);
    if !self
      .state
      .ephemeral
      .contains(&VisitingSimpleTermDescription)
    {
      self.open_element("p", &[], &block.meta.attrs);
    } else {
      eprintln!("foo bar");
    }
    // }
  }

  fn exit_paragraph_block(&mut self, block: &Block) {
    if self.doc_meta.get_doctype() == DocType::Inline {
      return;
    }
    // if block.meta.attrs.special_sect() == Some(SpecialSection::Abstract) {
    //   self.push_str("</blockquote>\n");
    // } else {
    if !self
      .state
      .ephemeral
      .contains(&VisitingSimpleTermDescription)
    {
      self.push_str("</p>");
    }
    if block.has_title() {
      self.push_str(r#"</section>"#);
    }
    // }
  }

  fn enter_sidebar_block(&mut self, block: &Block) {
    self.open_element("aside", &["sidebar"], &block.meta.attrs);
    self.render_buffered_block_title(block, true);
  }

  fn exit_sidebar_block(&mut self, block: &Block) {
    self.push_str("</aside>");
  }

  fn enter_open_block(&mut self, block: &Block) {
    // if block.meta.attrs.special_sect() == Some(SpecialSection::Abstract) {
    //   self.open_element("div", &["quoteblock abstract"], &block.meta.attrs);
    //   self.render_buffered_block_title(block, true);
    //   self.push_str(r#"<blockquote>"#);
    // } else {
    self.open_element("div", &["open-block"], &block.meta.attrs);
    self.render_buffered_block_title(block, true);
    self.push_str(r#"<div class="content">"#);
    // }
  }

  fn exit_open_block(&mut self, block: &Block) {
    // if block.meta.attrs.special_sect() == Some(SpecialSection::Abstract) {
    //   self.push_str("</blockquote></div>");
    // } else {
    self.push_str("</div></div>");
    // }
  }

  fn enter_example_block(&mut self, block: &Block) {
    if block.meta.attrs.has_option("collapsible") {
      self.open_element("details", &[], &block.meta.attrs);
      if block.meta.attrs.has_option("open") {
        self.html.pop();
        self.push_str(" open>");
      }
      if block.has_title() {
        self.push_str("<summary>");
        self.push_buffered();
        self.push_str("</summary>");
      }
      self.push_str(r#"<div class="content">"#);
    } else {
      let el = if block.has_title() { "figure" } else { "div" };
      self.open_element(el, &["example-block"], &block.meta.attrs);
      self.render_buffered_block_title(block, false);
      self.push_str(r#"<div class="example">"#);
    }
  }

  fn exit_example_block(&mut self, block: &Block) {
    if block.meta.attrs.has_option("collapsible") {
      self.push_str("</div></details>");
    } else if block.has_title() {
      self.push_str("</div></figure>");
    } else {
      self.push_str("</div></div>");
    }
  }

  fn enter_quote_block(&mut self, block: &Block, _has_attribution: bool) {
    let el = if block.has_title() { "section" } else { "div" };
    self.open_element(el, &["quote-block"], &block.meta.attrs);
    if block.has_title() {
      self.render_buffered_block_title(block, true);
    }
    self.push_str("<blockquote>");
  }

  fn exit_quote_block(&mut self, block: &Block, has_attribution: bool) {
    if block.context == BlockContext::Verse && !has_attribution {
      self.push_str("</pre>");
      // } else if !has_attribution {
      //   self.push_str("</blockquote>");
    }
    let el = if block.has_title() { "</section>" } else { "</div>" };
    self.push(["</blockquote>", el]);
  }

  fn enter_quote_cite(&mut self, _block: &Block, has_attribution: bool) {
    if has_attribution {
      self.push_str(r#", "#);
    } else {
      // self.push_str(r#"</blockquote><div class="attribution">&#8212; "#);
      self.push_str(r#"<footer>&#8212; <cite>"#);
    }
  }

  fn exit_quote_cite(&mut self, _block: &Block, has_attribution: bool) {
    // if has_attribution {
    self.push_str(r#"</cite></footer>"#);
    // } else {
    //   self.push_str("</div>");
    // }
  }

  fn enter_verse_block(&mut self, block: &Block, _has_attribution: bool) {
    self.open_element("div", &["verse-block"], &block.meta.attrs);
    self.render_buffered_block_title(block, false);
    self.push_str(r#"<blockquote class="verse"><pre class="verse">"#);
  }

  fn exit_verse_block(&mut self, block: &Block, has_attribution: bool) {
    self.exit_quote_block(block, has_attribution);
  }

  fn enter_listing_block(&mut self, block: &Block) {
    let el = if block.has_title() { "figure" } else { "div" };
    self.open_element(el, &["listing-block"], &block.meta.attrs);
    self.render_buffered_block_title(block, false);
    self.push_str("<pre");
    if let Some(lang) = self.source_lang(block) {
      self.push([
        r#" class="highlight"><code class="language-"#,
        &lang,
        r#"" data-lang=""#,
        &lang,
        r#"">"#,
      ]);
      // self.state.insert(IsSourceBlock);
      self.in_source_block = true;
    } else {
      self.push_ch('>');
    }
    // self.newlines = Newlines::Preserve;
  }

  fn exit_listing_block(&mut self, block: &Block) {
    // if self.state.remove(&IsSourceBlock) {
    if self.in_source_block {
      self.in_source_block = false;
      self.push_str("</code>");
    }
    self.push_str("</pre>");
    // dbg!(&block);
    let close = if block.has_title() { "</figure>" } else { "</div>" };
    self.push_str(close);
    // self.newlines = self.default_newlines;
  }

  fn enter_literal_block(&mut self, block: &Block) {
    let el = if block.has_title() { "section" } else { "div" };
    self.open_element(el, &["literal-block"], &block.meta.attrs);
    self.render_buffered_block_title(block, true);
    self.push_str(r#"<pre>"#);
    self.newlines = Newlines::Preserve;
  }

  fn exit_literal_block(&mut self, block: &Block) {
    self.push_str("</pre>");
    let end = if block.has_title() { "</section>" } else { "</div>" };
    self.push_str(end);
    self.newlines = self.default_newlines;
  }

  fn enter_passthrough_block(&mut self, block: &Block) {}
  fn exit_passthrough_block(&mut self, block: &Block) {}

  fn enter_image_block(&mut self, img_target: &SourceString, img_attrs: &AttrList, block: &Block) {
    let el = if block.has_title() { "figure" } else { "div" };
    let mut open_tag = OpenTag::new(el, &block.meta.attrs);
    open_tag.push_class("image-block");
    open_tag.push_opt_class(img_attrs.named("float"));
    open_tag.push_opt_prefixed_class(img_attrs.named("align"), Some("text-"));
    self.push_open_tag(open_tag);

    let mut has_link = false;
    if let Some(href) = &block
      .meta
      .attrs
      .named("link")
      .or_else(|| img_attrs.named("link"))
    {
      self.push([r#"<a class="image" href=""#, *href, r#"">"#]);
      has_link = true;
    }
    self.render_image(img_target, img_attrs, true);
    if has_link {
      self.push_str("</a>");
    }
  }

  fn exit_image_block(&mut self, img_target: &SourceString, img_attrs: &AttrList, block: &Block) {
    if let Some(title) = img_attrs.named("title") {
      self.render_block_title(title, block, false);
    } else if block.has_title() {
      let title = self.take_buffer();
      self.render_block_title(&title, block, false);
    }
    let el = if block.has_title() { "</figure>" } else { "</div>" };
    self.push_str(el);
  }

  fn enter_admonition_block(&mut self, kind: AdmonitionKind, block: &Block) {
    let classes = &["admonition-block", kind.lowercase_str()];
    self.open_element("aside", classes, &block.meta.attrs);
    self.html.pop();
    self.push_str(r#" role=""#);
    match kind {
      AdmonitionKind::Tip => self.push_str("doc-tip\">"),
      AdmonitionKind::Caution => todo!(),
      AdmonitionKind::Important => todo!(),
      AdmonitionKind::Note => self.push_str("note\">"),
      AdmonitionKind::Warning => todo!(),
    }
    self.push_str(r#"<h6 class="block-title"#);
    if block.meta.title.is_none() {
      self.push_str(r#" label-only"#);
    }
    self.push_str(r#""><span class="title-label">"#);
    self.push([kind.str(), ": </span>"]);
    if block.has_title() {
      self.render_buffered_block_title(block, false);
    }
    self.push_str("</h6>");
    if !matches!(block.content, BlockContent::Compound(_)) {
      self.push_str("<p>");
    }
    self.render_buffered_block_title(block, false);
  }

  fn exit_admonition_block(&mut self, _kind: AdmonitionKind, block: &Block) {
    if !matches!(block.content, BlockContent::Compound(_)) {
      self.push_str("</p>");
    }
    self.push_str(r#"</aside>"#);
  }

  fn enter_quote_attribution(&mut self, block: &Block, _has_cite: bool) {
    if block.context == BlockContext::QuotedParagraph {
      self.push_str("</p>");
    }
    self.push_str(r#"<footer>&#8212; <cite>"#);
  }

  fn exit_quote_attribution(&mut self, _block: &Block, has_cite: bool) {
    if !has_cite {
      self.push_str("</cite></footer>");
    }
  }

  fn enter_quoted_paragraph(&mut self, block: &Block) {
    self.open_element("div", &["quote-block"], &block.meta.attrs);
    self.render_buffered_block_title(block, false);
    self.push_str("<blockquote><p>");
  }

  fn exit_quoted_paragraph(&mut self, _block: &Block) {
    self.push_str("</blockquote></div>");
  }

  fn enter_discrete_heading(&mut self, level: u8, id: Option<&str>, block: &Block) {
    todo!()
  }

  fn exit_discrete_heading(&mut self, level: u8, id: Option<&str>, block: &Block) {
    todo!()
  }

  fn enter_unordered_list(&mut self, block: &Block, items: &[ListItem], depth: u8) {
    let el = if block.has_title() { "section" } else { "div" };
    let (wrap, mut list) = self.start_enter_unordered_list(el, block);
    if items.iter().any(ListItem::is_checklist) {
      list.push_class("task-list");
    }
    if depth == 1 {
      self.push_open_tag(wrap);
    }
    self.render_buffered_block_title(block, true);
    self.push_open_tag(list);
  }

  fn exit_unordered_list(&mut self, block: &Block, _items: &[ListItem], depth: u8) {
    self.state.interactive_list_stack.pop();
    self.push_str("</ul>");
    if depth == 1 {
      self.push_str(if block.has_title() { "</section>" } else { "</div>" });
    }
  }

  fn enter_ordered_list(&mut self, block: &Block, items: &[ListItem], depth: u8) {
    let (class, list_type) = self.start_enter_ordered_list(block, depth);
    if depth == 1 {
      let classes = &["olist", class];
      let el = if block.has_title() { "section" } else { "div" };
      self.open_element(el, classes, &block.meta.attrs);
    }
    self.render_buffered_block_title(block, true);
    self.finish_enter_ordered_list(class, list_type, block, items);
  }

  fn exit_ordered_list(&mut self, block: &Block, _items: &[ListItem], depth: u8) {
    self.state.interactive_list_stack.pop();
    self.push_str("</ol>");
    if depth == 1 {
      self.push_str(if block.has_title() { "</section>" } else { "</div>" });
    }
  }

  fn enter_callout_list(&mut self, block: &Block, items: &[ListItem], depth: u8) {
    self.autogen_conum = 1;
    // remove the </div> added by exit listing block
    self.html.truncate(self.html.len().saturating_sub(6));
    self.open_element("ol", &["callout-list arabic"], &block.meta.attrs);
    // self.push_str(if self.doc_meta.icon_mode() != IconMode::Text { "<table>" } else { "<ol>" });
  }

  fn exit_callout_list(&mut self, _block: &Block, _items: &[ListItem], _depth: u8) {
    // self.push_str(if self.doc_meta.icon_mode() != IconMode::Text {
    //   "</div>"
    // } else {
    // "</ol></div>"
    // });
    self.push_str("</ol></div>");
  }

  fn enter_description_list(&mut self, block: &Block, _items: &[ListItem], depth: u8) {
    self.state.desc_list_depth += 1;
    let mut tag = OpenTag::new("div", &block.meta.attrs);
    if depth == 1 {
      tag.push_class("dlist");
    }
    if block.meta.attrs.special_sect() == Some(SpecialSection::Glossary) {
      self.state.ephemeral.insert(InGlossaryList);
      tag.push_class("glossary");
    }
    if depth == 1 {
      self.push_open_tag(tag);
    }
    self.render_buffered_block_title(block, false);
    self.push_str("<dl");
    if block.meta.attrs.special_sect() == Some(SpecialSection::Glossary) {
      self.push_str(r#" class="glossary""#);
    }
    self.push_ch('>');
  }

  fn exit_description_list(&mut self, _block: &Block, _items: &[ListItem], depth: u8) {
    self.state.ephemeral.remove(&InGlossaryList);
    self.push_str("</dl>");
    if depth == 1 {
      self.push_str("</div>");
    }
    self.state.desc_list_depth -= 1;
  }

  fn enter_description_list_term(&mut self, _item: &ListItem) {
    // if self.state.ephemeral.contains(&InGlossaryList) {
    self.push_str(r#"<dt>"#);
    // } else {
    //   self.push_str(r#"<dt class="hdlist1">"#);
    // }
  }

  fn exit_description_list_term(&mut self, _item: &ListItem) {
    self.push_str("</dt>");
  }

  fn enter_description_list_description(&mut self, _item: &ListItem) {
    self.state.ephemeral.insert(InDescListDesc);
    self.push_str("<dd>");
  }

  fn exit_description_list_description(&mut self, _item: &ListItem) {
    self.state.ephemeral.remove(&InDescListDesc);
    self.push_str("</dd>");
  }

  fn enter_description_list_description_text(&mut self, _text: &Block, item: &ListItem) {
    let first = item.blocks.first();
    if first.is_none() || !matches!(first.unwrap().content, BlockContent::Simple { .. }) {
      self.state.ephemeral.insert(VisitingSimpleTermDescription);
    }
  }

  fn exit_description_list_description_text(&mut self, _text: &Block, _item: &ListItem) {
    self.state.ephemeral.remove(&VisitingSimpleTermDescription);
  }

  fn enter_description_list_description_block(&mut self, _block: &Block, _item: &ListItem) {}
  fn exit_description_list_description_block(&mut self, _block: &Block, _item: &ListItem) {}

  fn enter_list_item_principal(&mut self, item: &ListItem, list_variant: ListVariant) {
    if let ListItemTypeMeta::Checklist(checked, _) = &item.type_meta {
      self.push_str(r#"<li class="task-list-item"><input class="task-list-item-checkbox" type="checkbox" disabled"#);
      self.push_str(if *checked { " checked>" } else { ">" });
    // } else if list_variant != ListVariant::Callout || self.doc_meta.icon_mode() == IconMode::Text {
    //
    } else {
      self.push_str("<li>");
      // } else {
      //   self.push_str("<tr><td>");
      //   let n = item.marker.callout_num().unwrap_or(self.autogen_conum);
      //   self.autogen_conum = n + 1;
      //   if self.doc_meta.icon_mode() == IconMode::Font {
      //     self.push_callout_number_font(n);
      //   } else {
      //     self.push_callout_number_img(n);
      //   }
      //   self.push_str("</td><td>");
    }
    if item
      .blocks
      .first()
      .is_some_and(|b| !matches!(b.content, BlockContent::List { .. }))
    {
      self.push_str("<p>");
    }
  }

  fn exit_list_item_principal(&mut self, item: &ListItem, list_variant: ListVariant) {
    // if list_variant != ListVariant::Callout || self.doc_meta.icon_mode() == IconMode::Text {
    // self.push_str("</p>");
    // } else {
    //   self.push_str("</td>");
    // }
    // if !item.blocks.is_empty() {
    if item
      .blocks
      .first()
      .is_some_and(|b| !matches!(b.content, BlockContent::List { .. }))
    {
      self.push_str("</p>");
    }
  }

  fn enter_list_item_blocks(&mut self, blocks: &[Block], item: &ListItem, variant: ListVariant) {}

  fn exit_list_item_blocks(&mut self, blocks: &[Block], item: &ListItem, variant: ListVariant) {
    self.push_str("</li>");
  }

  fn enter_table(&mut self, table: &Table, block: &Block) {
    let el = if block.has_title() { "figure" } else { "div" };
    self.open_element(el, &["table-block"], &block.meta.attrs);
    self.render_buffered_block_title(block, false);
    let mut tag = OpenTag::new("table", &NoAttrs);
    backend::html::table::finish_open_table_tag(&mut tag, block, &self.doc_meta);
    self.push_open_tag(tag);
    backend::html::table::push_colgroup(&mut self.html, table, block);
  }

  fn exit_table(&mut self, _table: &Table, block: &Block) {
    self.push_str("</table>");
    self.push_str(if block.has_title() { "</figure>" } else { "</div>" });
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

  fn enter_table_row(&mut self, _row: &Row, _section: TableSection) {
    self.push_str("<tr>");
  }

  fn exit_table_row(&mut self, _row: &Row, _section: TableSection) {
    self.push_str("</tr>");
  }

  fn enter_table_cell(&mut self, cell: &Cell, section: TableSection) {
    backend::html::table::open_cell(&mut self.html, cell, &section, None);
    if matches!(cell.content, CellContent::Literal(_)) {
      self.newlines = Newlines::Preserve;
      self.push_str("<div class=\"literal\"><pre>");
    }
  }

  fn exit_table_cell(&mut self, cell: &Cell, section: TableSection) {
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
      (_, CellContent::AsciiDoc(_)) => self.push_str("</td>"),
      _ => self.push_str("</td>"),
    }
  }

  fn enter_cell_paragraph(&mut self, cell: &Cell, section: TableSection) {
    if cell.content.has_multiple_paras() {
      self.push_str("<p>");
    }
    match (section, &cell.content) {
      (_, CellContent::Emphasis(_)) => self.push_str("<em>"),
      (_, CellContent::Monospace(_)) => self.push_str("<code>"),
      (_, CellContent::Strong(_)) => self.push_str("<strong>"),
      _ => {} // tablenew
    }
  }

  fn exit_cell_paragraph(&mut self, cell: &Cell, section: TableSection) {
    match (section, &cell.content) {
      (TableSection::Header, _) => self.push_str(" "),
      (_, CellContent::Emphasis(_)) => self.push_str("</em>"),
      (_, CellContent::Monospace(_)) => self.push_str("</code>"),
      (_, CellContent::Strong(_)) => self.push_str("</strong>"),
      _ => {} // tablenew
    }
    if cell.content.has_multiple_paras() {
      self.push_str("</p>");
    }
  }

  fn asciidoc_table_cell_backend(&mut self) -> Self {
    Self {
      in_asciidoc_table_cell: true,
      footnotes: Rc::clone(&self.footnotes),
      ..Self::default()
    }
  }

  fn visit_asciidoc_table_cell_result(&mut self, cell_backend: Self) {
    self.in_asciidoc_table_cell = false;
    self.html.push_str(&cell_backend.into_result().unwrap());
  }

  fn enter_meta_title(&mut self) {
    self.start_buffering();
  }

  fn exit_meta_title(&mut self) {
    self.stop_buffering();
  }

  fn enter_simple_block_content(&mut self, block: &Block) {
    if block.context == BlockContext::BlockQuote {
      self.push_str("<p>"); // this is sketchy...
    }
    if block.meta.attrs.has_option("hardbreaks") {
      self.newlines = Newlines::JoinWithBreak;
    }
  }

  fn exit_simple_block_content(&mut self, block: &Block) {
    if block.context == BlockContext::BlockQuote {
      self.push_str("</p>");
    } else if block.context == BlockContext::Verse {
      self.push_str("</pre>");
    }
    self.newlines = self.default_newlines;
  }

  fn enter_compound_block_content(&mut self, children: &[Block], block: &Block) {}
  fn exit_compound_block_content(&mut self, children: &[Block], block: &Block) {}

  fn visit_thematic_break(&mut self, block: &Block) {
    self.open_element("hr", &[], &block.meta.attrs);
  }

  fn visit_page_break(&mut self, _block: &Block) {
    self.push_str(r#"<div role="doc-pagebreak" style="page-break-after: always;"></div>"#);
  }

  fn visit_inline_text(&mut self, text: &str) {
    self.push_str(text);
  }

  fn visit_joining_newline(&mut self) {
    match self.newlines {
      Newlines::JoinWithBreak => self.push_str("<br>\n"),
      Newlines::Preserve => self.push_str("\n"),
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

  fn visit_multichar_whitespace(&mut self, whitespace: &str) {
    self.push_str(whitespace);
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
    if resolving_xref {
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
    tag.push_str(target);
    tag.push_ch('"');

    if let Some(attrs) = attrs {
      tag.push_link_attrs(attrs, has_link_text, blank_window_shorthand);
    }

    if attrs.is_none() && (!has_link_text && !matches!(scheme, Some(UrlScheme::Mailto))) {
      tag.push_class("bare")
    }

    self.push_open_tag(tag);
  }

  // TODO: exactly same as other backend
  fn exit_link_macro(
    &mut self,
    target: &SourceString,
    _attrs: Option<&AttrList>,
    _scheme: Option<UrlScheme>,
    resolving_xref: bool,
    has_link_text: bool,
  ) {
    if resolving_xref {
      return;
    }
    if has_link_text {
      self.push_str("</a>");
      return;
    }
    if self.doc_meta.is_true("hide-uri-scheme") {
      self.push_str(file::remove_uri_scheme(target));
    } else {
      self.push_str(target);
    }
    self.push_str("</a>");
  }

  fn visit_button_macro(&mut self, text: &SourceString) {
    self.push([r#"<b class="button">"#, text, "</b>"])
  }

  fn visit_menu_macro(&mut self, items: &[SourceString]) {
    todo!()
  }

  fn visit_image_macro(&mut self, target: &SourceString, attrs: &AttrList) {
    todo!()
  }

  fn visit_icon_macro(&mut self, target: &SourceString, attrs: &AttrList) {
    todo!()
  }

  fn visit_callout(&mut self, callout: Callout) {
    if !self.html.ends_with(' ') {
      self.push_ch(' ');
    }
    match self.doc_meta.icon_mode() {
      IconMode::Image => self.push_callout_number_img(callout.number),
      IconMode::Font => self.push_callout_number_font(callout.number),
      // TODO: asciidoctor also handles special `guard` case
      //   elsif ::Array === (guard = node.attributes['guard'])
      //     %(&lt;!--<b class="conum">(#{node.text})</b>--&gt;)
      // @see https://github.com/asciidoctor/asciidoctor/issues/3319
      IconMode::Text => self.push([r##"<b class="conum">"##, &num_str!(callout.number), "</b>"]),
    }
  }

  fn visit_callout_tuck(&mut self, comment: &str) {
    if self.doc_meta.icon_mode() != IconMode::Font {
      self.push_str(comment);
    }
  }

  fn enter_inline_italic(&mut self) {
    self.push_str("<em>");
  }

  fn exit_inline_italic(&mut self) {
    self.push_str("</em>");
  }

  fn enter_inline_mono(&mut self) {
    self.push_str("<code>");
  }

  fn exit_inline_mono(&mut self) {
    self.push_str("</code>");
  }

  fn enter_inline_bold(&mut self) {
    self.push_str("<strong>");
  }

  fn exit_inline_bold(&mut self) {
    self.push_str("</strong>");
  }

  fn enter_inline_lit_mono(&mut self) {
    self.push_str("<code>");
  }

  fn exit_inline_lit_mono(&mut self) {
    self.push_str("</code>");
  }

  fn visit_inline_specialchar(&mut self, char: &SpecialCharKind) {
    match char {
      SpecialCharKind::Ampersand => self.push_str("&amp;"),
      SpecialCharKind::LessThan => self.push_str("&lt;"),
      SpecialCharKind::GreaterThan => self.push_str("&gt;"),
    }
  }

  fn enter_inline_passthrough(&mut self) {}
  fn exit_inline_passthrough(&mut self) {}

  fn enter_inline_highlight(&mut self) {
    self.push_str("<mark>");
  }

  fn exit_inline_highlight(&mut self) {
    self.push_str("</mark>");
  }

  fn enter_inline_subscript(&mut self) {
    self.push_str("<sub>");
  }

  fn exit_inline_subscript(&mut self) {
    self.push_str("</sub>");
  }

  fn enter_inline_superscript(&mut self) {
    self.push_str("<sup>");
  }

  fn exit_inline_superscript(&mut self) {
    self.push_str("</sup>");
  }

  fn enter_inline_quote(&mut self, kind: QuoteKind) {
    match kind {
      QuoteKind::Double => self.push_str("&#x201c;"),
      QuoteKind::Single => self.push_str("&#8216;"),
    }
  }

  fn exit_inline_quote(&mut self, kind: QuoteKind) {
    match kind {
      QuoteKind::Double => self.push_str("&#x201d;"),
      QuoteKind::Single => self.push_str("&#8217;"),
    }
  }

  fn enter_footnote(&mut self, id: Option<&SourceString>, has_content: bool) {
    if has_content {
      self.start_buffering();
      return;
    }
    let prev_ref_num = self
      .footnotes
      .borrow()
      .iter()
      .enumerate()
      .filter(|(_, (prev, _))| {
        prev.is_some() && prev.as_ref().map(|s| s.as_str()) == id.map(|s| &**s)
      })
      .map(|(i, _)| (i + 1).to_string())
      .next();
    if let Some(prev_ref_num) = prev_ref_num {
      self.push([
        r##"<a class="footnote-ref" href="#_footnote_"##,
        &prev_ref_num,
        r#"" title="View footnote "#,
        &prev_ref_num,
        r#"" role="doc-noteref">["#,
        &prev_ref_num,
        r#"]</a>"#,
      ]);
    } else {
      // TODO: maybe warn?
    }
  }

  fn exit_footnote(&mut self, id: Option<&SourceString>, has_content: bool) {
    if !has_content {
      return; // this means the footnore was referring to a previously defined fn by id
    }
    let num = self.footnotes.borrow().len() + 1;
    let footnote = self.swap_take_buffer();
    let nums = num.to_string();
    self.push_str(r#"<a class="footnote-ref" id="_footnoteref_"#);
    // if let Some(id) = id {
    //   self.push_str(id);
    // } else {
    //   self.push_str(&nums);
    // }
    // self.push_str(if let Some(id) = id { id } else { &nums });
    self.push([&nums, r##"" href="#_footnote_"##, &nums]);
    self.push([
      r#"" title="View footnote "#,
      &nums,
      r#"" role="doc-noteref">"#,
    ]);
    self
      .footnotes
      .borrow_mut()
      .push((id.map(|id| id.to_string()), footnote));
    self.push(["[", &nums, "]</a>"]);
  }

  fn enter_text_span(&mut self, attrs: &AttrList) {
    self.open_element("span", &[], attrs);
  }

  fn exit_text_span(&mut self, attrs: &AttrList) {
    self.push_str("</span>");
  }

  fn enter_xref(&mut self, target: &SourceString, _has_reftext: bool, kind: XrefKind) {
    self.xref_depth += 1;
    if self.xref_depth == 1 {
      self.push([
        "<a href=\"",
        &utils::xref::href(target, &self.doc_meta, kind, true),
        "\">",
      ]);
    }
  }

  fn exit_xref(&mut self, _target: &SourceString, _has_reftext: bool, _kind: XrefKind) {
    self.xref_depth -= 1;
    if self.xref_depth == 0 {
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

  fn visit_missing_xref(
    &mut self,
    target: &SourceString,
    kind: XrefKind,
    doc_title: Option<&DocTitle>,
  ) {
    self.render_missing_xref(target, kind, doc_title);
  }

  fn visit_inline_anchor(&mut self, id: &str) {
    self.push(["<a id=\"", id, "\" aria-hidden=\"true\"></a>"]);
  }

  fn visit_biblio_anchor(&mut self, id: &str, reftext: Option<&str>) {
    self.push([
      "<a id=\"",
      id,
      "\" aria-hidden=\"true\"></a>[",
      reftext.unwrap_or(id),
      "]",
    ]);
  }

  fn visit_symbol(&mut self, kind: SymbolKind) {
    match kind {
      SymbolKind::Copyright => self.push_str("&#169;"),
      SymbolKind::Registered => self.push_str("&#174;"),
      SymbolKind::Trademark => self.push_str("&#8482;"),
      SymbolKind::EmDash => self.push_str("&#8211;&#8203;"),
      SymbolKind::SpacedEmDash(_) => self.push_str("&#8201;&#8211;&#8201;"),
      SymbolKind::Ellipsis => self.push_str("&#8230;&#8203;"),
      SymbolKind::SingleRightArrow => self.push_str("&#8594;"),
      SymbolKind::DoubleRightArrow => self.push_str("&#8658;"),
      SymbolKind::SingleLeftArrow => self.push_str("&#8592;"),
      SymbolKind::DoubleLeftArrow => self.push_str("&#8656;"),
    }
  }

  fn visit_linebreak(&mut self) {
    self.push_str("<br>\n");
  }

  fn into_result(self) -> Result<Self::Output, Self::Error> {
    Ok(self.html)
  }

  fn result(&self) -> Result<&Self::Output, Self::Error> {
    Ok(&self.html)
  }
}

impl Html5s {
  pub fn new() -> Self {
    Self::default()
  }

  fn source_lang<'a>(&self, block: &'a Block) -> Option<Cow<'a, str>> {
    match (
      block.meta.attrs.str_positional_at(0),
      block.meta.attrs.str_positional_at(1),
    ) {
      (None | Some("source"), Some(lang)) => Some(Cow::Borrowed(lang)),
      _ => self
        .doc_meta
        .str("source-language")
        .map(|s| Cow::Owned(s.to_string())),
    }
  }

  fn render_buffered_block_title(&mut self, block: &Block, wrap_in_h6: bool) {
    if block.has_title() {
      let buf = self.take_buffer();
      self.render_block_title(&buf, block, wrap_in_h6);
    }
  }

  fn render_block_title(&mut self, title: &str, block: &Block, wrap_in_h6: bool) {
    if wrap_in_h6 {
      self.push_str(r#"<h6 class="block-title">"#);
    }
    let (open, close) = match block.context {
      BlockContext::Table => ("<figcaption>", "</figcaption>"),
      BlockContext::Image => ("<figcaption>", "</figcaption>"),
      BlockContext::Example => ("<figcaption>", "</figcaption>"),
      BlockContext::Listing => ("<figcaption>", "</figcaption>"),
      _ => ("", ""),
    };
    if let Some(custom_caption) = block.meta.attrs.named("caption") {
      self.push([open, custom_caption, title, close]);
    } else if let Some(caption) = block
      .context
      .caption_attr_name()
      .and_then(|attr_name| self.doc_meta.string(attr_name))
    {
      self.push([open, &caption, " "]);
      let num = match block.context {
        BlockContext::Table => incr(&mut self.table_caption_num),
        BlockContext::Image => incr(&mut self.fig_caption_num),
        BlockContext::Example => incr(&mut self.example_caption_num),
        BlockContext::Listing => incr(&mut self.listing_caption_num),
        _ => unreachable!(),
      };
      self.push([&num.to_string(), ". ", title, close]);
    } else {
      self.push([open, title, close]);
    }
    if wrap_in_h6 {
      self.push_str("</h6>");
    }
  }

  fn push_admonition_img(&mut self, kind: AdmonitionKind) {
    self.push_str(r#"<img src=""#);
    self.push_icon_uri(kind.lowercase_str(), None);
    self.push([r#"" alt=""#, kind.str(), r#"">"#]);
  }

  fn push_callout_number_font(&mut self, num: u8) {
    let n_str = &num_str!(num);
    // self.push([r#"<i class="conum" data-value=""#, n_str, r#""></i>"#]);
    self.push([r#"<b class="conum">"#, n_str, "</b>"]);
  }

  fn render_checklist_item(&mut self, item: &ListItem) {
    if let ListItemTypeMeta::Checklist(checked, _) = &item.type_meta {
      match (self.state.interactive_list_stack.last() == Some(&true), checked) {
        (false, true) => self.push_str(r#" class="task-list-item"><input class="task-list-item-checkbox" type="checkbox" disabled checked>"#),
        (false, false) => self.push_str("&#10063;"),
        (true, true) => self.push_str(r#"<input type="checkbox" data-item-complete="1" checked>"#),
        (true, false) => self.push_str(r#"<input type="checkbox" data-item-complete="0">"#),
      }
    }
  }

  fn render_footnotes(&mut self) {
    self.push_str(r#"<section class="footnotes" aria-label="Footnotes" role="doc-endnotes">"#);
    self.push_str(r#"<hr><ol class="footnotes">"#);
    let footnotes = mem::take(&mut self.footnotes);
    for (i, (_, footnote)) in footnotes.borrow().iter().enumerate() {
      let num = (i + 1).to_string();
      self.push_str(r#"<li class="footnote" id="_footnote_"#);
      self.push([
        &num,
        r##"" role="doc-endnote">"##,
        footnote,
        r##" <a class="footnote-backref" href="#_footnoteref_"##,
        &num,
        r#"" role="doc-backlink" title="Jump to the first occurrence in the text">&#8617;</a></li>"#,
      ]);
    }
    self.push_str(r#"</ol></section>"#);
    self.footnotes = footnotes;
  }
}

impl HtmlBuf for Html5s {
  fn htmlbuf(&mut self) -> &mut String {
    &mut self.html
  }
}

impl AltHtmlBuf for Html5s {
  fn alt_htmlbuf(&mut self) -> &mut String {
    &mut self.alt_html
  }
  fn buffers(&mut self) -> (&mut String, &mut String) {
    (&mut self.html, &mut self.alt_html)
  }
}

impl HtmlBackend for Html5s {
  fn state(&self) -> &BackendState {
    &self.state
  }
  fn state_mut(&mut self) -> &mut BackendState {
    &mut self.state
  }
}

const fn incr(num: &mut usize) -> usize {
  *num += 1;
  *num
}

pub(crate) use backend::num_str;
