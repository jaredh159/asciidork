use std::rc::Rc;
use std::sync::Once;

use tracing::instrument;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::{EnvFilter, fmt};

use crate::internal::*;
use EphemeralState::*;
use ast::AdjacentNewline;
use utils::set_backend_attrs;

#[derive(Debug, Default)]
pub struct AsciidoctorHtml {
  html: String,
  alt_html: String,
  doc_meta: DocumentMeta,
  default_newlines: Newlines,
  newlines: Newlines,
  autogen_conum: u8,
  fig_caption_num: usize,
  table_caption_num: usize,
  example_caption_num: usize,
  listing_caption_num: usize,
  state: BackendState,
}

impl Backend for AsciidoctorHtml {
  type Output = String;
  type Error = Infallible;
  const OUTFILESUFFIX: &'static str = ".html";

  fn set_job_attrs(attrs: &mut asciidork_core::JobAttrs) {
    Self::set_html_job_attrs(attrs);
  }

  #[instrument(skip_all)]
  fn enter_document(&mut self, document: &Document) {
    #[cfg(debug_assertions)]
    configure_test_tracing();

    self.doc_meta = document.meta.clone();
    set_backend_attrs::<Self>(&mut self.doc_meta);
    self.state.section_num_levels = document.meta.isize("sectnumlevels").unwrap_or(3);
    if document.meta.is_true("hardbreaks-option") {
      self.default_newlines = Newlines::JoinWithBreak
    }

    if !self.standalone() {
      return;
    }
    self.open_doc_head(&document.meta);
    self.meta_tags(&document.meta);
    self.render_favicon(&document.meta);
    self.render_meta_authors(document.meta.authors());
    self.render_title(document, &document.meta);
    self.render_styles(&document.meta);
    self.push_str("</head>");
    self.open_body(document);
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
      if !self.doc_meta.authors().is_empty()
        || self.doc_meta.get("revnumber").is_some()
        || self.doc_meta.get("revdate").is_some()
        || self.doc_meta.get("revremark").is_some()
      {
        self.render_header_details();
      }
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
    if !self.state.footnotes.borrow().is_empty() && !self.state.in_asciidoc_table_cell {
      self.render_footnotes();
    }
    if !self.doc_meta.embedded && !self.doc_meta.is_true("nofooter") {
      self.render_division_start("footer");
      if let Some(rev) = self.doc_meta.string("revnumber") {
        let label = self.doc_meta.string_or("version-label", "");
        self.push([r#"<div id="footer-text">"#, &label, " ", &rev, "<br></div>"]);
      }
      // TODO: last-update-label
      // TODO: docinfo
    }
  }

  #[instrument(skip_all)]
  fn exit_footer(&mut self) {
    if !self.doc_meta.embedded && !self.doc_meta.is_true("nofooter") {
      self.push_str("</div>")
    }
  }

  #[instrument(skip_all)]
  fn enter_document_title(&mut self) {
    if self.render_doc_title() {
      self.push_str("<h1>")
    } else {
      self.start_buffering();
    }
  }

  #[instrument(skip_all)]
  fn exit_document_title(&mut self) {
    if self.render_doc_title() {
      self.push_str("</h1>");
    } else {
      self.swap_take_buffer(); // discard
    }
  }

  #[instrument(skip_all)]
  fn enter_toc(&mut self, toc: &TableOfContents, macro_block: Option<&Block>) {
    self.state.ephemeral.insert(InTableOfContents);
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
    self.on_toc_exit();
    self.state.ephemeral.remove(&InTableOfContents);
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
    HtmlBackend::enter_toc_node(self, node);
  }

  #[instrument(skip_all)]
  fn exit_toc_node(&mut self, node: &TocNode) {
    HtmlBackend::exit_toc_node(self, node);
  }

  #[instrument(skip_all)]
  fn exit_toc_content(&mut self) {
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
    section_tag.push_class(backend::html::util::section_class(section));
    self.push_open_tag(section_tag);
    self.enter_section_state(section);
  }

  #[instrument(skip_all)]
  fn exit_section(&mut self, section: &Section) {
    if section.level == 1 {
      self.push_str("</div>");
    }
    self.push_str("</div>");
    self.exit_section_state(section);
  }

  #[instrument(skip_all)]
  fn enter_section_heading(&mut self, section: &Section) {
    HtmlBackend::enter_section_heading(self, section, false);
  }

  #[instrument(skip_all)]
  fn exit_section_heading(&mut self, section: &Section) {
    HtmlBackend::exit_section_heading(self, section);
    if section.level == 1 {
      self.push_str(r#"<div class="sectionbody">"#);
    }
  }

  #[instrument(skip_all)]
  fn enter_meta_title(&mut self) {
    self.start_buffering();
  }

  #[instrument(skip_all)]
  fn exit_meta_title(&mut self) {
    self.stop_buffering();
  }

  #[instrument(skip_all)]
  fn enter_compound_block_content(&mut self, _children: &[Block], _block: &Block) {}
  #[instrument(skip_all)]
  fn exit_compound_block_content(&mut self, _children: &[Block], _block: &Block) {}

  #[instrument(skip_all)]
  fn enter_simple_block_content(&mut self, block: &Block) {
    if block.context == BlockContext::Verse {
      self.newlines = Newlines::Preserve;
    } else if block.meta.attrs.has_option("hardbreaks") {
      self.newlines = Newlines::JoinWithBreak;
    }
  }

  #[instrument(skip_all)]
  fn exit_simple_block_content(&mut self, _block: &Block) {
    self.newlines = self.default_newlines;
  }

  #[instrument(skip_all)]
  fn enter_sidebar_block(&mut self, block: &Block) {
    self.open_element("div", &["sidebarblock"], &block.meta.attrs);
    self.push_str(r#"<div class="content">"#);
    self.render_buffered_block_title(block);
  }

  #[instrument(skip_all)]
  fn exit_sidebar_block(&mut self, _block: &Block) {
    self.push_str("</div></div>");
  }

  #[instrument(skip_all)]
  fn enter_listing_block(&mut self, block: &Block) {
    self.open_element("div", &["listingblock"], &block.meta.attrs);
    self.render_buffered_block_title(block);
    self.push_str(r#"<div class="content"><pre"#);
    let doc_lang = self.doc_meta.string("source-language");
    if block.meta.attrs.is_source() || doc_lang.is_some() {
      self.push_str(r#" class="highlight"><code"#);
      if let Some(lang) = block.meta.attrs.source_language() {
        self.push([" class=\"language-", lang, "\" data-lang=\"", lang, "\""]);
      } else if let Some(lang) = doc_lang {
        self.push([" class=\"language-", &lang, "\" data-lang=\"", &lang, "\""]);
      }
      self.push_ch('>');
      self.state.ephemeral.insert(IsSourceBlock);
    } else {
      self.push_ch('>');
    }
    self.newlines = Newlines::Preserve;
  }

  #[instrument(skip_all)]
  fn exit_listing_block(&mut self, _block: &Block) {
    if self.state.ephemeral.remove(&IsSourceBlock) {
      self.push_str("</code>");
    }
    self.push_str("</pre></div></div>");
    self.newlines = self.default_newlines;
  }

  #[instrument(skip_all)]
  fn enter_literal_block(&mut self, block: &Block) {
    self.open_element("div", &["literalblock"], &block.meta.attrs);
    self.render_buffered_block_title(block);
    self.push_str(r#"<div class="content"><pre>"#);
    self.newlines = Newlines::Preserve;
  }

  #[instrument(skip_all)]
  fn exit_literal_block(&mut self, _block: &Block) {
    self.push_str("</pre></div></div>");
    self.newlines = self.default_newlines;
  }

  #[instrument(skip_all)]
  fn enter_quoted_paragraph(&mut self, block: &Block) {
    self.open_element("div", &["quoteblock"], &block.meta.attrs);
    self.render_buffered_block_title(block);
    self.push_str("<blockquote>");
  }

  #[instrument(skip_all)]
  fn exit_quoted_paragraph(&mut self, _block: &Block) {
    self.push_str("</div>");
  }

  #[instrument(skip_all)]
  fn enter_quote_block(&mut self, block: &Block, _has_attribution: bool) {
    self.open_element("div", &["quoteblock"], &block.meta.attrs);
    self.render_buffered_block_title(block);
    self.push_str("<blockquote>");
  }

  #[instrument(skip_all)]
  fn exit_quote_block(&mut self, block: &Block, has_attribution: bool) {
    if block.context == BlockContext::Verse && !has_attribution {
      self.push_str("</pre>");
    } else if !has_attribution {
      self.push_str("</blockquote>");
    }
    self.push_str("</div>");
  }

  #[instrument(skip_all)]
  fn enter_quote_attribution(&mut self, block: &Block, _has_cite: bool) {
    if block.context == BlockContext::Verse {
      self.push_str("</pre>");
    } else {
      self.push_str("</blockquote>");
    }
    self.push_str(r#"<div class="attribution">&#8212; "#);
  }

  #[instrument(skip_all)]
  fn exit_quote_attribution(&mut self, _block: &Block, has_cite: bool) {
    if !has_cite {
      self.push_str("</div>");
    }
  }

  #[instrument(skip_all)]
  fn enter_quote_cite(&mut self, _block: &Block, has_attribution: bool) {
    if has_attribution {
      self.push_str(r#"<br><cite>"#);
    } else {
      self.push_str(r#"</blockquote><div class="attribution">&#8212; "#);
    }
  }

  #[instrument(skip_all)]
  fn exit_quote_cite(&mut self, _block: &Block, has_attribution: bool) {
    if has_attribution {
      self.push_str(r#"</cite></div>"#);
    } else {
      self.push_str("</div>");
    }
  }

  #[instrument(skip_all)]
  fn enter_verse_block(&mut self, block: &Block, _has_attribution: bool) {
    self.open_element("div", &["verseblock"], &block.meta.attrs);
    self.render_buffered_block_title(block);
    self.push_str(r#"<pre class="content">"#);
  }

  #[instrument(skip_all)]
  fn exit_verse_block(&mut self, block: &Block, has_attribution: bool) {
    self.exit_quote_block(block, has_attribution);
  }

  #[instrument(skip_all)]
  fn enter_example_block(&mut self, block: &Block) {
    if block.meta.attrs.has_option("collapsible") {
      self.open_element("details", &[], &block.meta.attrs);
      if block.meta.attrs.has_option("open") {
        self.html.pop();
        self.push_str(" open>");
      }
      self.push_str(r#"<summary class="title">"#);
      if block.has_title() {
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
  fn exit_example_block(&mut self, block: &Block) {
    if block.meta.attrs.has_option("collapsible") {
      self.push_str("</div></details>");
    } else {
      self.push_str("</div></div>");
    }
  }

  #[instrument(skip_all)]
  fn enter_open_block(&mut self, block: &Block) {
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
  fn exit_open_block(&mut self, block: &Block) {
    if block.meta.attrs.special_sect() == Some(SpecialSection::Abstract) {
      self.push_str("</blockquote></div>");
    } else {
      self.push_str("</div></div>");
    }
  }

  #[instrument(skip_all)]
  fn enter_discrete_heading(&mut self, level: u8, id: Option<&str>, block: &Block) {
    self.push_enter_discrete_heading(level, id, block);
  }

  #[instrument(skip_all)]
  fn exit_discrete_heading(&mut self, level: u8, _id: Option<&str>, _block: &Block) {
    self.push_exit_discrete_heading(level);
  }

  #[instrument(skip_all)]
  fn enter_unordered_list(&mut self, block: &Block, items: &[ListItem], _depth: u8) {
    let (mut wrap, mut list) = self.start_enter_unordered_list("div", block);
    if items.iter().any(ListItem::is_checklist) {
      wrap.push_class("checklist");
      list.push_class("checklist");
    }
    self.push_open_tag(wrap);
    self.render_buffered_block_title(block);
    self.push_open_tag(list);
  }

  #[instrument(skip_all)]
  fn exit_unordered_list(&mut self, _block: &Block, _items: &[ListItem], _depth: u8) {
    self.state.interactive_list_stack.pop();
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
    self.state.desc_list_depth += 1;
    if block.meta.attrs.has_str_positional("horizontal") {
      self.state.ephemeral.insert(InHorizontalDescList);
      self.open_element("div", &["hdlist"], &block.meta.attrs);
      self.render_buffered_block_title(block);
      self.push_str("<table>");
      let labelwidth = block.meta.attrs.named("labelwidth");
      let itemwidth = block.meta.attrs.named("itemwidth");
      if labelwidth.is_some() || itemwidth.is_some() {
        self.push_str("<colgroup>");
        if let Some(labelwidth) = labelwidth {
          self.push_str(&format!(
            r#"<col style="width: {}%;">"#,
            labelwidth.trim_end_matches('%')
          ));
        } else {
          self.push_str("<col>");
        }
        if let Some(itemwidth) = itemwidth {
          self.push_str(&format!(
            r#"<col style="width: {}%;">"#,
            itemwidth.trim_end_matches('%')
          ));
        } else {
          self.push_str("<col>");
        }
        self.push_str("</colgroup>");
      }
    } else if block.meta.attrs.has_str_positional("qanda") {
      self.state.ephemeral.insert(InQandaDescList);
      self.open_element("div", &["qlist", "qanda"], &block.meta.attrs);
      self.render_buffered_block_title(block);
      self.push_str("<ol>");
    } else if block.meta.attrs.special_sect() == Some(SpecialSection::Glossary) {
      self.state.ephemeral.insert(InGlossaryList);
      self.open_element("div", &["dlist", "glossary"], &block.meta.attrs);
      self.render_buffered_block_title(block);
      self.push_str("<dl>");
    } else {
      self.open_element("div", &["dlist"], &block.meta.attrs);
      self.render_buffered_block_title(block);
      self.push_str("<dl>");
    }
  }

  #[instrument(skip_all)]
  fn exit_description_list(&mut self, _block: &Block, _items: &[ListItem], _depth: u8) {
    self.state.ephemeral.remove(&InGlossaryList);
    if self.state.ephemeral.remove(&InHorizontalDescList) {
      self.push_str("</table></div>");
    } else if self.state.ephemeral.remove(&InQandaDescList) {
      self.push_str("</ol></div>");
    } else {
      self.push_str("</dl></div>");
    }
    self.state.desc_list_depth -= 1;
  }

  #[instrument(skip_all)]
  fn enter_description_list_term(&mut self, _item: &ListItem, num: usize, _total: usize) {
    if self.state.ephemeral.contains(&InGlossaryList) {
      self.push_str(r#"<dt>"#);
    } else if self.state.ephemeral.contains(&InHorizontalDescList) {
      self.push_str(r#"<tr><td class="hdlist1">"#);
    } else if self.state.ephemeral.contains(&InQandaDescList) {
      if num == 1 {
        self.push_str(r#"<li>"#);
      }
      self.push_str(r#"<p><em>"#);
    } else {
      self.push_str(r#"<dt class="hdlist1">"#);
    }
  }

  #[instrument(skip_all)]
  fn exit_description_list_term(&mut self, _item: &ListItem, _num: usize, _total: usize) {
    if self.state.ephemeral.contains(&InHorizontalDescList) {
      self.push_str("</td>");
    } else if self.state.ephemeral.contains(&InQandaDescList) {
      self.push_str(r#"</em></p>"#);
    } else {
      self.push_str("</dt>");
    }
  }

  #[instrument(skip_all)]
  fn enter_description_list_description(&mut self, _item: &ListItem) {
    if self.state.ephemeral.contains(&InHorizontalDescList) {
      self.push_str(r#"<td class="hdlist2">"#);
    } else if !self.state.ephemeral.contains(&InQandaDescList) {
      self.push_str("<dd>");
    }
  }

  #[instrument(skip_all)]
  fn exit_description_list_description(&mut self, _item: &ListItem) {
    if self.state.ephemeral.contains(&InHorizontalDescList) {
      self.push_str("</td></tr>");
    } else if self.state.ephemeral.contains(&InQandaDescList) {
      self.push_str("</li>");
    } else {
      self.push_str("</dd>");
    }
  }

  #[instrument(skip_all)]
  fn enter_description_list_description_text(&mut self, _text: &Block, _item: &ListItem) {
    self.state.ephemeral.insert(VisitingSimpleTermDescription);
  }

  #[instrument(skip_all)]
  fn exit_description_list_description_text(&mut self, _text: &Block, _item: &ListItem) {
    self.state.ephemeral.remove(&VisitingSimpleTermDescription);
  }

  #[instrument(skip_all)]
  fn enter_ordered_list(&mut self, block: &Block, items: &[ListItem], depth: u8) {
    let (class, list_type) = self.start_enter_ordered_list(block, depth);
    let classes = &["olist", class];
    self.open_element("div", classes, &block.meta.attrs);
    self.render_buffered_block_title(block);
    self.finish_enter_ordered_list(class, list_type, block, items);
  }

  #[instrument(skip_all)]
  fn exit_ordered_list(&mut self, _block: &Block, _items: &[ListItem], _depth: u8) {
    self.state.interactive_list_stack.pop();
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
    if !self
      .state
      .ephemeral
      .contains(&VisitingSimpleTermDescription)
    {
      self.open_block_wrap(block);
      self.render_buffered_block_title(block);
    }
    if block.meta.attrs.special_sect() == Some(SpecialSection::Abstract) {
      self.push_str("<blockquote>");
    } else {
      self.push_str("<p>");
    }
  }

  #[instrument(skip_all)]
  fn exit_paragraph_block(&mut self, block: &Block) {
    if self.doc_meta.get_doctype() == DocType::Inline {
      return;
    }
    if block.meta.attrs.special_sect() == Some(SpecialSection::Abstract) {
      self.push_str("</blockquote>");
    } else {
      self.push_str("</p>");
    }
    if !self
      .state
      .ephemeral
      .contains(&VisitingSimpleTermDescription)
    {
      self.push_str("</div>");
    }
    self.state.ephemeral.remove(&VisitingSimpleTermDescription);
  }

  #[instrument(skip_all)]
  fn asciidoc_table_cell_backend(&mut self) -> Self {
    let mut backend = Self::default();
    backend.state.footnotes = Rc::clone(&self.state.footnotes);
    backend.state.in_asciidoc_table_cell = true;
    backend
  }

  #[instrument(skip_all)]
  fn visit_asciidoc_table_cell_result(&mut self, cell_backend: Self) {
    self.state.in_asciidoc_table_cell = false;
    self.html.push_str(&cell_backend.into_result().unwrap());
  }

  #[instrument(skip_all)]
  fn enter_table(&mut self, table: &Table, block: &Block) {
    let mut tag = OpenTag::new("table", &block.meta.attrs);
    tag.push_class("tableblock");
    finish_open_table_tag(&mut tag, block, &self.doc_meta);
    self.push_open_tag(tag);
    self.render_buffered_block_title(block);
    backend::html::table::push_colgroup(&mut self.html, table, block);
  }

  fn exit_table(&mut self, _table: &Table, _block: &Block) {
    self.push_str("</table>");
  }

  #[instrument(skip_all)]
  fn enter_table_section(&mut self, section: TableSection) {
    HtmlBackend::enter_table_section(self, section);
  }

  #[instrument(skip_all)]
  fn exit_table_section(&mut self, section: TableSection) {
    HtmlBackend::exit_table_section(self, section);
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
    backend::html::table::open_cell(&mut self.html, cell, &section, Some("tableblock"));
    match &cell.content {
      CellContent::AsciiDoc(_) => self.push_str("<div class=\"content\">"),
      CellContent::Literal(_) => {
        self.newlines = Newlines::Preserve;
        self.push_str("<div class=\"literal\"><pre>");
      }
      _ => {}
    }
  }

  #[instrument(skip_all)]
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
      (_, CellContent::AsciiDoc(_)) => self.push_str("</div></td>"),
      _ => self.push_str("</td>"),
    }
  }

  #[instrument(skip_all)]
  fn enter_cell_paragraph(&mut self, cell: &Cell, section: TableSection) {
    match (section, &cell.content) {
      (TableSection::Header, _) => {}
      (_, CellContent::Emphasis(_)) => self.push_str("<p class=\"tableblock\"><em>"),
      (_, CellContent::Monospace(_)) => self.push_str("<p class=\"tableblock\"><code>"),
      (_, CellContent::Strong(_)) => self.push_str("<p class=\"tableblock\"><strong>"),
      _ => self.push_str("<p class=\"tableblock\">"),
    }
  }

  #[instrument(skip_all)]
  fn exit_cell_paragraph(&mut self, cell: &Cell, section: TableSection) {
    match (section, &cell.content) {
      (TableSection::Header, _) => self.push_str(" "),
      (_, CellContent::Emphasis(_)) => self.push_str("</em></p>"),
      (_, CellContent::Monospace(_)) => self.push_str("</code></p>"),
      (_, CellContent::Strong(_)) => self.push_str("</strong></p>"),
      _ => self.push_str("</p>"),
    }
  }

  #[instrument(skip_all)]
  fn enter_inline_italic(&mut self, attrs: Option<&AttrList>) {
    self.open_element_opt("em", &[], attrs);
  }

  #[instrument(skip_all)]
  fn exit_inline_italic(&mut self, _attrs: Option<&AttrList>) {
    self.push_str("</em>");
  }

  #[instrument(skip_all)]
  fn enter_inline_mono(&mut self, attrs: Option<&AttrList>) {
    self.open_element_opt("code", &[], attrs);
  }

  #[instrument(skip_all)]
  fn exit_inline_mono(&mut self, _attrs: Option<&AttrList>) {
    self.push_str("</code>");
  }

  #[instrument(skip_all)]
  fn enter_inline_bold(&mut self, attrs: Option<&AttrList>) {
    self.open_element_opt("strong", &[], attrs);
  }

  #[instrument(skip_all)]
  fn exit_inline_bold(&mut self, _attrs: Option<&AttrList>) {
    self.push_str("</strong>");
  }

  #[instrument(skip_all)]
  fn enter_inline_lit_mono(&mut self, attrs: Option<&AttrList>) {
    self.open_element_opt("code", &[], attrs);
  }

  #[instrument(skip_all)]
  fn exit_inline_lit_mono(&mut self, _attrs: Option<&AttrList>) {
    self.push_str("</code>");
  }

  #[instrument(skip_all)]
  fn enter_inline_highlight(&mut self, attrs: Option<&AttrList>) {
    self.open_element_opt("mark", &[], attrs);
  }

  #[instrument(skip_all)]
  fn exit_inline_highlight(&mut self, _attrs: Option<&AttrList>) {
    self.push_str("</mark>");
  }

  #[instrument(skip_all)]
  fn enter_inline_subscript(&mut self, attrs: Option<&AttrList>) {
    self.open_element_opt("sub", &[], attrs);
  }

  #[instrument(skip_all)]
  fn exit_inline_subscript(&mut self, _attrs: Option<&AttrList>) {
    self.push_str("</sub>");
  }

  #[instrument(skip_all)]
  fn enter_inline_superscript(&mut self, attrs: Option<&AttrList>) {
    self.open_element_opt("sup", &[], attrs);
  }

  #[instrument(skip_all)]
  fn exit_inline_superscript(&mut self, _attrs: Option<&AttrList>) {
    self.push_str("</sup>");
  }

  #[instrument(skip_all)]
  fn visit_spaced_dashes(&mut self, len: u8, _adjacent_newline: AdjacentNewline) {
    if len == 2 {
      self.push_str("&#8201;&#8212;&#8201;");
    } else {
      self.push_str(" --- ");
    }
  }

  #[instrument(skip_all)]
  fn visit_inline_specialchar(&mut self, char: &SpecialCharKind) {
    HtmlBackend::visit_inline_specialchar(self, char);
  }

  #[instrument(skip_all)]
  fn visit_symbol(&mut self, kind: SymbolKind) {
    match kind {
      SymbolKind::Copyright => self.push_str("&#169;"),
      SymbolKind::Registered => self.push_str("&#174;"),
      SymbolKind::Trademark => self.push_str("&#8482;"),
      SymbolKind::EmDash => self.push_str("&#8212;&#8203;"),
      SymbolKind::TripleDash => self.push_str("---"),
      SymbolKind::Ellipsis => self.push_str("&#8230;&#8203;"),
      SymbolKind::SingleRightArrow => self.push_str("&#8594;"),
      SymbolKind::DoubleRightArrow => self.push_str("&#8658;"),
      SymbolKind::SingleLeftArrow => self.push_str("&#8592;"),
      SymbolKind::DoubleLeftArrow => self.push_str("&#8656;"),
    }
  }

  #[instrument(skip_all)]
  fn enter_inline_quote(&mut self, kind: QuoteKind) {
    match kind {
      QuoteKind::Double => self.push_str("&#8220;"),
      QuoteKind::Single => self.push_str("&#8216;"),
    }
  }

  #[instrument(skip_all)]
  fn exit_inline_quote(&mut self, kind: QuoteKind) {
    match kind {
      QuoteKind::Double => self.push_str("&#8221;"),
      QuoteKind::Single => self.push_str("&#8217;"),
    }
  }

  #[instrument(skip_all)]
  fn visit_curly_quote(&mut self, kind: CurlyKind) {
    HtmlBackend::visit_curly_quote(self, kind);
  }

  #[instrument(skip_all)]
  fn visit_multichar_whitespace(&mut self, _whitespace: &str) {
    self.push_ch(' ');
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
  fn enter_text_span(&mut self, attrs: Option<&AttrList>) {
    self.open_element_opt("span", &[], attrs);
  }

  #[instrument(skip_all)]
  fn exit_text_span(&mut self, _attrs: Option<&AttrList>) {
    self.push_str("</span>");
  }

  #[instrument(skip_all)]
  fn enter_xref(&mut self, target: &SourceString, _has_reftext: bool, kind: XrefKind) {
    HtmlBackend::enter_xref(self, target, kind);
  }

  #[instrument(skip_all)]
  fn exit_xref(&mut self, _target: &SourceString, _has_reftext: bool, _kind: XrefKind) {
    HtmlBackend::exit_xref(self);
  }

  #[instrument(skip_all)]
  fn enter_xref_text(&mut self, is_biblio: bool) {
    HtmlBackend::enter_xref_text(self, is_biblio);
  }

  #[instrument(skip_all)]
  fn exit_xref_text(&mut self, is_biblio: bool) {
    HtmlBackend::exit_xref_text(self, is_biblio);
  }

  #[instrument(skip_all)]
  fn visit_missing_xref(
    &mut self,
    target: &SourceString,
    kind: XrefKind,
    doc_title: Option<&DocTitle>,
  ) {
    self.render_missing_xref(target, kind, doc_title);
  }

  #[instrument(skip_all)]
  fn visit_inline_anchor(&mut self, id: &str) {
    if !self.state.ephemeral.contains(&InTableOfContents) {
      self.push(["<a id=\"", id, "\"></a>"]);
    }
  }

  #[instrument(skip_all)]
  fn visit_biblio_anchor(&mut self, id: &str, reftext: Option<&str>) {
    self.push(["<a id=\"", id, "\"></a>[", reftext.unwrap_or(id), "]"]);
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
  fn visit_button_macro(&mut self, text: &SourceString) {
    self.push([r#"<b class="button">"#, text, "</b>"])
  }

  #[instrument(skip_all)]
  fn visit_icon_macro(&mut self, target: &SourceString, attrs: &AttrList) {
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
        self.push_ch('"');
        self.push_named_attr("width", attrs);
        self.push_named_attr("title", attrs);
        self.push_str(r#">"#);
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
        self.push_ch('"');
        self.push_named_attr("title", attrs);
        self.push_str("></i>");
      }
    }
    if has_link {
      self.push_str("</a>");
    }
    self.push_str("</span>");
  }

  #[instrument(skip_all)]
  fn visit_image_macro(&mut self, target: &SourceString, attrs: &AttrList) {
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
        assert!(self.alt_html.is_empty());
        self.swap_buffers();
        self.push_img_path(target);
        a_tag.push_str(&self.swap_take_buffer());
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
    target: &SourceString,
    attrs: Option<&AttrList>,
    scheme: Option<UrlScheme>,
    resolving_xref: bool,
    has_link_text: bool,
    blank_window_shorthand: bool,
  ) {
    HtmlBackend::enter_link_macro(
      self,
      target,
      attrs,
      scheme,
      resolving_xref,
      has_link_text,
      blank_window_shorthand,
    );
  }

  #[instrument(skip_all)]
  fn exit_link_macro(
    &mut self,
    target: &SourceString,
    _attrs: Option<&AttrList>,
    _scheme: Option<UrlScheme>,
    resolving_xref: bool,
    has_link_text: bool,
  ) {
    HtmlBackend::exit_link_macro(self, target, resolving_xref, has_link_text);
  }

  #[instrument(skip_all)]
  fn visit_menu_macro(&mut self, items: &[SourceString]) {
    HtmlBackend::visit_menu_macro(self, items);
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
  fn enter_image_block(&mut self, img_target: &SourceString, img_attrs: &AttrList, block: &Block) {
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
      let mut a_tag = OpenTag::new("a", &NoAttrs);
      a_tag.push_class("image");
      a_tag.push_str("\" href=\"");
      a_tag.push_str(href);
      a_tag.push_ch('"');
      a_tag.opened_classes = false;
      a_tag.push_link_attrs(img_attrs, true, false);
      self.push_open_tag(a_tag);
      has_link = true;
    }
    self.render_image(img_target, img_attrs, true);
    if has_link {
      self.push_str("</a>");
    }
    self.push_str(r#"</div>"#);
  }

  #[instrument(skip_all)]
  fn exit_image_block(&mut self, _target: &SourceString, attrs: &AttrList, block: &Block) {
    if let Some(title) = attrs.named("title") {
      self.render_block_title(title, block);
    } else if block.has_title() {
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
    } else if name == "sectnumlevels"
      && let Some(level) = value.isize()
    {
      self.state.section_num_levels = level;
    }
    // TODO: consider warning?
    _ = self.doc_meta.insert_doc_attr(name, value.clone());
  }

  #[instrument(skip_all)]
  fn enter_footnote(&mut self, id: Option<&SourceString>, has_content: bool) {
    if has_content {
      self.start_buffering();
      return;
    }
    if let Some(prev_ref_num) = self.prev_footnote_ref_num(id) {
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
  fn exit_footnote(&mut self, id: Option<&SourceString>, has_content: bool) {
    if !has_content {
      return; // this means the footnore was referring to a previously defined fn by id
    }
    let num = self.state.footnotes.borrow().len() + 1;
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
      .state
      .footnotes
      .borrow_mut()
      .push((id.map(|id| id.to_string()), footnote));
  }

  #[instrument(skip_all)]
  fn enter_visible_index_term(&mut self) -> bool {
    true
  }

  #[instrument(skip_all)]
  fn enter_concealed_index_term(&mut self, _num_terms: u8) -> bool {
    false
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

impl AltHtmlBuf for AsciidoctorHtml {
  fn alt_htmlbuf(&mut self) -> &mut String {
    &mut self.alt_html
  }

  fn buffers(&mut self) -> (&mut String, &mut String) {
    (&mut self.html, &mut self.alt_html)
  }
}

impl HtmlBackend for AsciidoctorHtml {
  fn state(&self) -> &BackendState {
    &self.state
  }
  fn state_mut(&mut self) -> &mut BackendState {
    &mut self.state
  }
  fn doc_meta(&self) -> &DocumentMeta {
    &self.doc_meta
  }
}

impl AsciidoctorHtml {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn into_string(self) -> String {
    self.html
  }

  fn render_buffered_block_title(&mut self, block: &Block) {
    if block.has_title() {
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

  pub fn open_block_wrap(&mut self, block: &Block) {
    let mut classes: &[&str] = &["paragraph"];
    if block.meta.attrs.special_sect() == Some(SpecialSection::Abstract) {
      classes = &["quoteblock", "abstract"]
    };
    self.open_element("div", classes, &block.meta.attrs);
  }

  fn render_footnotes(&mut self) {
    self.render_division_start("footnotes");
    self.push_str("<hr>");
    let footnotes = mem::take(&mut self.state.footnotes);
    for (i, (_, footnote)) in footnotes.borrow().iter().enumerate() {
      let num = (i + 1).to_string();
      self.push_str(r#"<div class="footnote" id="_footnotedef_"#);
      self.push([&num, r##""><a href="#_footnoteref_"##, &num, "\">"]);
      self.push([&num, "</a>. ", footnote, "</div>"]);
    }
    self.push_str(r#"</div>"#);
    self.state.footnotes = footnotes;
  }

  fn render_styles(&mut self, meta: &DocumentMeta) {
    if meta.str("stylesheet") == Some("") {
      let family = match meta.str("webfonts") {
        None | Some("") => {
          "Open+Sans:300,300italic,400,400italic,600,600italic%7CNoto+Serif:400,400italic,700,700italic%7CDroid+Sans+Mono:400,700"
        }
        Some(custom) => custom,
      };
      self.push([
        r#"<link rel="stylesheet" href="https://fonts.googleapis.com/css?family="#,
        family,
        "\" />",
      ]);
    }

    self.render_embedded_stylesheet(crate::css::DEFAULT);
  }

  fn render_checklist_item(&mut self, item: &ListItem) {
    if let ListItemTypeMeta::Checklist(checked, _) = &item.type_meta {
      match (
        self.state.interactive_list_stack.last() == Some(&true),
        checked,
      ) {
        (false, true) => self.push_str("&#10003;"),
        (false, false) => self.push_str("&#10063;"),
        (true, true) => self.push_str(r#"<input type="checkbox" data-item-complete="1" checked>"#),
        (true, false) => self.push_str(r#"<input type="checkbox" data-item-complete="0">"#),
      }
    }
  }

  fn push_admonition_img(&mut self, kind: AdmonitionKind) {
    self.push_str(r#"<img src=""#);
    self.push_icon_uri(kind.lowercase_str(), None);
    self.push([r#"" alt=""#, kind.str(), r#"">"#]);
  }

  fn render_division_start(&mut self, id: &str) {
    self.push([r#"<div id=""#, id, "\""]);
    if let Some(max_width) = self.doc_meta.string("max-width") {
      self.push([r#" style="max-width: "#, &max_width, r#";">"#]);
    } else {
      self.push_str(">");
    }
  }

  fn render_image(&mut self, target: &str, attrs: &AttrList, is_block: bool) {
    let format = attrs.named("format").or_else(|| file::ext(target));
    let is_svg = matches!(format, Some("svg" | "SVG"));
    if is_svg && attrs.has_option("interactive") && self.doc_meta.safe_mode != SafeMode::Secure {
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

  fn push_callout_number_font(&mut self, num: u8) {
    let n_str = &num_str!(num);
    self.push([r#"<i class="conum" data-value=""#, n_str, r#""></i>"#]);
    self.push([r#"<b>("#, n_str, ")</b>"]);
  }

  fn render_header_details(&mut self) {
    self.push_str("<div class=\"details\">");
    if !self.doc_meta.authors().is_empty() {
      let authors = std::mem::take(&mut self.doc_meta.authors);
      for (index, author) in authors.iter().enumerate() {
        self.render_author_detail(author, index);
      }
      self.doc_meta.authors = authors;
    }

    if let Some(revnumber) = self.doc_meta.string("revnumber") {
      let label = self.doc_meta.string_or("version-label", "").to_lowercase();
      self.push([r#"<span id="revnumber">"#, &label, " ", &revnumber]);
      if self.doc_meta.get("revdate").is_some() {
        self.push_str(",</span>");
      } else {
        self.push_str("</span>");
      }
    }
    if let Some(revdate) = self.doc_meta.string("revdate") {
      self.push([r#"<span id="revdate">"#, &revdate, "</span>"]);
    }
    if let Some(revremark) = self.doc_meta.string("revremark") {
      self.push([r#"<br><span id="revremark">"#, &revremark, "</span>"]);
    }
    self.push_str("</div>");
  }

  fn render_author_detail(&mut self, author: &Author, index: usize) {
    self.push_str(r#"<span id="author"#);
    if index > 0 {
      self.push_str(&(index + 1).to_string());
    }
    self.push_str(r#"" class="author">"#);
    self.push_str(&author.fullname());
    self.push_str("</span><br>");

    if let Some(email) = &author.email {
      self.push_str(r#"<span id="email"#);
      if index > 0 {
        self.push_str(&(index + 1).to_string());
      }
      self.push_str(r#"" class="email"><a href="mailto:"#);
      self.push_str(email);
      self.push_str(r#"">"#);
      self.push_str(email);
      self.push_str("</a></span><br>");
    }
  }
}

fn finish_open_table_tag(tag: &mut OpenTag, block: &Block, doc_meta: &DocumentMeta) {
  tag.push_resolved_attr_class(
    "frame",
    Some("all"),
    Some("table-frame"),
    Some("frame-"),
    &block.meta,
    doc_meta,
  );

  tag.push_resolved_attr_class(
    "grid",
    Some("all"),
    Some("table-grid"),
    Some("grid-"),
    &block.meta,
    doc_meta,
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
    doc_meta,
  );

  if let Some(width) = explicit_width {
    tag.push_style(format!("width: {width}%;"));
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Newlines {
  JoinWithBreak,
  #[default]
  JoinWithSpace,
  Preserve,
}

const fn incr(num: &mut usize) -> usize {
  *num += 1;
  *num
}

pub(crate) use backend::num_str;

lazy_static! {
  pub static ref REMOVE_FILE_EXT: Regex = Regex::new(r"^(.*)\.[^.]+$").unwrap();
}

// TODO: maybe move this into the parser?
#[cfg(debug_assertions)]
static INIT: Once = Once::new();

#[cfg(debug_assertions)]
fn configure_test_tracing() {
  INIT.call_once(|| {
    // usage: `ASCIIDORK_LOG=1 RUST_LOG=trace cargo test`
    if std::env::var("ASCIIDORK_LOG").is_ok() {
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
