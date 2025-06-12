use std::collections::HashSet;
use std::fmt::Write;
use std::sync::Once;
use std::{cell::RefCell, rc::Rc};

use roman_numerals_fn::to_roman_numeral;
use tracing::instrument;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::{fmt, EnvFilter};

use crate::internal::*;
use utils::set_backend_attrs;
use EphemeralState::*;

#[derive(Debug, Default)]
pub struct AsciidoctorHtml {
  pub(crate) html: String,
  pub(crate) alt_html: String,
  #[allow(clippy::type_complexity)]
  pub(crate) footnotes: Rc<RefCell<Vec<(Option<String>, String)>>>,
  pub(crate) doc_meta: DocumentMeta,
  pub(crate) list_stack: Vec<bool>,
  pub(crate) default_newlines: Newlines,
  pub(crate) newlines: Newlines,
  pub(crate) state: HashSet<EphemeralState>,
  pub(crate) autogen_conum: u8,
  pub(crate) xref_depth: u8,
  pub(crate) in_asciidoc_table_cell: bool,
  pub(crate) section_nums: [u16; 5],
  pub(crate) section_num_levels: isize,
  pub(crate) fig_caption_num: usize,
  pub(crate) book_part_num: usize,
  pub(crate) table_caption_num: usize,
  pub(crate) example_caption_num: usize,
  pub(crate) listing_caption_num: usize,
  pub(crate) appendix_caption_num: u8,
}

impl Backend for AsciidoctorHtml {
  type Output = String;
  type Error = Infallible;
  const OUTFILESUFFIX: &'static str = ".html";

  fn set_job_attrs(attrs: &mut asciidork_core::JobAttrs) {
    attrs.insert_unchecked("backend", JobAttr::readonly("html5"));
    attrs.insert_unchecked("backend-html5", JobAttr::readonly(true));
    attrs.insert_unchecked("basebackend", JobAttr::readonly("html"));
    attrs.insert_unchecked("basebackend-html", JobAttr::readonly(true));
  }

  #[instrument(skip_all)]
  fn enter_document(&mut self, document: &Document) {
    #[cfg(debug_assertions)]
    configure_test_tracing();

    self.doc_meta = document.meta.clone();
    set_backend_attrs::<Self>(&mut self.doc_meta);
    self.section_num_levels = document.meta.isize("sectnumlevels").unwrap_or(3);
    if document.meta.is_true("hardbreaks-option") {
      self.default_newlines = Newlines::JoinWithBreak
    }

    if !self.standalone() {
      return;
    }
    self.push_str(r#"<!DOCTYPE html><html"#);
    if !document.meta.is_true("nolang") {
      self.push([r#" lang=""#, document.meta.str_or("lang", "en"), "\""]);
    }
    let encoding = document.meta.str_or("encoding", "UTF-8");
    self.push([r#"><head><meta charset=""#, encoding, r#"">"#]);
    self.push_str(r#"<meta http-equiv="X-UA-Compatible" content="IE=edge">"#);
    self.push_str(r#"<meta name="viewport" content="width=device-width, initial-scale=1.0">"#);
    if !document.meta.is_true("reproducible") {
      self.push_str(r#"<meta name="generator" content="Asciidork">"#);
    }
    if let Some(appname) = document.meta.str("app-name") {
      self.push([r#"<meta name="application-name" content=""#, appname, "\">"]);
    }
    if let Some(desc) = document.meta.str("description") {
      self.push([r#"<meta name="description" content=""#, desc, "\">"]);
    }
    if let Some(keywords) = document.meta.str("keywords") {
      self.push([r#"<meta name="keywords" content=""#, keywords, "\">"]);
    }
    if let Some(copyright) = document.meta.str("copyright") {
      self.push([r#"<meta name="copyright" content=""#, copyright, "\">"]);
    }
    self.render_favicon(&document.meta);
    self.render_authors(document.meta.authors());
    self.render_title(document, &document.meta);
    self.render_styles(&document.meta);

    self.push_str("</head><body");
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

  #[instrument(skip_all)]
  fn exit_document(&mut self, _document: &Document) {
    if self.standalone() {
      self.push_str("</body></html>");
    }
  }

  #[instrument(skip_all)]
  fn enter_header(&mut self) {
    if !self.doc_meta.embedded && !self.doc_meta.is_true("noheader") {
      self.render_division_start("header");
    }
  }

  #[instrument(skip_all)]
  fn exit_header(&mut self) {
    if !self.doc_meta.embedded && !self.doc_meta.is_true("noheader") {
      self.push_str("</div>")
    }
  }

  #[instrument(skip_all)]
  fn enter_content(&mut self) {
    if !self.doc_meta.embedded {
      self.render_division_start("content");
    }
  }

  #[instrument(skip_all)]
  fn exit_content(&mut self) {
    if !self.doc_meta.embedded {
      self.push_str("</div>")
    }
  }

  #[instrument(skip_all)]
  fn enter_footer(&mut self) {
    if !self.footnotes.borrow().is_empty() && !self.in_asciidoc_table_cell {
      self.render_footnotes();
    }
    if !self.doc_meta.embedded && !self.doc_meta.is_true("nofooter") {
      self.render_division_start("footer");
    }
  }

  #[instrument(skip_all)]
  fn exit_footer(&mut self) {
    if !self.doc_meta.embedded && !self.doc_meta.is_true("nofooter") {
      self.push_str("</div>")
    }
  }

  #[instrument(skip_all)]
  fn enter_document_title(&mut self, _nodes: &[InlineNode]) {
    if self.render_doc_title() {
      self.push_str("<h1>")
    } else {
      self.start_buffering();
    }
  }

  #[instrument(skip_all)]
  fn exit_document_title(&mut self, _nodes: &[InlineNode]) {
    if self.render_doc_title() {
      self.push_str("</h1>");
    } else {
      self.swap_take_buffer(); // discard
    }
    self.render_document_authors();
  }

  #[instrument(skip_all)]
  fn enter_toc(&mut self, toc: &TableOfContents, macro_block: Option<&Block>) {
    let id = &macro_block
      .and_then(|b| b.meta.attrs.id().map(|id| id.to_string()))
      .unwrap_or("toc".to_string());
    self.push([r#"<div id=""#, id, r#"" class=""#]);
    self.push_str(&self.doc_meta.string_or("toc-class", "toc"));
    if matches!(toc.position, TocPosition::Left | TocPosition::Right) {
      self.push_ch('2'); // `toc2` roughly means "toc-aside", per dr src
    }
    self.push([r#""><div id=""#, id, r#"title""#]);
    if macro_block.is_some() {
      self.push_str(r#" class="title""#);
    }
    self.push_ch('>');
    self.push_str(&toc.title);
    self.push_str("</div>");
  }

  #[instrument(skip_all)]
  fn exit_toc(&mut self, _toc: &TableOfContents) {
    self.push_str("</div>");
    self.appendix_caption_num = 0;
    self.section_nums = [0; 5];
    self.book_part_num = 0;
  }

  #[instrument(skip_all)]
  fn enter_toc_level(&mut self, level: u8, _nodes: &[TocNode]) {
    self.push(["<ul class=\"sectlevel", &num_str!(level), "\">"]);
  }

  #[instrument(skip_all)]
  fn exit_toc_level(&mut self, _level: u8, _nodes: &[TocNode]) {
    self.push_str("</ul>");
  }

  #[instrument(skip_all)]
  fn enter_toc_node(&mut self, node: &TocNode) {
    self.push_str("<li><a href=\"#");
    if let Some(id) = &node.id {
      self.push_str(id);
    }
    self.push_str("\">");
    if node.special_sect == Some(SpecialSection::Appendix) {
      self.section_nums = [0; 5];
      self.state.insert(InAppendix);
      self.push_appendix_caption();
    } else if node.level == 0 {
      self.push_part_prefix();
    } else {
      self.push_section_heading_prefix(node.level, node.special_sect);
    }
  }

  #[instrument(skip_all)]
  fn exit_toc_node(&mut self, node: &TocNode) {
    if node.special_sect == Some(SpecialSection::Appendix) {
      self.section_nums = [0; 5];
      self.state.remove(&InAppendix);
    }
    self.push_str("</li>");
  }

  #[instrument(skip_all)]
  fn exit_toc_content(&mut self, _content: &[InlineNode]) {
    self.push_str("</a>");
  }

  #[instrument(skip_all)]
  fn enter_book_part(&mut self, _part: &Part) {}

  #[instrument(skip_all)]
  fn exit_book_part(&mut self, _part: &Part) {}

  #[instrument(skip_all)]
  fn enter_book_part_title(&mut self, title: &PartTitle) {
    self.push_str("<h1");
    if let Some(id) = &title.id {
      self.push([r#" id=""#, id, "\""]);
    }
    self.push_str(r#" class="sect0"#);
    for role in title.meta.attrs.roles() {
      self.push([" ", role]);
    }
    self.push_str("\">");
    self.push_part_prefix();
  }

  #[instrument(skip_all)]
  fn exit_book_part_title(&mut self, _title: &PartTitle) {
    self.push_str("</h1>");
  }

  #[instrument(skip_all)]
  fn enter_book_part_intro(&mut self, part: &Part) {
    self.push_str(r#"<div class="openblock partintro">"#);
    if part.title.meta.title.is_some() {
      self.push_str(r#"<div class="title">"#);
    }
  }

  #[instrument(skip_all)]
  fn exit_book_part_intro(&mut self, _part: &Part) {
    self.push_str("</div>");
  }

  #[instrument(skip_all)]
  fn enter_book_part_intro_content(&mut self, part: &Part) {
    if part.title.meta.title.is_some() {
      self.push_str("</div>");
    }
    self.push_str(r#"<div class="content">"#);
  }

  #[instrument(skip_all)]
  fn exit_book_part_intro_content(&mut self, _part: &Part) {
    self.push_str("</div>");
  }

  #[instrument(skip_all)]
  fn enter_preamble(&mut self, doc_has_title: bool, _blocks: &[Block]) {
    if doc_has_title {
      self.push_str(r#"<div id="preamble"><div class="sectionbody">"#);
    }
  }

  #[instrument(skip_all)]
  fn exit_preamble(&mut self, doc_has_title: bool, _blocks: &[Block]) {
    if doc_has_title {
      self.push_str("</div></div>");
    }
  }

  #[instrument(skip_all)]
  fn enter_section(&mut self, section: &Section) {
    let mut section_tag = OpenTag::without_id("div", &section.meta.attrs);
    section_tag.push_class(section::class(section));
    self.push_open_tag(section_tag);
    match section.meta.attrs.special_sect() {
      Some(SpecialSection::Appendix) => {
        self.section_nums = [0; 5];
        self.state.insert(InAppendix)
      }
      Some(SpecialSection::Bibliography) => self.state.insert(InBibliography),
      _ => true,
    };
  }

  #[instrument(skip_all)]
  fn exit_section(&mut self, section: &Section) {
    if section.level == 1 {
      self.push_str("</div>");
    }
    self.push_str("</div>");
    match section.meta.attrs.special_sect() {
      Some(SpecialSection::Appendix) => {
        self.section_nums = [0; 5];
        self.state.remove(&InAppendix)
      }
      Some(SpecialSection::Bibliography) => self.state.remove(&InBibliography),
      _ => true,
    };
  }

  #[instrument(skip_all)]
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

  #[instrument(skip_all)]
  fn exit_section_heading(&mut self, section: &Section) {
    let level_str = num_str!(section.level + 1);
    self.push(["</h", &level_str, ">"]);
    if section.level == 1 {
      self.push_str(r#"<div class="sectionbody">"#);
    }
  }

  #[instrument(skip_all)]
  fn enter_meta_title(&mut self, _title: &[InlineNode]) {
    self.start_buffering();
  }

  #[instrument(skip_all)]
  fn exit_meta_title(&mut self, _title: &[InlineNode]) {
    self.stop_buffering();
  }

  #[instrument(skip_all)]
  fn enter_compound_block_content(&mut self, _children: &[Block], _block: &Block) {}
  #[instrument(skip_all)]
  fn exit_compound_block_content(&mut self, _children: &[Block], _block: &Block) {}

  #[instrument(skip_all)]
  fn enter_simple_block_content(&mut self, _children: &[InlineNode], block: &Block) {
    if block.context == BlockContext::Verse {
      self.newlines = Newlines::Preserve;
    } else if block.meta.attrs.has_option("hardbreaks") {
      self.newlines = Newlines::JoinWithBreak;
    }
  }

  #[instrument(skip_all)]
  fn exit_simple_block_content(&mut self, _children: &[InlineNode], _block: &Block) {
    self.newlines = self.default_newlines;
  }

  #[instrument(skip_all)]
  fn enter_sidebar_block(&mut self, block: &Block, _content: &BlockContent) {
    self.open_element("div", &["sidebarblock"], &block.meta.attrs);
    self.push_str(r#"<div class="content">"#);
  }

  #[instrument(skip_all)]
  fn exit_sidebar_block(&mut self, _block: &Block, _content: &BlockContent) {
    self.push_str("</div></div>");
  }

  #[instrument(skip_all)]
  fn enter_listing_block(&mut self, block: &Block, _content: &BlockContent) {
    self.open_element("div", &["listingblock"], &block.meta.attrs);
    self.render_buffered_block_title(block);
    self.push_str(r#"<div class="content"><pre"#);
    if let Some(lang) = self.source_lang(block) {
      self.push([
        r#" class="highlight"><code class="language-"#,
        &lang,
        r#"" data-lang=""#,
        &lang,
        r#"">"#,
      ]);
      self.state.insert(IsSourceBlock);
    } else {
      self.push_ch('>');
    }
    self.newlines = Newlines::Preserve;
  }

  #[instrument(skip_all)]
  fn exit_listing_block(&mut self, _block: &Block, _content: &BlockContent) {
    if self.state.remove(&IsSourceBlock) {
      self.push_str("</code>");
    }
    self.push_str("</pre></div></div>");
    self.newlines = self.default_newlines;
  }

  #[instrument(skip_all)]
  fn enter_literal_block(&mut self, block: &Block, _content: &BlockContent) {
    self.open_element("div", &["literalblock"], &block.meta.attrs);
    self.push_str(r#"<div class="content"><pre>"#);
    self.newlines = Newlines::Preserve;
  }

  #[instrument(skip_all)]
  fn exit_literal_block(&mut self, _block: &Block, _content: &BlockContent) {
    self.push_str("</pre></div></div>");
    self.newlines = self.default_newlines;
  }

  #[instrument(skip_all)]
  fn enter_passthrough_block(&mut self, _block: &Block, _content: &BlockContent) {}
  #[instrument(skip_all)]
  fn exit_passthrough_block(&mut self, _block: &Block, _content: &BlockContent) {}

  #[instrument(skip_all)]
  fn enter_quoted_paragraph(&mut self, block: &Block, _attr: &str, _cite: Option<&str>) {
    self.open_element("div", &["quoteblock"], &block.meta.attrs);
    self.render_buffered_block_title(block);
    self.push_str("<blockquote>");
  }

  #[instrument(skip_all)]
  fn exit_quoted_paragraph(&mut self, _block: &Block, attr: &str, cite: Option<&str>) {
    self.exit_attributed(BlockContext::BlockQuote, Some(attr), cite);
  }

  #[instrument(skip_all)]
  fn enter_quote_block(&mut self, block: &Block, _content: &BlockContent) {
    self.open_element("div", &["quoteblock"], &block.meta.attrs);
    self.render_buffered_block_title(block);
    self.push_str("<blockquote>");
  }

  #[instrument(skip_all)]
  fn exit_quote_block(&mut self, block: &Block, _content: &BlockContent) {
    self.exit_attributed(
      block.context,
      block.meta.attrs.str_positional_at(1),
      block.meta.attrs.str_positional_at(2),
    );
  }

  #[instrument(skip_all)]
  fn enter_verse_block(&mut self, block: &Block, _content: &BlockContent) {
    self.open_element("div", &["verseblock"], &block.meta.attrs);
    self.render_buffered_block_title(block);
    self.push_str(r#"<pre class="content">"#);
  }

  #[instrument(skip_all)]
  fn exit_verse_block(&mut self, block: &Block, content: &BlockContent) {
    self.exit_quote_block(block, content)
  }

  #[instrument(skip_all)]
  fn enter_example_block(&mut self, block: &Block, _content: &BlockContent) {
    if block.meta.attrs.has_option("collapsible") {
      self.open_element("details", &[], &block.meta.attrs);
      if block.meta.attrs.has_option("open") {
        self.html.pop();
        self.push_str(" open>");
      }
      self.push_str(r#"<summary class="title">"#);
      if block.meta.title.is_some() {
        self.push_buffered();
      } else {
        self.push_str("Details");
      }
      self.push_str("</summary>");
    } else {
      self.open_element("div", &["exampleblock"], &block.meta.attrs);
      self.render_buffered_block_title(block);
    }
    self.push_str(r#"<div class="content">"#);
  }

  #[instrument(skip_all)]
  fn exit_example_block(&mut self, block: &Block, _content: &BlockContent) {
    if block.meta.attrs.has_option("collapsible") {
      self.push_str("</div></details>");
    } else {
      self.push_str("</div></div>");
    }
  }

  #[instrument(skip_all)]
  fn enter_open_block(&mut self, block: &Block, _content: &BlockContent) {
    if block.meta.attrs.special_sect() == Some(SpecialSection::Abstract) {
      self.open_element("div", &["quoteblock abstract"], &block.meta.attrs);
      self.render_buffered_block_title(block);
      self.push_str(r#"<blockquote>"#);
    } else {
      self.open_element("div", &["openblock"], &block.meta.attrs);
      self.render_buffered_block_title(block);
      self.push_str(r#"<div class="content">"#);
    }
  }

  #[instrument(skip_all)]
  fn exit_open_block(&mut self, block: &Block, _content: &BlockContent) {
    if block.meta.attrs.special_sect() == Some(SpecialSection::Abstract) {
      self.push_str("</blockquote></div>");
    } else {
      self.push_str("</div></div>");
    }
  }

  #[instrument(skip_all)]
  fn enter_discrete_heading(&mut self, level: u8, id: Option<&str>, block: &Block) {
    let level_str = num_str!(level + 1);
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

  #[instrument(skip_all)]
  fn exit_discrete_heading(&mut self, level: u8, _id: Option<&str>, _block: &Block) {
    self.push(["</h", &num_str!(level + 1), ">"]);
  }

  #[instrument(skip_all)]
  fn enter_unordered_list(&mut self, block: &Block, items: &[ListItem], _depth: u8) {
    let custom = block.meta.attrs.unordered_list_custom_marker_style();
    let interactive = block.meta.attrs.has_option("interactive");
    self.list_stack.push(interactive);
    let mut div = OpenTag::new("div", &block.meta.attrs);
    let mut ul = OpenTag::new("ul", &NoAttrs);
    div.push_class("ulist");
    if self.state.contains(&InBibliography)
      || block.meta.attrs.special_sect() == Some(SpecialSection::Bibliography)
    {
      div.push_class("bibliography");
      ul.push_class("bibliography");
    }
    if let Some(custom) = custom {
      div.push_class(custom);
      ul.push_class(custom);
    }
    if items.iter().any(ListItem::is_checklist) {
      div.push_class("checklist");
      ul.push_class("checklist");
    }
    self.push_open_tag(div);
    self.render_buffered_block_title(block);
    self.push_open_tag(ul);
  }

  #[instrument(skip_all)]
  fn exit_unordered_list(&mut self, _block: &Block, _items: &[ListItem], _depth: u8) {
    self.list_stack.pop();
    self.push_str("</ul></div>");
  }

  #[instrument(skip_all)]
  fn enter_callout_list(&mut self, block: &Block, _items: &[ListItem], _depth: u8) {
    self.autogen_conum = 1;
    self.open_element("div", &["colist arabic"], &block.meta.attrs);
    self.push_str(if self.doc_meta.icon_mode() != IconMode::Text { "<table>" } else { "<ol>" });
  }

  #[instrument(skip_all)]
  fn exit_callout_list(&mut self, _block: &Block, _items: &[ListItem], _depth: u8) {
    self.push_str(if self.doc_meta.icon_mode() != IconMode::Text {
      "</table></div>"
    } else {
      "</ol></div>"
    });
  }

  #[instrument(skip_all)]
  fn enter_description_list(&mut self, block: &Block, _items: &[ListItem], _depth: u8) {
    if block.meta.attrs.special_sect() == Some(SpecialSection::Glossary) {
      self.state.insert(InGlossaryList);
      self.open_element("div", &["dlist", "glossary"], &block.meta.attrs);
    } else {
      self.open_element("div", &["dlist"], &block.meta.attrs);
    }
    self.render_buffered_block_title(block);
    self.push_str("<dl>");
  }

  #[instrument(skip_all)]
  fn exit_description_list(&mut self, _block: &Block, _items: &[ListItem], _depth: u8) {
    self.state.remove(&InGlossaryList);
    self.push_str("</dl></div>");
  }

  #[instrument(skip_all)]
  fn enter_description_list_term(&mut self, _term: &[InlineNode], _item: &ListItem) {
    if self.state.contains(&InGlossaryList) {
      self.push_str(r#"<dt>"#);
    } else {
      self.push_str(r#"<dt class="hdlist1">"#);
    }
  }

  #[instrument(skip_all)]
  fn exit_description_list_term(&mut self, _term: &[InlineNode], _item: &ListItem) {
    self.push_str("</dt>");
  }

  #[instrument(skip_all)]
  fn enter_description_list_description(&mut self, _item: &ListItem) {
    self.push_str("<dd>");
  }

  #[instrument(skip_all)]
  fn exit_description_list_description(&mut self, _item: &ListItem) {
    self.push_str("</dd>");
  }

  #[instrument(skip_all)]
  fn enter_description_list_description_text(&mut self, _text: &Block, _item: &ListItem) {
    self.state.insert(VisitingSimpleTermDescription);
  }

  #[instrument(skip_all)]
  fn exit_description_list_description_text(&mut self, _text: &Block, _item: &ListItem) {
    self.state.remove(&VisitingSimpleTermDescription);
  }

  #[instrument(skip_all)]
  fn enter_description_list_description_block(&mut self, _block: &Block, _item: &ListItem) {}

  #[instrument(skip_all)]
  fn exit_description_list_description_block(&mut self, _block: &Block, _item: &ListItem) {}

  #[instrument(skip_all)]
  fn enter_ordered_list(&mut self, block: &Block, items: &[ListItem], depth: u8) {
    self.list_stack.push(false);
    let custom = block.meta.attrs.ordered_list_custom_number_style();
    let list_type = custom
      .and_then(list_type_from_class)
      .unwrap_or_else(|| list_type_from_depth(depth));
    let class = custom.unwrap_or_else(|| list_class_from_depth(depth));
    let classes = &["olist", class];
    self.open_element("div", classes, &block.meta.attrs);
    self.render_buffered_block_title(block);
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
          self.push([" start=\"", &num_str!(n), "\""]);
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

  #[instrument(skip_all)]
  fn exit_ordered_list(&mut self, _block: &Block, _items: &[ListItem], _depth: u8) {
    self.list_stack.pop();
    self.push_str("</ol></div>");
  }

  #[instrument(skip_all)]
  fn enter_list_item_principal(&mut self, item: &ListItem, list_variant: ListVariant) {
    if list_variant != ListVariant::Callout || self.doc_meta.icon_mode() == IconMode::Text {
      self.push_str("<li><p>");
      self.render_checklist_item(item);
    } else {
      self.push_str("<tr><td>");
      let n = item.marker.callout_num().unwrap_or(self.autogen_conum);
      self.autogen_conum = n + 1;
      if self.doc_meta.icon_mode() == IconMode::Font {
        self.push_callout_number_font(n);
      } else {
        self.push_callout_number_img(n);
      }
      self.push_str("</td><td>");
    }
  }

  #[instrument(skip_all)]
  fn exit_list_item_principal(&mut self, _item: &ListItem, list_variant: ListVariant) {
    if list_variant != ListVariant::Callout || self.doc_meta.icon_mode() == IconMode::Text {
      self.push_str("</p>");
    } else {
      self.push_str("</td>");
    }
  }

  #[instrument(skip_all)]
  fn enter_list_item_blocks(&mut self, _: &[Block], _: &ListItem, _: ListVariant) {}

  #[instrument(skip_all)]
  fn exit_list_item_blocks(&mut self, _blocks: &[Block], _items: &ListItem, variant: ListVariant) {
    if variant != ListVariant::Callout || self.doc_meta.icon_mode() == IconMode::Text {
      self.push_str("</li>");
    } else {
      self.push_str("</tr>");
    }
  }

  #[instrument(skip_all)]
  fn enter_paragraph_block(&mut self, block: &Block) {
    if self.doc_meta.get_doctype() == DocType::Inline {
      return;
    }
    if !self.state.contains(&VisitingSimpleTermDescription) {
      self.open_block_wrap(block);
      self.render_buffered_block_title(block);
    }
    self.open_block_content(block);
  }

  #[instrument(skip_all)]
  fn exit_paragraph_block(&mut self, block: &Block) {
    if self.doc_meta.get_doctype() == DocType::Inline {
      return;
    }
    self.close_block_content(block);
    if !self.state.contains(&VisitingSimpleTermDescription) {
      self.push_str("</div>");
    }
    self.state.remove(&VisitingSimpleTermDescription);
  }

  #[instrument(skip_all)]
  fn enter_table(&mut self, table: &Table, block: &Block) {
    self.open_table_element(block);
    self.render_buffered_block_title(block);
    self.push_str("<colgroup>");
    let autowidth = block.meta.attrs.has_option("autowidth");
    for width in table.col_widths.distribute() {
      self.push_str("<col");
      if !autowidth {
        if let DistributedColWidth::Percentage(width) = width {
          if width.fract() == 0.0 {
            write!(self.html, r#" style="width: {}%;""#, width).unwrap();
          } else {
            let width_s = format!("{:.4}", width);
            let width_s = width_s.trim_end_matches('0');
            write!(self.html, r#" style="width: {width_s}%;""#).unwrap();
          }
        }
      }
      self.push_ch('>');
    }
    self.push_str("</colgroup>");
  }

  fn exit_table(&mut self, _table: &Table, _block: &Block) {
    self.push_str("</table>");
  }

  fn asciidoc_table_cell_backend(&mut self) -> Self {
    Self {
      in_asciidoc_table_cell: true,
      footnotes: Rc::clone(&self.footnotes),
      ..Self::default()
    }
  }

  #[instrument(skip_all)]
  fn visit_asciidoc_table_cell_result(&mut self, cell_backend: Self) {
    self.in_asciidoc_table_cell = false;
    self.html.push_str(&cell_backend.into_result().unwrap());
  }

  #[instrument(skip_all)]
  fn enter_table_section(&mut self, section: TableSection) {
    match section {
      TableSection::Header => self.push_str("<thead>"),
      TableSection::Body => self.push_str("<tbody>"),
      TableSection::Footer => self.push_str("<tfoot>"),
    }
  }

  #[instrument(skip_all)]
  fn exit_table_section(&mut self, section: TableSection) {
    match section {
      TableSection::Header => self.push_str("</thead>"),
      TableSection::Body => self.push_str("</tbody>"),
      TableSection::Footer => self.push_str("</tfoot>"),
    }
  }

  #[instrument(skip_all)]
  fn enter_table_row(&mut self, _row: &Row, _section: TableSection) {
    self.push_str("<tr>");
  }

  #[instrument(skip_all)]
  fn exit_table_row(&mut self, _row: &Row, _section: TableSection) {
    self.push_str("</tr>");
  }

  #[instrument(skip_all)]
  fn enter_table_cell(&mut self, cell: &Cell, section: TableSection) {
    self.open_cell(cell, section);
  }

  #[instrument(skip_all)]
  fn exit_table_cell(&mut self, cell: &Cell, section: TableSection) {
    self.close_cell(cell, section);
  }

  #[instrument(skip_all)]
  fn enter_cell_paragraph(&mut self, cell: &Cell, section: TableSection) {
    self.open_cell_paragraph(cell, section);
  }

  #[instrument(skip_all)]
  fn exit_cell_paragraph(&mut self, cell: &Cell, section: TableSection) {
    self.close_cell_paragraph(cell, section);
  }

  #[instrument(skip_all)]
  fn enter_inline_italic(&mut self, _children: &[InlineNode]) {
    self.push_str("<em>");
  }

  #[instrument(skip_all)]
  fn exit_inline_italic(&mut self, _children: &[InlineNode]) {
    self.push_str("</em>");
  }

  #[instrument(skip_all)]
  fn visit_thematic_break(&mut self, block: &Block) {
    self.open_element("hr", &[], &block.meta.attrs);
  }

  #[instrument(skip_all)]
  fn visit_page_break(&mut self, _block: &Block) {
    self.push_str(r#"<div style="page-break-after: always;"></div>"#);
  }

  #[instrument(skip_all)]
  fn visit_inline_text(&mut self, text: &str) {
    self.push_str(text);
  }

  #[instrument(skip_all)]
  fn visit_joining_newline(&mut self) {
    match self.newlines {
      Newlines::JoinWithSpace => self.push_ch(' '),
      Newlines::JoinWithBreak => self.push_str("<br> "),
      Newlines::Preserve => self.push_str("\n"),
    }
  }

  #[instrument(skip_all)]
  fn enter_text_span(&mut self, attrs: &AttrList, _children: &[InlineNode]) {
    self.open_element("span", &[], attrs);
  }

  #[instrument(skip_all)]
  fn exit_text_span(&mut self, _attrs: &AttrList, _children: &[InlineNode]) {
    self.push_str("</span>");
  }

  #[instrument(skip_all)]
  fn enter_xref(&mut self, target: &str, _reftext: Option<&[InlineNode]>, kind: XrefKind) {
    self.xref_depth += 1;
    if self.xref_depth == 1 {
      self.push([
        "<a href=\"",
        &utils::xref::href(target, &self.doc_meta, kind, true),
        "\">",
      ]);
    }
  }

  #[instrument(skip_all)]
  fn exit_xref(&mut self, _target: &str, _reftext: Option<&[InlineNode]>, _kind: XrefKind) {
    self.xref_depth -= 1;
    if self.xref_depth == 0 {
      self.push_str("</a>");
    }
  }

  #[instrument(skip_all)]
  fn visit_missing_xref(&mut self, target: &str, kind: XrefKind, doc_title: Option<&DocTitle>) {
    // TODO: consider whether all this logic could be moved into backend::utils::xref
    // it's possible that other backends would want to do the exact same things
    if target == "#" || Some(target) == self.doc_meta.str("asciidork-docfilename") {
      let doctitle = doc_title
        .and_then(|t| t.attrs.named("reftext"))
        .unwrap_or_else(|| self.doc_meta.str("doctitle").unwrap_or("[^top]"))
        .to_string();
      self.push_str(&doctitle);
    } else if utils::xref::is_interdoc(target, kind) {
      let href = utils::xref::href(target, &self.doc_meta, kind, false);
      self.push_str(utils::xref::remove_leading_hash(&href));
    } else {
      self.push(["[", target.strip_prefix('#').unwrap_or(target), "]"]);
    }
  }

  #[instrument(skip_all)]
  fn visit_inline_anchor(&mut self, id: &str) {
    self.push(["<a id=\"", id, "\"></a>"]);
  }

  #[instrument(skip_all)]
  fn visit_biblio_anchor(&mut self, id: &str, reftext: Option<&str>) {
    self.push(["<a id=\"", id, "\"></a>[", reftext.unwrap_or(id), "]"]);
  }

  #[instrument(skip_all)]
  fn enter_xref_text(&mut self, _text: &[InlineNode], is_biblio: bool) {
    if is_biblio {
      self.push_str("[");
    }
  }

  #[instrument(skip_all)]
  fn exit_xref_text(&mut self, _text: &[InlineNode], is_biblio: bool) {
    if is_biblio {
      self.push_str("]");
    }
  }

  #[instrument(skip_all)]
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
      IconMode::Text => self.push([r#"<b class="conum">("#, &num_str!(callout.number), ")</b>"]),
    }
  }

  #[instrument(skip_all)]
  fn visit_callout_tuck(&mut self, comment: &str) {
    if self.doc_meta.icon_mode() != IconMode::Font {
      self.push_str(comment);
    }
  }

  #[instrument(skip_all)]
  fn visit_linebreak(&mut self) {
    self.push_str("<br> ");
  }

  #[instrument(skip_all)]
  fn enter_inline_mono(&mut self, _children: &[InlineNode]) {
    self.push_str("<code>");
  }

  #[instrument(skip_all)]
  fn exit_inline_mono(&mut self, _children: &[InlineNode]) {
    self.push_str("</code>");
  }

  #[instrument(skip_all)]
  fn enter_inline_bold(&mut self, _children: &[InlineNode]) {
    self.push_str("<strong>");
  }

  #[instrument(skip_all)]
  fn exit_inline_bold(&mut self, _children: &[InlineNode]) {
    self.push_str("</strong>");
  }

  #[instrument(skip_all)]
  fn enter_inline_passthrough(&mut self, _children: &[InlineNode]) {}
  #[instrument(skip_all)]
  fn exit_inline_passthrough(&mut self, _children: &[InlineNode]) {}

  #[instrument(skip_all)]
  fn visit_button_macro(&mut self, text: &str) {
    self.push([r#"<b class="button">"#, text, "</b>"])
  }

  #[instrument(skip_all)]
  fn visit_icon_macro(&mut self, target: &str, attrs: &AttrList) {
    self.push_str(r#"<span class="icon"#);
    attrs.roles.iter().for_each(|role| {
      self.push_str(" ");
      self.push_str(role);
    });
    self.push_str(r#"">"#);
    let has_link = if let Some(link) = attrs.named("link") {
      self.push_str(r#"<a class="image""#);
      self.push_html_attr("href", link);
      if let Some(window) = attrs.named("window") {
        self.push_html_attr("target", window);
      }
      self.push_str(">");
      true
    } else {
      false
    };
    match self.doc_meta.icon_mode() {
      IconMode::Text => {
        self.push_ch('[');
        self.push_str(attrs.named("alt").unwrap_or(target));
        self.push_str("&#93;");
      }
      IconMode::Image => {
        self.push_str(r#"<img src=""#);
        self.push_icon_uri(target, None);
        self.push_str(r#"" alt=""#);
        self.push_str(attrs.named("alt").unwrap_or(target));
        if let Some(width) = attrs.named("width") {
          self.push([r#"" width=""#, width]);
        }
        if let Some(title) = attrs.named("title") {
          self.push([r#"" title=""#, title]);
        }
        self.push_str(r#"">"#);
      }
      IconMode::Font => {
        self.push_str(r#"<i class="fa fa-"#);
        self.push_str(target);
        if let Some(size) = attrs.named("size").or(attrs.str_positional_at(0)) {
          self.push([r#" fa-"#, size]);
        }
        if let Some(flip) = attrs.named("flip") {
          self.push([r#" fa-flip-"#, flip]);
        }
        if let Some(rotate) = attrs.named("rotate") {
          self.push([r#" fa-rotate-"#, rotate]);
        }
        if let Some(title) = attrs.named("title") {
          self.push([r#"" title=""#, title]);
        }
        self.push_str(r#""></i>"#);
      }
    }
    if has_link {
      self.push_str("</a>");
    }
    self.push_str("</span>");
  }

  #[instrument(skip_all)]
  fn visit_image_macro(&mut self, target: &str, attrs: &AttrList) {
    let mut open_tag = OpenTag::new("span", &NoAttrs);
    open_tag.push_class("image");
    open_tag.push_opt_class(attrs.named("float"));
    open_tag.push_opt_prefixed_class(attrs.named("align"), Some("text-"));
    open_tag.push_classes(attrs.roles.iter());
    self.push_open_tag(open_tag);

    let with_link = if let Some(link_href) = attrs.named("link") {
      let mut a_tag = OpenTag::new("a", &NoAttrs);
      a_tag.push_class("image");
      a_tag.push_str("\" href=\"");
      if link_href == "self" {
        push_img_path(a_tag.htmlbuf(), target, &self.doc_meta);
      } else {
        a_tag.push_str_attr_escaped(link_href);
      }
      a_tag.push_ch('"');
      a_tag.opened_classes = false;
      a_tag.push_link_attrs(attrs, true, false);
      self.push_open_tag(a_tag);
      true
    } else {
      false
    };

    self.render_image(target, attrs, false);
    if with_link {
      self.push_str("</a>");
    }
    self.push_str("</span>");
  }

  #[instrument(skip_all)]
  fn visit_keyboard_macro(&mut self, keys: &[&str]) {
    if keys.len() > 1 {
      self.push_str(r#"<span class="keyseq">"#);
    }
    for (idx, key) in keys.iter().enumerate() {
      if idx > 0 {
        self.push_ch('+');
      }
      self.push(["<kbd>", key, "</kbd>"]);
    }
    if keys.len() > 1 {
      self.push_str("</span>");
    }
  }

  #[instrument(skip_all)]
  fn enter_link_macro(
    &mut self,
    target: &str,
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

  #[instrument(skip_all)]
  fn exit_link_macro(
    &mut self,
    target: &str,
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

  #[instrument(skip_all)]
  fn visit_menu_macro(&mut self, items: &[&str]) {
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

  #[instrument(skip_all)]
  fn visit_inline_specialchar(&mut self, char: &SpecialCharKind) {
    match char {
      SpecialCharKind::Ampersand => self.push_str("&amp;"),
      SpecialCharKind::LessThan => self.push_str("&lt;"),
      SpecialCharKind::GreaterThan => self.push_str("&gt;"),
    }
  }

  #[instrument(skip_all)]
  fn visit_symbol(&mut self, kind: SymbolKind) {
    match kind {
      SymbolKind::Copyright => self.push_str("&#169;"),
      SymbolKind::Registered => self.push_str("&#174;"),
      SymbolKind::Trademark => self.push_str("&#8482;"),
      SymbolKind::EmDash => self.push_str("&#8212;&#8203;"),
      SymbolKind::SpacedEmDash(_) => self.push_str("&#8201;&#8212;&#8201;"),
      SymbolKind::Ellipsis => self.push_str("&#8230;&#8203;"),
      SymbolKind::SingleRightArrow => self.push_str("&#8594;"),
      SymbolKind::DoubleRightArrow => self.push_str("&#8658;"),
      SymbolKind::SingleLeftArrow => self.push_str("&#8592;"),
      SymbolKind::DoubleLeftArrow => self.push_str("&#8656;"),
    }
  }

  #[instrument(skip_all)]
  fn enter_inline_highlight(&mut self, _children: &[InlineNode]) {
    self.push_str("<mark>");
  }

  #[instrument(skip_all)]
  fn exit_inline_highlight(&mut self, _children: &[InlineNode]) {
    self.push_str("</mark>");
  }

  #[instrument(skip_all)]
  fn enter_inline_subscript(&mut self, _children: &[InlineNode]) {
    self.push_str("<sub>");
  }

  #[instrument(skip_all)]
  fn exit_inline_subscript(&mut self, _children: &[InlineNode]) {
    self.push_str("</sub>");
  }

  #[instrument(skip_all)]
  fn enter_inline_superscript(&mut self, _children: &[InlineNode]) {
    self.push_str("<sup>");
  }

  #[instrument(skip_all)]
  fn exit_inline_superscript(&mut self, _children: &[InlineNode]) {
    self.push_str("</sup>");
  }

  #[instrument(skip_all)]
  fn enter_inline_quote(&mut self, kind: QuoteKind, _children: &[InlineNode]) {
    match kind {
      QuoteKind::Double => self.push_str("&#8220;"),
      QuoteKind::Single => self.push_str("&#8216;"),
    }
  }

  #[instrument(skip_all)]
  fn exit_inline_quote(&mut self, kind: QuoteKind, _children: &[InlineNode]) {
    match kind {
      QuoteKind::Double => self.push_str("&#8221;"),
      QuoteKind::Single => self.push_str("&#8217;"),
    }
  }

  #[instrument(skip_all)]
  fn visit_curly_quote(&mut self, kind: CurlyKind) {
    match kind {
      CurlyKind::LeftDouble => self.push_str("&#8221;"),
      CurlyKind::RightDouble => self.push_str("&#8220;"),
      CurlyKind::LeftSingle => self.push_str("&#8216;"),
      CurlyKind::RightSingle => self.push_str("&#8217;"),
      CurlyKind::LegacyImplicitApostrophe => self.push_str("&#8217;"),
    }
  }

  #[instrument(skip_all)]
  fn enter_inline_lit_mono(&mut self, _children: &[InlineNode]) {
    self.push_str("<code>");
  }

  #[instrument(skip_all)]
  fn exit_inline_lit_mono(&mut self, _children: &[InlineNode]) {
    self.push_str("</code>");
  }

  #[instrument(skip_all)]
  fn visit_multichar_whitespace(&mut self, _whitespace: &str) {
    self.push_ch(' ');
  }

  #[instrument(skip_all)]
  fn enter_admonition_block(&mut self, kind: AdmonitionKind, block: &Block) {
    let classes = &["admonitionblock", kind.lowercase_str()];
    self.open_element("div", classes, &block.meta.attrs);
    self.push_str(r#"<table><tr><td class="icon">"#);
    match self.doc_meta.icon_mode() {
      IconMode::Text => {
        self.push([r#"<div class="title">"#, kind.str()]);
        self.push_str(r#"</div></td><td class="content">"#);
      }
      IconMode::Image => {
        self.push_admonition_img(kind);
        self.push_str(r#"</td><td class="content">"#);
      }
      IconMode::Font => {
        self.push([r#"<i class="fa icon-"#, kind.lowercase_str(), "\" title=\""]);
        self.push([kind.str(), r#""></i></td><td class="content">"#]);
      }
    }
    self.render_buffered_block_title(block);
  }

  #[instrument(skip_all)]
  fn exit_admonition_block(&mut self, _kind: AdmonitionKind, _block: &Block) {
    self.push_str(r#"</td></tr></table></div>"#);
  }

  #[instrument(skip_all)]
  fn enter_image_block(&mut self, img_target: &str, img_attrs: &AttrList, block: &Block) {
    let mut open_tag = OpenTag::new("div", &block.meta.attrs);
    open_tag.push_class("imageblock");
    open_tag.push_opt_class(img_attrs.named("float"));
    open_tag.push_opt_prefixed_class(img_attrs.named("align"), Some("text-"));
    self.push_open_tag(open_tag);

    self.push_str(r#"<div class="content">"#);
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
    self.push_str(r#"</div>"#);
  }

  #[instrument(skip_all)]
  fn exit_image_block(&mut self, _target: &str, attrs: &AttrList, block: &Block) {
    if let Some(title) = attrs.named("title") {
      self.render_block_title(title, block);
    } else if block.meta.title.is_some() {
      let title = self.take_buffer();
      self.render_block_title(&title, block);
    }
    self.push_str(r#"</div>"#);
  }

  #[instrument(skip_all)]
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
    // TODO: consider warning?
    _ = self.doc_meta.insert_doc_attr(name, value.clone());
  }

  #[instrument(skip_all)]
  fn enter_footnote(&mut self, id: Option<&str>, content: Option<&[InlineNode]>) {
    if content.is_some() {
      self.start_buffering();
      return;
    }
    let prev_ref_num = self
      .footnotes
      .borrow()
      .iter()
      .enumerate()
      .filter(|(_, (prev, _))| prev.is_some() && prev.as_ref().map(|s| s.as_str()) == id)
      .map(|(i, _)| (i + 1).to_string())
      .next();
    if let Some(prev_ref_num) = prev_ref_num {
      self.push([
        r##"<sup class="footnoteref">[<a class="footnote" href="#_footnotedef_"##,
        &prev_ref_num,
        r#"" title="View footnote.">"#,
        &prev_ref_num,
        "</a>]</sup>",
      ]);
    } else {
      // TODO: maybe warn?
    }
  }

  #[instrument(skip_all)]
  fn exit_footnote(&mut self, id: Option<&str>, content: Option<&[InlineNode]>) {
    if content.is_none() {
      return; // this means the footnore was referring to a previously defined fn by id
    }
    let num = self.footnotes.borrow().len() + 1;
    let footnote = self.swap_take_buffer();
    let nums = num.to_string();
    self.push_str(r#"<sup class="footnote""#);
    if let Some(id) = id {
      self.push([r#" id="_footnote_"#, id, "\""]);
    }
    self.push_str(r#">[<a id="_footnoteref_"#);
    self.push([&nums, r##"" class="footnote" href="#_footnotedef_"##, &nums]);
    self.push([r#"" title="View footnote.">"#, &nums, "</a>]</sup>"]);
    self
      .footnotes
      .borrow_mut()
      .push((id.map(|id| id.to_string()), footnote));
  }

  fn into_result(self) -> Result<Self::Output, Self::Error> {
    Ok(self.html)
  }

  fn result(&self) -> Result<&Self::Output, Self::Error> {
    Ok(&self.html)
  }
}

impl HtmlBuf for AsciidoctorHtml {
  fn htmlbuf(&mut self) -> &mut String {
    &mut self.html
  }
}

impl AsciidoctorHtml {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn into_string(self) -> String {
    self.html
  }

  pub(crate) fn push_buffered(&mut self) {
    let buffer = self.take_buffer();
    self.push_str(&buffer);
  }

  pub(crate) fn push_appendix_caption(&mut self) {
    if let Some(appendix_caption) = self.doc_meta.string("appendix-caption") {
      self.push([&appendix_caption, " "]);
    }

    let letter = (self.appendix_caption_num + b'A') as char;
    self.push_ch(letter);
    self.appendix_caption_num += 1;

    if self.doc_meta.is_false("appendix-caption") {
      self.push_str(". ");
    } else {
      self.push_str(": ");
    }
  }

  fn take_buffer(&mut self) -> String {
    mem::take(&mut self.alt_html)
  }

  fn swap_take_buffer(&mut self) -> String {
    std::mem::swap(&mut self.alt_html, &mut self.html);
    std::mem::take(&mut self.alt_html)
  }

  pub(crate) fn push_open_tag(&mut self, tag: OpenTag) {
    self.push_str(&tag.finish());
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

  fn render_buffered_block_title(&mut self, block: &Block) {
    if block.meta.title.is_some() {
      let buf = self.take_buffer();
      self.render_block_title(&buf, block);
    }
  }

  fn render_block_title(&mut self, title: &str, block: &Block) {
    if block.context == BlockContext::Table {
      self.push_str(r#"<caption class="title">"#);
    } else {
      self.push_str(r#"<div class="title">"#);
    }
    if let Some(custom_caption) = block.meta.attrs.named("caption") {
      self.push_str(custom_caption);
    } else if let Some(caption) = block
      .context
      .caption_attr_name()
      .and_then(|attr_name| self.doc_meta.string(attr_name))
    {
      self.push_str(&caption);
      self.push_ch(' ');
      let num = match block.context {
        BlockContext::Table => incr(&mut self.table_caption_num),
        BlockContext::Image => incr(&mut self.fig_caption_num),
        BlockContext::Example => incr(&mut self.example_caption_num),
        BlockContext::Listing => incr(&mut self.listing_caption_num),
        _ => unreachable!(),
      };
      self.push_str(&num.to_string());
      self.push_str(". ");
    }
    self.push_str(title);
    if block.context == BlockContext::Table {
      self.push_str(r#"</caption>"#);
    } else {
      self.push_str(r#"</div>"#);
    }
  }

  pub(crate) fn open_element(&mut self, element: &str, classes: &[&str], attrs: &impl AttrData) {
    let mut open_tag = OpenTag::new(element, attrs);
    classes.iter().for_each(|c| open_tag.push_class(c));
    self.push_open_tag(open_tag);
  }

  pub fn open_block_wrap(&mut self, block: &Block) {
    let mut classes: &[&str] = &["paragraph"];
    if block.meta.attrs.special_sect() == Some(SpecialSection::Abstract) {
      classes = &["quoteblock", "abstract"]
    };
    self.open_element("div", classes, &block.meta.attrs);
  }

  pub fn open_block_content(&mut self, block: &Block) {
    if block.meta.attrs.special_sect() == Some(SpecialSection::Abstract) {
      self.push_str("<blockquote>");
    } else {
      self.push_str("<p>");
    }
  }

  pub fn close_block_content(&mut self, block: &Block) {
    if block.meta.attrs.special_sect() == Some(SpecialSection::Abstract) {
      self.push_str("</blockquote>");
    } else {
      self.push_str("</p>");
    }
  }

  fn render_footnotes(&mut self) {
    self.render_division_start("footnotes");
    self.push_str("<hr>");
    let footnotes = mem::take(&mut self.footnotes);
    for (i, (_, footnote)) in footnotes.borrow().iter().enumerate() {
      let num = (i + 1).to_string();
      self.push_str(r#"<div class="footnote" id="_footnotedef_"#);
      self.push([&num, r##""><a href="#_footnoteref_"##, &num, "\">"]);
      self.push([&num, "</a>. ", footnote, "</div>"]);
    }
    self.push_str(r#"</div>"#);
    self.footnotes = footnotes;
  }

  fn render_favicon(&mut self, meta: &DocumentMeta) {
    match meta.get("favicon") {
      Some(AttrValue::String(path)) => {
        let ext = helpers::file_ext(path).unwrap_or("ico");
        self.push_str(r#"<link rel="icon" type="image/"#);
        self.push([ext, r#"" href=""#, path, "\">"]);
      }
      Some(AttrValue::Bool(true)) => {
        self.push_str(r#"<link rel="icon" type="image/x-icon" href="favicon.ico">"#);
      }
      _ => {}
    }
  }

  fn render_authors(&mut self, authors: &[Author]) {
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

  fn render_styles(&mut self, meta: &DocumentMeta) {
    if meta.str("stylesheet") == Some("") {
      let family = match meta.str("webfonts") {
        None | Some("") => "Open+Sans:300,300italic,400,400italic,600,600italic%7CNoto+Serif:400,400italic,700,700italic%7CDroid+Sans+Mono:400,700",
        Some(custom) => custom,
      };
      self.push([
        r#"<link rel="stylesheet" href="https://fonts.googleapis.com/css?family="#,
        family,
        "\" />",
      ]);
    }

    if meta.str("stylesheet") == Some("") {
      self.push(["<style>", crate::css::DEFAULT, "</style>"]);
    } else if let Some(css) = meta.string("_asciidork_asciidoctor_resolved_css") {
      self.push(["<style>", &css, "</style>"]);
    }
  }

  fn exit_attributed(
    &mut self,
    context: BlockContext,
    attribution: Option<&str>,
    cite: Option<&str>,
  ) {
    if context == BlockContext::BlockQuote {
      self.push_str("</blockquote>");
    } else {
      self.push_str("</pre>");
    }
    if let Some(attribution) = attribution {
      self.push_str(r#"<div class="attribution">&#8212; "#);
      self.push_str(attribution);
      if let Some(cite) = cite {
        self.push_str(r#"<br><cite>"#);
        self.push([cite, "</cite>"]);
      }
      self.push_str("</div>");
    } else if let Some(cite) = cite {
      self.push_str(r#"<div class="attribution">&#8212; "#);
      self.push([cite, "</div>"]);
    }
    self.push_str("</div>");
  }

  fn render_checklist_item(&mut self, item: &ListItem) {
    if let ListItemTypeMeta::Checklist(checked, _) = &item.type_meta {
      match (self.list_stack.last() == Some(&true), checked) {
        (false, true) => self.push_str("&#10003;"),
        (false, false) => self.push_str("&#10063;"),
        (true, true) => self.push_str(r#"<input type="checkbox" data-item-complete="1" checked>"#),
        (true, false) => self.push_str(r#"<input type="checkbox" data-item-complete="0">"#),
      }
    }
  }

  const fn start_buffering(&mut self) {
    mem::swap(&mut self.html, &mut self.alt_html);
  }

  const fn stop_buffering(&mut self) {
    mem::swap(&mut self.html, &mut self.alt_html);
  }

  // TODO: handle embedding images, data-uri, etc., this is a naive impl
  // @see https://github.com/jaredh159/asciidork/issues/7
  fn push_icon_uri(&mut self, name: &str, prefix: Option<&str>) {
    // PERF: we could work to prevent all these allocations w/ some caching
    // these might get rendered many times in a given document
    let icondir = self.doc_meta.string_or("iconsdir", "./images/icons");
    let ext = self.doc_meta.string_or("icontype", "png");
    self.push([&icondir, "/", prefix.unwrap_or(""), name, ".", &ext]);
  }

  fn push_admonition_img(&mut self, kind: AdmonitionKind) {
    self.push_str(r#"<img src=""#);
    self.push_icon_uri(kind.lowercase_str(), None);
    self.push([r#"" alt=""#, kind.str(), r#"">"#]);
  }

  fn push_callout_number_img(&mut self, num: u8) {
    let n_str = &num_str!(num);
    self.push_str(r#"<img src=""#);
    self.push_icon_uri(n_str, Some("callouts/"));
    self.push([r#"" alt=""#, n_str, r#"">"#]);
  }

  fn push_callout_number_font(&mut self, num: u8) {
    let n_str = &num_str!(num);
    self.push([r#"<i class="conum" data-value=""#, n_str, r#""></i>"#]);
    self.push([r#"<b>("#, n_str, ")</b>"]);
  }

  fn render_document_authors(&mut self) {
    let authors = self.doc_meta.authors();
    if self.doc_meta.embedded || authors.is_empty() {
      return;
    }
    let mut buffer = String::with_capacity(authors.len() * 100);
    buffer.push_str(r#"<div class="details">"#);
    for (idx, author) in authors.iter().enumerate() {
      buffer.push_str(r#"<span id="author"#);
      if idx > 0 {
        buffer.push_str(&num_str!(idx + 1));
      }
      buffer.push_str(r#"" class="author">"#);
      buffer.push_str(&author.fullname());
      buffer.push_str(r#"</span><br>"#);
      if let Some(email) = &author.email {
        buffer.push_str(r#"<span id="email"#);
        if idx > 0 {
          buffer.push_str(&num_str!(idx + 1));
        }
        buffer.push_str(r#"" class="email"><a href="mailto:"#);
        buffer.push_str(email);
        buffer.push_str(r#"">"#);
        buffer.push_str(email);
        buffer.push_str(r#"</a></span><br>"#);
      }
    }
    self.push([&buffer, "</div>"]);
  }

  fn standalone(&self) -> bool {
    self.doc_meta.get_doctype() != DocType::Inline
      && !self.in_asciidoc_table_cell
      && !self.doc_meta.embedded
  }

  fn render_doc_title(&self) -> bool {
    !self.doc_meta.is_true("noheader") && self.doc_meta.show_doc_title()
  }

  fn render_division_start(&mut self, id: &str) {
    self.push([r#"<div id=""#, id, "\""]);
    if let Some(max_width) = self.doc_meta.string("max-width") {
      self.push([r#" style="max-width: "#, &max_width, r#";">"#]);
    } else {
      self.push_str(">");
    }
  }

  fn render_interactive_svg(&mut self, target: &str, attrs: &AttrList) {
    self.push_str(r#"<object type="image/svg+xml" data=""#);
    push_img_path(&mut self.html, target, &self.doc_meta);
    self.push_ch('"');
    self.push_named_or_pos_attr("width", 1, attrs);
    self.push_named_or_pos_attr("height", 2, attrs);
    self.push_ch('>');
    if let Some(fallback) = attrs.named("fallback") {
      self.push_str(r#"<img src=""#);
      push_img_path(&mut self.html, fallback, &self.doc_meta);
      self.push_ch('"');
      self.push_named_or_pos_attr("alt", 0, attrs);
      self.push_ch('>');
    } else if let Some(alt) = attrs.named("alt").or_else(|| attrs.str_positional_at(0)) {
      self.push([r#"<span class="alt">"#, alt, "</span>"]);
    }
    self.push_str("</object>");
  }

  fn render_image(&mut self, target: &str, attrs: &AttrList, is_block: bool) {
    let format = attrs.named("format").or_else(|| file::ext(target));
    let is_svg = matches!(format, Some("svg" | "SVG"));
    if is_svg && attrs.has_option("interactive") && self.doc_meta.safe_mode != SafeMode::Secure {
      return self.render_interactive_svg(target, attrs);
    }
    self.push_str(r#"<img src=""#);
    push_img_path(&mut self.html, target, &self.doc_meta);
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

  fn push_part_prefix(&mut self) {
    if self.doc_meta.is_true("partnums") {
      let part_num = incr(&mut self.book_part_num);
      if part_num <= 3999 {
        if let Some(part_signifier) = self.doc_meta.string("part-signifier") {
          self.push([&part_signifier, " "]);
        }
        self.push_str(&to_roman_numeral(part_num as u16).unwrap());
        self.push_str(": ");
      }
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Newlines {
  JoinWithBreak,
  #[default]
  JoinWithSpace,
  Preserve,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EphemeralState {
  VisitingSimpleTermDescription,
  IsSourceBlock,
  InBibliography,
  InGlossaryList,
  InAppendix,
}

const fn list_type_from_depth(depth: u8) -> &'static str {
  match depth {
    1 => "1",
    2 => "a",
    3 => "i",
    4 => "A",
    _ => "I",
  }
}

fn list_type_from_class(class: &str) -> Option<&'static str> {
  match class {
    "arabic" => Some("1"),
    "loweralpha" => Some("a"),
    "lowerroman" => Some("i"),
    "upperalpha" => Some("A"),
    "upperroman" => Some("I"),
    _ => None,
  }
}

const fn list_class_from_depth(depth: u8) -> &'static str {
  match depth {
    1 => "arabic",
    2 => "loweralpha",
    3 => "lowerroman",
    4 => "upperalpha",
    _ => "upperroman",
  }
}

macro_rules! num_str {
  ($n:expr) => {
    match $n {
      0 => Cow::Borrowed("0"),
      1 => Cow::Borrowed("1"),
      2 => Cow::Borrowed("2"),
      3 => Cow::Borrowed("3"),
      4 => Cow::Borrowed("4"),
      5 => Cow::Borrowed("5"),
      6 => Cow::Borrowed("6"),
      _ => Cow::Owned($n.to_string()),
    }
  };
}

const fn incr(num: &mut usize) -> usize {
  *num += 1;
  *num
}

pub(crate) use num_str;

lazy_static! {
  pub static ref REMOVE_FILE_EXT: Regex = Regex::new(r"^(.*)\.[^.]+$").unwrap();
}

// TODO: maybe move this into the parser?
#[cfg(debug_assertions)]
static INIT: Once = Once::new();

#[cfg(debug_assertions)]
fn configure_test_tracing() {
  INIT.call_once(|| {
    if std::env::var("RUST_LOG").is_ok() {
      let subscriber = fmt::Subscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .with_test_writer()
        .with_span_events(FmtSpan::ENTER)
        .finish();
      tracing::subscriber::set_global_default(subscriber)
        .expect("setting default tracing subscriber failed");
    }
  });
}
