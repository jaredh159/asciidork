use std::rc::Rc;

use crate::internal::*;
use ast::AdjacentNewline;
use EphemeralState::*;

#[derive(Debug, Default)]
pub struct Html5s {
  doc_meta: DocumentMeta,
  html: String,
  alt_html: String,
  fig_caption_num: usize,
  table_caption_num: usize,
  example_caption_num: usize,
  listing_caption_num: usize,
  newlines: Newlines,
  default_newlines: Newlines,
  section_level_stack: Vec<u8>,
  state: BackendState,
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

  fn set_job_attrs(attrs: &mut asciidork_core::JobAttrs) {
    Self::set_html_job_attrs(attrs);
  }

  fn enter_document(&mut self, document: &Document) {
    self.doc_meta = document.meta.clone();
    utils::set_backend_attrs::<Self>(&mut self.doc_meta);
    self.state.section_num_levels = document.meta.isize("sectnumlevels").unwrap_or(3);

    if !self.standalone() {
      return;
    }
    self.open_doc_head(&document.meta);
    self.meta_tags(&document.meta);
    self.render_favicon(&document.meta);
    self.render_meta_authors(document.meta.authors());
    self.render_title(document, &document.meta);
    self.render_embedded_stylesheet(crate::css::DEFAULT);
    self.push_str("</head>");
    self.open_body(document);
    if let Some(max_width) = document.meta.string("max-width") {
      self.html.pop();
      self.push([r#" style="max-width: "#, &max_width, r#";">"#]);
    }
  }

  fn exit_document(&mut self, _document: &Document) {
    if self.standalone() {
      self.push_str("</body></html>");
    }
  }

  fn enter_header(&mut self) {
    if !self.doc_meta.embedded && !self.doc_meta.is_true("noheader") {
      self.push_str("<header>");
    }
  }

  fn exit_header(&mut self) {
    if self.doc_meta.embedded || self.doc_meta.is_true("noheader") {
      return;
    }
    if !self.doc_meta.authors().is_empty()
      || self.doc_meta.get("revnumber").is_some()
      || self.doc_meta.get("revdate").is_some()
      || self.doc_meta.get("revremark").is_some()
    {
      self.render_header_details();
    }
    self.push_str("</header>");
  }

  fn enter_content(&mut self) {
    if !self.doc_meta.embedded {
      self.push_str(r#"<div id="content">"#)
    }
  }

  fn exit_content(&mut self) {
    if !self.doc_meta.embedded {
      self.push_str("</div>")
    }
  }

  fn enter_footer(&mut self) {
    if !self.state.footnotes.borrow().is_empty() && !self.state.in_asciidoc_table_cell {
      self.render_footnotes();
    }
    if self.doc_meta.embedded || self.doc_meta.is_true("nofooter") {
      return;
    }
    self.push_str("<footer><div id=\"footer-text\">");
    if let Some(revnumber) = self.doc_meta.string("revnumber") {
      let label = self.doc_meta.string_or("version-label", "");
      self.push([&label, " ", &revnumber]);
    }
    // TODO: last-update-label
    // TODO: docinfo
    self.push_str("</div>");
  }

  fn exit_footer(&mut self) {
    if !self.doc_meta.embedded && !self.doc_meta.is_true("nofooter") {
      self.push_str("</footer>");
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
  }

  fn enter_toc(&mut self, toc: &TableOfContents, macro_block: Option<&Block>) {
    self.state.ephemeral.insert(InTableOfContents);
    let id = &macro_block
      .and_then(|b| b.meta.attrs.id().map(|id| id.to_string()))
      .unwrap_or("toc".to_string());
    self.push([r#"<nav id=""#, id, r#"" class=""#]);
    self.push_str(&self.doc_meta.string_or("toc-class", "toc"));
    self.push_str(r#"" role="doc-toc"#);
    if matches!(toc.position, TocPosition::Left | TocPosition::Right) {
      self.push_ch('2'); // `toc2` roughly means "toc-aside", per dr src
    }
    let level = self.section_level_stack.last().copied().unwrap_or(0) + 2;
    self.push([r#""><h"#, &num_str!(level), r#" id=""#, id, r#"-title">"#]);
    self.push_str(&toc.title);
    self.push([r#"</h"#, &num_str!(level), ">"]);
  }

  fn exit_toc(&mut self, _toc: &TableOfContents) {
    self.push_str("</nav>");
    self.on_toc_exit();
    self.state.ephemeral.remove(&InTableOfContents);
  }

  fn enter_toc_level(&mut self, level: u8, _nodes: &[TocNode]) {
    self.push(["<ol class=\"toc-list level-", &num_str!(level), "\">"]);
  }

  fn exit_toc_level(&mut self, _level: u8, _nodes: &[TocNode]) {
    self.push_str("</ol>");
  }

  fn enter_toc_node(&mut self, node: &TocNode) {
    HtmlBackend::enter_toc_node(self, node);
  }

  fn exit_toc_node(&mut self, node: &TocNode) {
    HtmlBackend::exit_toc_node(self, node);
  }

  fn exit_toc_content(&mut self) {
    self.push_str("</a>");
  }

  fn enter_book_part(&mut self, part: &Part) {
    let mut section_tag = OpenTag::without_id("section", &part.title.meta.attrs);
    section_tag.push_class("doc-section");
    section_tag.push_class("level-0");
    self.push_open_tag(section_tag);
  }
  fn exit_book_part(&mut self, _part: &Part) {
    self.push_str("</section>");
  }

  fn enter_book_part_title(&mut self, title: &PartTitle) {
    self.push_str("<h1");
    if let Some(id) = &title.id {
      self.push([r#" id=""#, id]);
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

  fn enter_preamble(&mut self, doc_has_title: bool, _blocks: &[Block]) {
    if doc_has_title {
      self.push_str(r#"<section id="preamble" aria-label="Preamble">"#);
    }
  }

  fn exit_preamble(&mut self, doc_has_title: bool, _blocks: &[Block]) {
    if doc_has_title {
      self.push_str("</section>");
    }
  }

  fn enter_section(&mut self, section: &Section) {
    self.section_level_stack.push(section.level);
    let mut section_tag = OpenTag::without_id("section", &section.meta.attrs);
    section_tag.push_class("doc-section");
    section_tag.push_class(format!("level-{}", section.level));
    self.enter_section_state(section);
    self.push_open_tag(section_tag);
  }

  fn exit_section(&mut self, section: &Section) {
    self.push_str("</section>");
    self.exit_section_state(section);
    self.section_level_stack.pop();
  }

  fn enter_section_heading(&mut self, section: &Section) {
    HtmlBackend::enter_section_heading(self, section);
  }

  fn exit_section_heading(&mut self, section: &Section) {
    HtmlBackend::exit_section_heading(self, section);
  }

  fn enter_compound_block_content(&mut self, _children: &[Block], _block: &Block) {}
  fn exit_compound_block_content(&mut self, _children: &[Block], _block: &Block) {}

  fn enter_simple_block_content(&mut self, block: &Block) {
    if block.context == BlockContext::BlockQuote {
      self.push_str("<p>");
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

  fn enter_sidebar_block(&mut self, block: &Block) {
    self.open_element("aside", &["sidebar"], &block.meta.attrs);
    self.render_buffered_block_title(block, true);
  }

  fn exit_sidebar_block(&mut self, _block: &Block) {
    self.push_str("</aside>");
  }

  fn enter_listing_block(&mut self, block: &Block) {
    let el = if block.has_title() { "figure" } else { "div" };
    self.open_element(el, &["listing-block"], &block.meta.attrs);
    self.render_buffered_block_title(block, false);
    self.push_str("<pre");
    let doc_lang = self.doc_meta.string("source-language");
    if block.meta.attrs.is_source() || doc_lang.is_some() {
      self.push_str(" class=\"highlight");
      if block.meta.attrs.has_option("nowrap") {
        self.push_str(" nowrap");
      }
      if block.meta.attrs.has_option("numbered") {
        self.push_str(" linenums");
      }
      self.push_str("\"><code");
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
  }

  fn exit_listing_block(&mut self, block: &Block) {
    if self.state.ephemeral.remove(&IsSourceBlock) {
      self.push_str("</code>");
    }
    self.push_str("</pre>");
    let close = if block.has_title() { "</figure>" } else { "</div>" };
    self.push_str(close);
  }

  fn enter_literal_block(&mut self, block: &Block) {
    let el = if block.has_title() { "section" } else { "div" };
    self.open_element(el, &["literal-block"], &block.meta.attrs);
    self.render_buffered_block_title(block, true);
    self.push_str("<pre>");
    self.newlines = Newlines::Preserve;
  }

  fn exit_literal_block(&mut self, block: &Block) {
    self.push_str("</pre>");
    let end = if block.has_title() { "</section>" } else { "</div>" };
    self.push_str(end);
    self.newlines = self.default_newlines;
  }

  fn enter_quoted_paragraph(&mut self, block: &Block) {
    self.open_element("div", &["quote-block"], &block.meta.attrs);
    self.render_buffered_block_title(block, false);
    self.push_str("<blockquote><p>");
  }

  fn exit_quoted_paragraph(&mut self, _block: &Block) {
    self.push_str("</blockquote></div>");
  }

  fn enter_quote_block(&mut self, block: &Block, _has_attribution: bool) {
    let el = if block.has_title() { "section" } else { "div" };
    self.open_element(el, &["quote-block"], &block.meta.attrs);
    if block.has_title() {
      self.render_buffered_block_title(block, true);
    }
    self.push_str("<blockquote>");
  }

  fn exit_quote_block(&mut self, block: &Block, _has_attribution: bool) {
    let el = if block.has_title() { "</section>" } else { "</div>" };
    self.push(["</blockquote>", el]);
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

  fn enter_quote_cite(&mut self, _block: &Block, has_attribution: bool) {
    if has_attribution {
      self.push_str(r#", "#);
    } else {
      self.push_str(r#"<footer>&#8212; <cite>"#);
    }
  }

  fn exit_quote_cite(&mut self, _block: &Block, _has_attribution: bool) {
    self.push_str(r#"</cite></footer>"#);
  }

  fn enter_verse_block(&mut self, block: &Block, has_attribution: bool) {
    self.open_element("div", &["verse-block"], &block.meta.attrs);
    self.render_buffered_block_title(block, false);
    if has_attribution {
      self.push_str(r#"<blockquote class="verse">"#);
    }
    self.push_str(r#"<pre class="verse">"#);
  }

  fn exit_verse_block(&mut self, block: &Block, has_attribution: bool) {
    if has_attribution {
      self.push_str("</blockquote>");
    }
    self.push_str(if block.has_title() { "</section>" } else { "</div>" });
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

  fn enter_open_block(&mut self, block: &Block) {
    if block.meta.attrs.special_sect() == Some(SpecialSection::Abstract) {
      let el = if block.has_title() { "section" } else { "div" };
      self.open_element(el, &["quote-block abstract"], &block.meta.attrs);
      self.render_buffered_block_title(block, true);
      self.push_str(r#"<blockquote>"#);
    } else {
      self.open_element("div", &["open-block"], &block.meta.attrs);
      self.render_buffered_block_title(block, true);
      self.push_str(r#"<div class="content">"#);
    }
  }

  fn exit_open_block(&mut self, block: &Block) {
    if block.meta.attrs.special_sect() == Some(SpecialSection::Abstract) {
      self.push_str("</blockquote>");
      self.push_str(if block.has_title() { "</section>" } else { "</div>" });
    } else {
      self.push_str("</div></div>");
    }
  }

  fn enter_discrete_heading(&mut self, level: u8, id: Option<&str>, block: &Block) {
    self.push_enter_discrete_heading(level, id, block);
  }

  fn exit_discrete_heading(&mut self, level: u8, _id: Option<&str>, _block: &Block) {
    self.push_exit_discrete_heading(level);
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

  fn enter_callout_list(&mut self, block: &Block, _items: &[ListItem], _depth: u8) {
    self.html.truncate(self.html.len().saturating_sub(6));
    self.open_element("ol", &["callout-list arabic"], &block.meta.attrs);
  }

  fn exit_callout_list(&mut self, _block: &Block, _items: &[ListItem], _depth: u8) {
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
    self.push_str(r#"<dt>"#);
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

  fn enter_list_item_principal(&mut self, item: &ListItem, list_variant: ListVariant) {
    if let ListItemTypeMeta::Checklist(checked, _) = &item.type_meta {
      self.push_str(r#"<li class="task-list-item"><input class="task-list-item-checkbox" type="checkbox" disabled"#);
      self.push_str(if *checked { " checked>" } else { ">" });
    } else if list_variant != ListVariant::Callout || self.doc_meta.icon_mode() == IconMode::Text {
      self.push_str("<li>");
    } else {
      self.push_str("<li>");
    }
    if item
      .blocks
      .first()
      .is_some_and(|b| !matches!(b.content, BlockContent::List { .. }))
    {
      self.push_str("<p>");
    }
  }

  fn exit_list_item_principal(&mut self, item: &ListItem, _list_variant: ListVariant) {
    if item
      .blocks
      .first()
      .is_some_and(|b| !matches!(b.content, BlockContent::List { .. }))
    {
      self.push_str("</p>");
    }
  }

  fn enter_list_item_blocks(&mut self, _blocks: &[Block], _item: &ListItem, _variant: ListVariant) {
  }

  fn exit_list_item_blocks(&mut self, _blocks: &[Block], _item: &ListItem, _variant: ListVariant) {
    self.push_str("</li>");
  }

  fn enter_paragraph_block(&mut self, block: &Block) {
    if self.doc_meta.get_doctype() == DocType::Inline {
      return;
    }
    if block.has_title() {
      if block.meta.attrs.special_sect() == Some(SpecialSection::Abstract) {
        self.push_str(r#"<section class="quote-block abstract">"#);
      } else {
        self.push_str(r#"<section class="paragraph">"#);
      }
      self.render_buffered_block_title(block, true);
    }
    if !self
      .state
      .ephemeral
      .contains(&VisitingSimpleTermDescription)
    {
      if block.meta.attrs.special_sect() == Some(SpecialSection::Abstract) {
        self.push_str("<blockquote>");
      } else {
        self.open_element("p", &[], &block.meta.attrs);
      }
    }
  }

  fn exit_paragraph_block(&mut self, block: &Block) {
    if self.doc_meta.get_doctype() == DocType::Inline {
      return;
    }
    if !self
      .state
      .ephemeral
      .contains(&VisitingSimpleTermDescription)
    {
      if block.meta.attrs.special_sect() == Some(SpecialSection::Abstract) {
        self.push_str("</blockquote>");
      } else {
        self.push_str("</p>");
      }
    }
    if block.has_title() {
      self.push_str(r#"</section>"#);
    }
    self.state.ephemeral.remove(&VisitingSimpleTermDescription);
  }

  fn asciidoc_table_cell_backend(&mut self) -> Self {
    let mut backend = Self::default();
    backend.state.footnotes = Rc::clone(&self.state.footnotes);
    backend.state.in_asciidoc_table_cell = true;
    backend
  }

  fn visit_asciidoc_table_cell_result(&mut self, cell_backend: Self) {
    self.state.in_asciidoc_table_cell = false;
    self.html.push_str(&cell_backend.into_result().unwrap());
  }

  fn enter_table(&mut self, table: &Table, block: &Block) {
    let el = if block.has_title() { "figure" } else { "div" };
    self.open_element(el, &["table-block"], &block.meta.attrs);
    self.render_buffered_block_title(block, false);
    let mut tag = OpenTag::new("table", &NoAttrs);
    finish_open_table_tag(&mut tag, block, &self.doc_meta);
    self.push_open_tag(tag);
    backend::html::table::push_colgroup(&mut self.html, table, block);
  }

  fn exit_table(&mut self, _table: &Table, block: &Block) {
    self.push_str("</table>");
    self.push_str(if block.has_title() { "</figure>" } else { "</div>" });
  }

  fn enter_table_section(&mut self, section: TableSection) {
    HtmlBackend::enter_table_section(self, section);
  }

  fn exit_table_section(&mut self, section: TableSection) {
    HtmlBackend::exit_table_section(self, section);
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
      _ => {}
    }
  }

  fn exit_cell_paragraph(&mut self, cell: &Cell, section: TableSection) {
    match (section, &cell.content) {
      (TableSection::Header, _) => self.push_str(" "),
      (_, CellContent::Emphasis(_)) => self.push_str("</em>"),
      (_, CellContent::Monospace(_)) => self.push_str("</code>"),
      (_, CellContent::Strong(_)) => self.push_str("</strong>"),
      _ => {}
    }
    if cell.content.has_multiple_paras() {
      self.push_str("</p>");
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

  fn visit_spaced_dashes(&mut self, len: u8, _adjacent_newline: AdjacentNewline) {
    if len == 2 {
      self.push_str("&#8201;&#8211;&#8201;");
    } else {
      self.push_str("&#8201;&#8212;&#8201;");
    }
  }

  fn visit_inline_specialchar(&mut self, char: &SpecialCharKind) {
    HtmlBackend::visit_inline_specialchar(self, char);
  }

  fn visit_symbol(&mut self, kind: SymbolKind) {
    match kind {
      SymbolKind::Copyright => self.push_str("&#169;"),
      SymbolKind::Registered => self.push_str("&#174;"),
      SymbolKind::Trademark => self.push_str("&#8482;"),
      SymbolKind::EmDash => self.push_str("&#8211;"),
      SymbolKind::TripleDash => self.push_str("&#8212;&#8203;"),
      SymbolKind::Ellipsis => self.push_str("&#8230;&#8203;"),
      SymbolKind::SingleRightArrow => self.push_str("&#8594;"),
      SymbolKind::DoubleRightArrow => self.push_str("&#8658;"),
      SymbolKind::SingleLeftArrow => self.push_str("&#8592;"),
      SymbolKind::DoubleLeftArrow => self.push_str("&#8656;"),
    }
  }

  fn enter_inline_quote(&mut self, kind: QuoteKind) {
    let q = quote_entities(self.doc_meta.str_or("lang", "en"));
    self.push_str(match kind {
      QuoteKind::Single => q[0],
      QuoteKind::Double => q[2],
    });
  }

  fn exit_inline_quote(&mut self, kind: QuoteKind) {
    let q = quote_entities(self.doc_meta.str_or("lang", "en"));
    self.push_str(match kind {
      QuoteKind::Single => q[1],
      QuoteKind::Double => q[3],
    });
  }

  fn visit_curly_quote(&mut self, kind: CurlyKind) {
    HtmlBackend::visit_curly_quote(self, kind);
  }

  fn visit_multichar_whitespace(&mut self, whitespace: &str) {
    self.push_str(whitespace);
  }

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

  fn enter_text_span(&mut self, attrs: &AttrList) {
    match attrs.roles.first().map(|r| r.src.as_str()) {
      Some("line-through" | "strike") => {
        let mut attrs = attrs.clone();
        attrs
          .roles
          .retain(|r| r.src != "line-through" && r.src != "strike");
        self.open_element("s", &[], &attrs);
      }
      Some("del") => {
        let mut attrs = attrs.clone();
        attrs.roles.retain(|r| r.src != "del");
        self.open_element("del", &[], &attrs);
      }
      Some("ins") => {
        let mut attrs = attrs.clone();
        attrs.roles.retain(|r| r.src != "ins");
        self.open_element("ins", &[], &attrs);
      }
      _ => {
        self.open_element("span", &[], attrs);
      }
    }
  }

  fn exit_text_span(&mut self, attrs: &AttrList) {
    match attrs.roles.first().map(|r| r.src.as_str()) {
      Some("line-through" | "strike") => self.push_str("</s>"),
      Some("del") => self.push_str("</del>"),
      Some("ins") => self.push_str("</ins>"),
      _ => self.push_str("</span>"),
    }
  }

  fn enter_xref(&mut self, target: &SourceString, _has_reftext: bool, kind: XrefKind) {
    HtmlBackend::enter_xref(self, target, kind);
  }

  fn exit_xref(&mut self, _target: &SourceString, _has_reftext: bool, _kind: XrefKind) {
    HtmlBackend::exit_xref(self);
  }

  fn enter_xref_text(&mut self, is_biblio: bool) {
    HtmlBackend::enter_xref_text(self, is_biblio);
  }

  fn exit_xref_text(&mut self, is_biblio: bool) {
    HtmlBackend::exit_xref_text(self, is_biblio);
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
    if !self.state.ephemeral.contains(&InTableOfContents) {
      self.push(["<a id=\"", id, "\" aria-hidden=\"true\"></a>"]);
    }
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

  fn visit_callout(&mut self, callout: Callout) {
    if !self.html.ends_with(' ') {
      self.push_ch(' ');
    }
    self.push([r#"<b class="conum">"#, &num_str!(callout.number), "</b>"]);
  }

  fn visit_callout_tuck(&mut self, comment: &str) {
    self.push_str(comment);
  }

  fn visit_linebreak(&mut self) {
    self.push_str("<br>\n");
  }

  fn visit_button_macro(&mut self, text: &SourceString) {
    self.push([r#"<b class="button">"#, text, "</b>"])
  }

  fn visit_icon_macro(&mut self, target: &SourceString, attrs: &AttrList) {
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
        self.push_str(r#"<b class="icon"#);
        attrs.roles.iter().for_each(|role| {
          self.push_str(" ");
          self.push_str(role);
        });
        if let Some(title) = attrs.named("title") {
          self.push([r#"" title=""#, title]);
        }
        self.push_str(r#"">["#);
        self.push_str(attrs.named("alt").unwrap_or(target));
        self.push_str("]</b>");
      }
      IconMode::Image => {
        self.push_str(r#"<img src=""#);
        self.push_icon_uri(target, None);
        self.push_str(r#"" alt=""#);
        self.push_str(attrs.named("alt").unwrap_or(target));
        self.push_ch('"');
        self.push_named_attr("width", attrs);
        self.push_named_attr("title", attrs);
        self.push_str(r#" class="icon"#);
        attrs.roles.iter().for_each(|role| {
          self.push_str(" ");
          self.push_str(role);
        });
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
        attrs.roles.iter().for_each(|role| {
          self.push_str(" ");
          self.push_str(role);
        });
        self.push_ch('"');
        self.push_named_attr("title", attrs);
        self.push_str("></i>");
      }
    }
    if has_link {
      self.push_str("</a>");
    }
  }

  fn visit_image_macro(&mut self, target: &SourceString, attrs: &AttrList) {
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
    let mut style = String::new();
    if let Some(align) = attrs.named("align") {
      style.push_str("text-align: ");
      style.push_str(align);
      if attrs.named("float").is_some() {
        style.push_str("; ");
      } else {
        style.push(';');
      }
    }
    if let Some(float) = attrs.named("float") {
      style.push_str("float: ");
      style.push_str(float);
      style.push(';');
    }
    if !style.is_empty() {
      self.html.pop();
      self.push([" style=\"", &style, "\">"]);
    }
    if !attrs.roles.is_empty() {
      self.html.pop();
      self.push_str(" class=\"");
      for role in attrs.roles.iter() {
        self.push_str(&role.src);
        self.push_ch(' ');
      }
      self.html.pop();
      self.push_str("\">");
    }
    if attrs.named("loading") == Some("lazy") {
      self.html.pop();
      self.push_str(r#" loading="lazy">"#);
    }
    if with_link {
      self.push_str("</a>");
    }
  }

  fn visit_keyboard_macro(&mut self, keys: &[&str]) {
    if keys.len() > 1 {
      self.push_str(r#"<kbd class="keyseq">"#);
    }
    for (idx, key) in keys.iter().enumerate() {
      if idx > 0 {
        self.push_ch('+');
      }
      self.push(["<kbd class=\"key\">", key, "</kbd>"]);
    }
    if keys.len() > 1 {
      self.push_str("</kbd>");
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

  fn visit_menu_macro(&mut self, items: &[SourceString]) {
    HtmlBackend::visit_menu_macro(self, items);
  }

  fn enter_admonition_block(&mut self, kind: AdmonitionKind, block: &Block) {
    let classes = &["admonition-block", kind.lowercase_str()];
    if matches!(kind, AdmonitionKind::Note | AdmonitionKind::Tip) {
      self.open_element("aside", classes, &block.meta.attrs);
    } else {
      self.open_element("section", classes, &block.meta.attrs);
    }
    self.html.pop();
    self.push_str(r#" role=""#);
    match kind {
      AdmonitionKind::Tip => self.push_str("doc-tip\">"),
      AdmonitionKind::Note => self.push_str("note\">"),
      _ => self.push_str("doc-notice\">"),
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

  fn exit_admonition_block(&mut self, kind: AdmonitionKind, block: &Block) {
    if !matches!(block.content, BlockContent::Compound(_)) {
      self.push_str("</p>");
    }
    if matches!(kind, AdmonitionKind::Note | AdmonitionKind::Tip) {
      self.push_str("</aside>");
    } else {
      self.push_str("</section>");
    }
  }

  fn enter_image_block(&mut self, img_target: &SourceString, img_attrs: &AttrList, block: &Block) {
    let el = if block.has_title() || img_attrs.named("title").is_some() {
      "figure"
    } else {
      "div"
    };
    let mut open_tag = OpenTag::new(el, &block.meta.attrs);
    open_tag.push_class("image-block");
    if let Some(align) = img_attrs.named("align") {
      open_tag.push_style(format!("text-align: {align}"));
    }
    if let Some(float) = img_attrs.named("float") {
      open_tag.push_style(format!("float: {float}"));
    }
    self.push_open_tag(open_tag);

    let mut has_link = false;
    if let Some(href) = &block
      .meta
      .attrs
      .named("link")
      .or_else(|| img_attrs.named("link"))
      .or_else(|| {
        if self.doc_meta.str("html5s-image-default-link") == Some("self") {
          Some("self")
        } else {
          None
        }
      })
      .filter(|h| *h != "none")
    {
      self.push_str(r#"<a class="image"#);
      let self_link = if *href == img_target.src || *href == "self" {
        self.push_str(" bare");
        true
      } else {
        false
      };
      let href = if *href == "self" { &img_target.src } else { *href };
      self.push([r#"" href=""#, href, r#"""#]);
      if let Some(window) = img_attrs.named("window") {
        self.push([r#" target=""#, window, "\""]);
        if window == "_blank" || img_attrs.has_option("noopener") {
          self.push_str(" rel=\"noopener");
        }
        if img_attrs.has_option("nofollow") {
          self.push_str(" nofollow\"");
        } else {
          self.push_ch('"');
        }
      } else if img_attrs.has_option("noopener") {
        self.push_str(" rel=\"noopener\"");
      } else if img_attrs.has_option("nofollow") {
        self.push_str(" rel=\"nofollow\"");
      }
      if self_link {
        let label = self.doc_meta.string_or(
          "html5s-image-self-link-label",
          "Open the image in full size",
        );
        self.push([r#" title=""#, &label, r#"" aria-label=""#, &label, r#"">"#]);
      } else {
        self.push_ch('>');
      }
      has_link = true;
    }
    self.render_image(img_target, img_attrs, true);
    if img_attrs.named("loading") == Some("lazy") {
      self.html.pop();
      self.push_str(r#" loading="lazy">"#);
    }
    if has_link {
      self.push_str("</a>");
    }
  }

  fn exit_image_block(&mut self, _img_target: &SourceString, img_attrs: &AttrList, block: &Block) {
    if let Some(title) = img_attrs.named("title") {
      self.render_block_title(title, block, false);
    } else if block.has_title() {
      let title = self.take_buffer();
      self.render_block_title(&title, block, false);
    }
    if block.has_title() || img_attrs.named("title").is_some() {
      self.push_str("</figure>");
    } else {
      self.push_str("</div>");
    };
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

  fn enter_footnote(&mut self, id: Option<&SourceString>, has_content: bool) {
    if has_content {
      self.start_buffering();
      return;
    }
    if let Some(prev_ref_num) = self.prev_footnote_ref_num(id) {
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
    let num = self.state.footnotes.borrow().len() + 1;
    let footnote = self.swap_take_buffer();
    let nums = num.to_string();
    self.push_str(r#"<a class="footnote-ref" id="_footnoteref_"#);
    self.push([&nums, r##"" href="#_footnote_"##, &nums]);
    self.push([
      r#"" title="View footnote "#,
      &nums,
      r#"" role="doc-noteref">"#,
    ]);
    self
      .state
      .footnotes
      .borrow_mut()
      .push((id.map(|id| id.to_string()), footnote));
    self.push(["[", &nums, "]</a>"]);
  }

  fn enter_meta_title(&mut self) {
    self.start_buffering();
  }

  fn exit_meta_title(&mut self) {
    self.stop_buffering();
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

  fn render_footnotes(&mut self) {
    self.push_str(r#"<section class="footnotes" aria-label="Footnotes" role="doc-endnotes">"#);
    self.push_str(r#"<hr><ol class="footnotes">"#);
    let footnotes = mem::take(&mut self.state.footnotes);
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
    self.state.footnotes = footnotes;
  }

  fn render_author_detail(&mut self, author: &Author, index: usize) {
    self.push_str(r#"<span class="author" id="author"#);
    if index > 0 {
      self.push_str(&(index + 1).to_string());
    }
    self.push_str("\">");
    self.push_str(&author.fullname());
    self.push_str("</span><br>");
    if let Some(email) = &author.email {
      self.push_str(r#"<span class="email" id="email"#);
      if index > 0 {
        self.push_str(&(index + 1).to_string());
      }
      self.push(["\"><a href=\"mailto:", email, "\">", email, "</a></span>"]);
      if index == 0 {
        self.push_str("<br>");
      }
    }
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
        self.push_str(",</span> ");
      } else {
        self.push_str("</span>");
      }
    }
    if let Some(revdate) = self.doc_meta.string("revdate") {
      self.push([
        r#"<time id="revdate" datetime=""#,
        &backend::time::format_date_str(&revdate, "%Y-%m-%d").unwrap_or_else(|| revdate.clone()),
        r#"">"#,
        &revdate,
        "</time>",
      ]);
    }
    if let Some(revremark) = self.doc_meta.string("revremark") {
      self.push([r#"<br><span id="revremark">"#, &revremark, "</span>"]);
    }
    self.push_str("</div>");
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

  if !block.meta.attrs.has_option("autowidth") && explicit_width.is_none() {
    tag.push_class("stretch");
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

  if let Some(float) = block.meta.attrs.named("float") {
    tag.push_style(format!("float: {float};"));
  }
}

fn quote_entities(lang: &str) -> [&'static str; 4] {
  match lang {
    "bs" | "fi" | "sv" => ["&#x2019;", "&#x2019;", "&#x201d;", "&#x201d;"],
    "cs" | "da" | "de" | "is" | "lt" | "sl" | "sk" | "sr" => {
      ["&#x201a;", "&#x2018;", "&#x201e;", "&#x201c;"]
    }
    "nl" => ["&#x201a;", "&#x2019;", "&#x201e;", "&#x201d;"],
    "hu" | "pl" | "ro" => ["&#x00ab;", "&#x00bb;", "&#x201e;", "&#x201d;"],
    _ => ["&#x2018;", "&#x2019;", "&#x201c;", "&#x201d;"],
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
  fn doc_meta(&self) -> &DocumentMeta {
    &self.doc_meta
  }
}

const fn incr(num: &mut usize) -> usize {
  *num += 1;
  *num
}

pub(crate) use backend::num_str;
