use std::collections::HashSet;
use std::fmt::Write;

use crate::internal::*;
use EphemeralState::*;

#[derive(Debug, Default)]
pub struct AsciidoctorHtml {
  pub(crate) html: String,
  pub(crate) alt_html: String,
  pub(crate) footnotes: Vec<(String, String)>,
  pub(crate) doc_attrs: AttrEntries,
  pub(crate) doc_type: DocType,
  pub(crate) fig_caption_num: usize,
  pub(crate) table_caption_num: usize,
  pub(crate) opts: Opts,
  pub(crate) list_stack: Vec<bool>,
  pub(crate) default_newlines: Newlines,
  pub(crate) newlines: Newlines,
  pub(crate) state: HashSet<EphemeralState>,
  pub(crate) autogen_conum: u8,
  pub(crate) render_doc_header: bool,
  pub(crate) in_asciidoc_table_cell: bool,
  pub(crate) section_nums: [u16; 5],
  pub(crate) section_num_levels: isize,
}

impl Backend for AsciidoctorHtml {
  type Output = String;
  type Error = Infallible;

  fn enter_document(&mut self, document: &Document, opts: Opts) {
    self.opts = opts;
    self.doc_attrs = document.attrs.clone();
    self.doc_type = document.get_type();
    self.section_num_levels = document.attrs.isize("sectnumlevels").unwrap_or(3);
    if document.attrs.is_true("hardbreaks-option") {
      self.default_newlines = Newlines::JoinWithBreak
    }
    if opts.doc_type == DocType::Inline || self.in_asciidoc_table_cell {
      self.render_doc_header = document.attrs.is_true("showtitle");
      return;
    }
    self.render_doc_header = !document.attrs.is_false("showtitle");
    self.push_str(r#"<!DOCTYPE html><html"#);
    if !document.attrs.is_true("nolang") {
      self.push([r#" lang=""#, document.attrs.str_or("lang", "en"), "\""]);
    }
    let encoding = document.attrs.str_or("encoding", "UTF-8");
    self.push([r#"><head><meta charset=""#, encoding, r#"">"#]);
    self.push_str(r#"<meta http-equiv="X-UA-Compatible" content="IE=edge">"#);
    self.push_str(r#"<meta name="viewport" content="width=device-width, initial-scale=1.0">"#);
    if !document.attrs.is_true("reproducible") {
      self.push_str(r#"<meta name="generator" content="Asciidork">"#);
    }
    if let Some(appname) = document.attrs.str("app-name") {
      self.push([r#"<meta name="application-name" content=""#, appname, "\">"]);
    }
    if let Some(desc) = document.attrs.str("description") {
      self.push([r#"<meta name="description" content=""#, desc, "\">"]);
    }
    if let Some(keywords) = document.attrs.str("keywords") {
      self.push([r#"<meta name="keywords" content=""#, keywords, "\">"]);
    }
    if let Some(copyright) = document.attrs.str("copyright") {
      self.push([r#"<meta name="copyright" content=""#, copyright, "\">"]);
    }
    self.render_favicon(&document.attrs);
    self.render_authors(&document.header);
    self.render_title(document, &document.attrs);
    // TODO: stylesheets
    self.push([r#"</head><body class=""#, opts.doc_type.to_str()]);
    match document.toc.as_ref().map(|toc| &toc.position) {
      Some(TocPosition::Left) => self.push_str(" toc2 toc-left"),
      Some(TocPosition::Right) => self.push_str(" toc2 toc-right"),
      _ => {}
    }
    self.push_str("\">");
  }

  fn exit_document(&mut self, _document: &Document) {
    if !self.footnotes.is_empty() {
      self.render_footnotes();
    }
    if self.opts.doc_type != DocType::Inline && !self.in_asciidoc_table_cell {
      self.push_str("</body></html>");
    }
  }

  fn enter_document_header(&mut self, _doc_header: &DocHeader) {
    if self.render_doc_header {
      self.push_str(r#"<div id="header">"#)
    }
  }

  fn exit_document_header(&mut self, _doc_header: &DocHeader) {
    if self.render_doc_header {
      self.push_str("</div>");
    }
  }

  fn enter_document_title(&mut self, _doc_title: &DocTitle) {
    if self.render_doc_header {
      self.push_str("<h1>")
    } else {
      self.start_buffering();
    }
  }

  fn exit_document_title(&mut self, _doc_title: &DocTitle) {
    if self.render_doc_header {
      self.push_str("</h1>");
    } else {
      self.take_buffer(); // discard
    }
  }

  fn visit_document_authors(&mut self, authors: &[Author]) {
    if self.render_doc_header && !authors.is_empty() {
      self.push_str(r#"<div class="details">"#);
      for (idx, author) in authors.iter().enumerate() {
        self.push_str(r#"<span id="author"#);
        if idx > 0 {
          self.push_str(&num_str!(idx + 1));
        }
        self.push([r#"" class="author">"#, &author.first_name]);
        if let Some(middle_name) = &author.middle_name {
          self.push([" ", middle_name]);
        }
        self.push([" ", &author.last_name, r#"</span><br>"#]);
        if let Some(email) = &author.email {
          self.push_str(r#"<span id="email"#);
          if idx > 0 {
            self.push_str(&num_str!(idx + 1));
          }
          self.push([r#"" class="email"><a href="mailto:"#, &email]);
          self.push([r#"">"#, &email, "</a></span><br>"]);
        }
      }
      self.push_str("</div>");
    }
  }

  fn enter_toc(&mut self, toc: &TableOfContents) {
    self.push_str(r#"<div id="toc" class="toc"#);
    if matches!(toc.position, TocPosition::Left | TocPosition::Right) {
      self.push_ch('2'); // `toc2` roughly means "toc-aside", per dr src
    }
    self.push_str(r#""><div id="toctitle">"#);
    self.push_str(&toc.title);
    self.push_str("</div>");
  }

  fn exit_toc(&mut self, _toc: &TableOfContents) {
    self.push_str("</div>");
  }

  fn enter_toc_level(&mut self, level: u8, _nodes: &[TocNode]) {
    self.push(["<ul class=\"sectlevel", &num_str!(level), "\">"]);
  }

  fn exit_toc_level(&mut self, _level: u8, _nodes: &[TocNode]) {
    self.push_str("</ul>");
  }

  fn enter_toc_node(&mut self, node: &TocNode) {
    self.push_str("<li><a href=\"#");
    if let Some(id) = &node.id {
      self.push_str(id);
    }
    self.push_str("\">")
  }

  fn exit_toc_node(&mut self, _node: &TocNode) {
    self.push_str("</li>");
  }

  fn exit_toc_content(&mut self, _content: &[InlineNode]) {
    self.push_str("</a>");
  }

  fn enter_preamble(&mut self, _blocks: &[Block]) {
    self.push_str(r#"<div id="preamble"><div class="sectionbody">"#);
  }

  fn exit_preamble(&mut self, _blocks: &[Block]) {
    self.push_str("</div></div>");
  }

  fn enter_section(&mut self, section: &Section) {
    let mut classes = SmallVec::<[&str; 5]>::from_slice(&[section::class(section)]);
    if let Some(roles) = section.meta.attrs.as_ref().map(|a| &a.roles) {
      roles.iter().for_each(|role| classes.push(role));
    }
    self.open_element("div", &classes, None);
  }

  fn exit_section(&mut self, section: &Section) {
    if section.level == 1 {
      self.push_str("</div>");
    }
    self.push_str("</div>");
  }

  fn enter_section_heading(&mut self, section: &Section) {
    let level_str = num_str!(section.level + 1);
    if let Some(id) = &section.id {
      self.push(["<h", &level_str, r#" id=""#, id, "\">"]);
    } else {
      self.push(["<h", &level_str, ">"]);
    }
    if self.should_number_section(section) {
      let prefix = section::number_prefix(section.level, &mut self.section_nums);
      self.push_str(&prefix);
    }
  }

  fn exit_section_heading(&mut self, section: &Section) {
    let level_str = num_str!(section.level + 1);
    self.push(["</h", &level_str, ">"]);
    if section.level == 1 {
      self.push_str(r#"<div class="sectionbody">"#);
    }
  }

  fn enter_block_title(&mut self, _title: &[InlineNode], _block: &Block) {
    self.start_buffering();
  }

  fn exit_block_title(&mut self, _title: &[InlineNode], _block: &Block) {
    self.stop_buffering();
  }

  fn enter_compound_block_content(&mut self, _children: &[Block], _block: &Block) {}
  fn exit_compound_block_content(&mut self, _children: &[Block], _block: &Block) {}

  fn enter_simple_block_content(&mut self, _children: &[InlineNode], block: &Block) {
    if block.context == BlockContext::Verse {
      self.newlines = Newlines::Preserve;
    } else if block.has_attr_option("hardbreaks") {
      self.newlines = Newlines::JoinWithBreak;
    }
  }

  fn exit_simple_block_content(&mut self, _children: &[InlineNode], _block: &Block) {
    self.newlines = self.default_newlines;
  }

  fn enter_sidebar_block(&mut self, block: &Block, _content: &BlockContent) {
    self.open_element("div", &["sidebarblock"], block.meta.attrs.as_ref());
    self.push_str(r#"<div class="content">"#);
  }

  fn exit_sidebar_block(&mut self, _block: &Block, _content: &BlockContent) {
    self.push_str("</div></div>");
  }

  fn enter_listing_block(&mut self, block: &Block, _content: &BlockContent) {
    self.open_element("div", &["listingblock"], block.meta.attrs.as_ref());
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

  fn exit_listing_block(&mut self, _block: &Block, _content: &BlockContent) {
    if self.state.remove(&IsSourceBlock) {
      self.push_str("</code>");
    }
    self.push_str("</pre></div></div>");
    self.newlines = self.default_newlines;
  }

  fn enter_literal_block(&mut self, block: &Block, _content: &BlockContent) {
    self.open_element("div", &["literalblock"], block.meta.attrs.as_ref());
    self.push_str(r#"<div class="content"><pre>"#);
    self.newlines = Newlines::Preserve;
  }

  fn exit_literal_block(&mut self, _block: &Block, _content: &BlockContent) {
    self.push_str("</pre></div></div>");
    self.newlines = self.default_newlines;
  }

  fn enter_passthrough_block(&mut self, _block: &Block, _content: &BlockContent) {}
  fn exit_passthrough_block(&mut self, _block: &Block, _content: &BlockContent) {}

  fn enter_quoted_paragraph(&mut self, block: &Block, _attr: &str, _cite: Option<&str>) {
    self.open_element("div", &["quoteblock"], block.meta.attrs.as_ref());
    self.render_block_title(&block.meta);
    self.push_str("<blockquote>");
  }

  fn exit_quoted_paragraph(&mut self, _block: &Block, attr: &str, cite: Option<&str>) {
    self.exit_attributed(BlockContext::BlockQuote, Some(attr), cite);
  }

  fn enter_quote_block(&mut self, block: &Block, _content: &BlockContent) {
    self.open_element("div", &["quoteblock"], block.meta.attrs.as_ref());
    self.render_block_title(&block.meta);
    self.push_str("<blockquote>");
  }

  fn exit_quote_block(&mut self, block: &Block, _content: &BlockContent) {
    if let Some(attrs) = &block.meta.attrs {
      self.exit_attributed(
        block.context,
        attrs.str_positional_at(1),
        attrs.str_positional_at(2),
      );
    } else {
      self.exit_attributed(block.context, None, None);
    }
  }

  fn enter_verse_block(&mut self, block: &Block, _content: &BlockContent) {
    self.open_element("div", &["verseblock"], block.meta.attrs.as_ref());
    self.render_block_title(&block.meta);
    self.push_str(r#"<pre class="content">"#);
  }

  fn exit_verse_block(&mut self, block: &Block, content: &BlockContent) {
    self.exit_quote_block(block, content)
  }

  fn enter_example_block(&mut self, block: &Block, _content: &BlockContent) {
    if block.has_attr_option("collapsible") {
      self.open_element("details", &[], block.meta.attrs.as_ref());
      if block.has_attr_option("open") {
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
      self.open_element("div", &["exampleblock"], block.meta.attrs.as_ref());
    }
    self.push_str(r#"<div class="content">"#);
  }

  fn exit_example_block(&mut self, block: &Block, _content: &BlockContent) {
    if block.has_attr_option("collapsible") {
      self.push_str("</div></details>");
    } else {
      self.push_str("</div></div>");
    }
  }

  fn enter_open_block(&mut self, block: &Block, _content: &BlockContent) {
    self.open_element("div", &["openblock"], block.meta.attrs.as_ref());
    self.push_str(r#"<div class="content">"#);
  }

  fn exit_open_block(&mut self, _block: &Block, _content: &BlockContent) {
    self.push_str("</div></div>");
  }

  fn enter_discrete_heading(&mut self, level: u8, id: Option<&str>, block: &Block) {
    let level_str = num_str!(level + 1);
    if let Some(id) = id {
      self.push(["<h", &level_str, r#" id=""#, id, "\""]);
    } else {
      self.push(["<h", &level_str]);
    }
    self.push_str(r#" class="discrete"#);
    if let Some(roles) = block.meta.attrs.as_ref().map(|a| &a.roles) {
      for role in roles {
        self.push_ch(' ');
        self.push_str(role);
      }
    }
    self.push_str("\">");
  }

  fn exit_discrete_heading(&mut self, level: u8, _id: Option<&str>, _block: &Block) {
    self.push(["</h", &num_str!(level + 1), ">"]);
  }

  fn enter_unordered_list(&mut self, block: &Block, items: &[ListItem], _depth: u8) {
    let attrs = block.meta.attrs.as_ref();
    let custom = attrs.and_then(|a| a.unordered_list_custom_marker_style());
    let interactive = attrs.map(|a| a.has_option("interactive")).unwrap_or(false);
    self.list_stack.push(interactive);
    let mut wrap_classes = SmallVec::<[&str; 3]>::from_slice(&["ulist"]);
    let mut list_classes = SmallVec::<[&str; 2]>::new();
    if let Some(custom) = custom {
      wrap_classes.push(custom);
      list_classes.push(custom);
    }
    if items.iter().any(ListItem::is_checklist) {
      wrap_classes.push("checklist");
      list_classes.push("checklist");
    }
    self.open_element("div", &wrap_classes, block.meta.attrs.as_ref());
    self.render_block_title(&block.meta);
    self.push_str("<ul");
    self.add_classes(&list_classes);
    self.push_ch('>');
  }

  fn exit_unordered_list(&mut self, _block: &Block, _items: &[ListItem], _depth: u8) {
    self.list_stack.pop();
    self.push_str("</ul></div>");
  }

  fn enter_callout_list(&mut self, block: &Block, _items: &[ListItem], _depth: u8) {
    self.autogen_conum = 1;
    self.open_element("div", &["colist arabic"], block.meta.attrs.as_ref());
    self.push_str(if self.doc_attrs.get("icons").is_some() { "<table>" } else { "<ol>" });
  }

  fn exit_callout_list(&mut self, _block: &Block, _items: &[ListItem], _depth: u8) {
    self.push_str(if self.doc_attrs.get("icons").is_some() {
      "</table></div>"
    } else {
      "</ol></div>"
    });
  }

  fn enter_description_list(&mut self, block: &Block, _items: &[ListItem], _depth: u8) {
    self.open_element("div", &["dlist"], block.meta.attrs.as_ref());
    self.push_str("<dl>");
  }

  fn exit_description_list(&mut self, _block: &Block, _items: &[ListItem], _depth: u8) {
    self.push_str("</dl></div>");
  }

  fn enter_description_list_term(&mut self, _item: &ListItem) {
    self.push_str(r#"<dt class="hdlist1">"#);
  }

  fn exit_description_list_term(&mut self, _item: &ListItem) {
    self.push_str("</dt>");
  }

  fn enter_description_list_description(&mut self, blocks: &[Block], _item: &ListItem) {
    if blocks.first().map_or(false, |block| {
      block.context == BlockContext::Paragraph && matches!(block.content, BlockContent::Simple(_))
    }) {
      self.state.insert(VisitingSimpleTermDescription);
    }
    self.push_str("<dd>");
  }

  fn exit_description_list_description(&mut self, _blocks: &[Block], _item: &ListItem) {
    self.push_str("</dd>");
  }

  fn enter_ordered_list(&mut self, block: &Block, items: &[ListItem], depth: u8) {
    self.list_stack.push(false);
    let attrs = block.meta.attrs.as_ref();
    let custom = attrs.and_then(|attrs| attrs.ordered_list_custom_number_style());
    let list_type = custom
      .and_then(list_type_from_class)
      .unwrap_or_else(|| list_type_from_depth(depth));
    let class = custom.unwrap_or_else(|| list_class_from_depth(depth));
    let classes = &["olist", class];
    self.open_element("div", classes, block.meta.attrs.as_ref());
    self.render_block_title(&block.meta);
    self.push([r#"<ol class=""#, class, "\""]);

    if list_type != "1" {
      self.push([" type=\"", list_type, "\""]);
    }

    if let Some(attr_start) = attrs.and_then(|attrs| attrs.named("start")) {
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

    if block.has_attr_option("reversed") {
      self.push_str(" reversed>");
    } else {
      self.push_str(">");
    }
  }

  fn exit_ordered_list(&mut self, _block: &Block, _items: &[ListItem], _depth: u8) {
    self.list_stack.pop();
    self.push_str("</ol></div>");
  }

  fn enter_list_item_principal(&mut self, item: &ListItem, list_variant: ListVariant) {
    if list_variant != ListVariant::Callout || self.doc_attrs.get("icons").is_none() {
      self.push_str("<li><p>");
      self.render_checklist_item(item);
    } else {
      self.push_str("<tr><td>");
      let n = item.marker.callout_num().unwrap_or(self.autogen_conum);
      self.autogen_conum = n + 1;
      if self.doc_attrs.str("icons") == Some("font") {
        self.push_callout_number_font(n);
      } else {
        self.push_callout_number_img(n);
      }
      self.push_str("</td><td>");
    }
  }

  fn exit_list_item_principal(&mut self, _item: &ListItem, list_variant: ListVariant) {
    if list_variant != ListVariant::Callout || self.doc_attrs.get("icons").is_none() {
      self.push_str("</p>");
    } else {
      self.push_str("</td>");
    }
  }

  fn enter_list_item_blocks(&mut self, _: &[Block], _: &ListItem, _: ListVariant) {}

  fn exit_list_item_blocks(&mut self, _blocks: &[Block], _items: &ListItem, variant: ListVariant) {
    if variant != ListVariant::Callout || self.doc_attrs.get("icons").is_none() {
      self.push_str("</li>");
    } else {
      self.push_str("</tr>");
    }
  }

  fn enter_paragraph_block(&mut self, block: &Block) {
    if self.doc_type != DocType::Inline {
      if !self.state.contains(&VisitingSimpleTermDescription) {
        self.open_element("div", &["paragraph"], block.meta.attrs.as_ref());
        self.render_block_title(&block.meta);
      }
      self.push_str("<p>");
    }
  }

  fn exit_paragraph_block(&mut self, _block: &Block) {
    if self.doc_type != DocType::Inline {
      self.push_str("</p>");
      if !self.state.contains(&VisitingSimpleTermDescription) {
        self.push_str("</div>");
      }
      self.state.remove(&VisitingSimpleTermDescription);
    }
  }

  fn enter_table(&mut self, table: &Table, block: &Block) {
    self.open_table_element(block);
    self.table_caption(block);
    self.push_str("<colgroup>");
    for width in table.col_widths.distribute() {
      self.push_str("<col");
      if let DistributedColWidth::Percentage(width) = width {
        write!(self.html, r#" style="width: {}%;""#, width).unwrap();
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
      ..Self::default()
    }
  }

  fn visit_asciidoc_table_cell_result(&mut self, result: Result<Self::Output, Self::Error>) {
    self.html.push_str(&result.unwrap());
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
    self.open_cell(cell, section);
  }

  fn exit_table_cell(&mut self, cell: &Cell, section: TableSection) {
    self.close_cell(cell, section);
  }

  fn enter_cell_paragraph(&mut self, cell: &Cell, section: TableSection) {
    self.open_cell_paragraph(cell, section);
  }

  fn exit_cell_paragraph(&mut self, cell: &Cell, section: TableSection) {
    self.close_cell_paragraph(cell, section);
  }

  fn enter_inline_italic(&mut self, _children: &[InlineNode]) {
    self.push_str("<em>");
  }

  fn exit_inline_italic(&mut self, _children: &[InlineNode]) {
    self.push_str("</em>");
  }

  fn visit_thematic_break(&mut self, block: &Block) {
    self.open_element("hr", &[], block.meta.attrs.as_ref());
  }

  fn visit_page_break(&mut self, _block: &Block) {
    self.push_str(r#"<div style="page-break-after: always;"></div>"#);
  }

  fn visit_inline_text(&mut self, text: &str) {
    self.push_str(text);
  }

  fn visit_joining_newline(&mut self) {
    match self.newlines {
      Newlines::JoinWithSpace => self.push_ch(' '),
      Newlines::JoinWithBreak => self.push_str("<br> "),
      Newlines::Preserve => self.push_str("\n"),
    }
  }

  fn enter_text_span(&mut self, attrs: &AttrList, _children: &[InlineNode]) {
    self.open_element("span", &[], Some(attrs));
  }

  fn exit_text_span(&mut self, _attrs: &AttrList, _children: &[InlineNode]) {
    self.push_str("</span>");
  }

  fn enter_xref(&mut self, id: &str, _target: Option<&[InlineNode]>) {
    self.push(["<a href=\"#", id, "\">"]);
  }

  fn exit_xref(&mut self, _id: &str, _target: Option<&[InlineNode]>) {
    self.push_str("</a>");
  }

  fn visit_missing_xref(&mut self, id: &str) {
    self.push(["[", id, "]"]);
  }

  fn visit_callout(&mut self, callout: Callout) {
    if !self.html.ends_with(' ') {
      self.push_ch(' ');
    }
    let icons = self.doc_attrs.get("icons");
    match icons {
      Some(AttrValue::Bool(true)) => self.push_callout_number_img(callout.number),
      Some(AttrValue::String(icons)) if icons == "font" => {
        self.push_callout_number_font(callout.number);
      }
      // TODO: asciidoctor also handles special `guard` case
      //   elsif ::Array === (guard = node.attributes['guard'])
      //     %(&lt;!--<b class="conum">(#{node.text})</b>--&gt;)
      // @see https://github.com/asciidoctor/asciidoctor/issues/3319
      _ => {
        self.push([r#"<b class="conum">("#, &num_str!(callout.number), ")</b>"]);
      }
    }
  }

  fn visit_callout_tuck(&mut self, comment: &str) {
    if self.doc_attrs.str("icons") != Some("font") {
      self.push_str(comment);
    }
  }

  fn visit_attribute_reference(&mut self, name: &str) {
    let val = self.doc_attrs.str(name).map(|s| s.to_string());
    if let Some(val) = val {
      self.push_str(&val);
      return;
    } else if let Some(builtin) = self.doc_attrs.builtin(name) {
      self.push_str(builtin);
      return;
    }
    match self.opts.attribute_missing {
      AttributeMissing::Drop => {}
      AttributeMissing::Skip => {
        self.push_str("{");
        self.push_str(name);
        self.push_ch('}');
      }
      AttributeMissing::Warn => {
        // TODO: warning
      }
    }
  }

  fn visit_linebreak(&mut self) {
    self.push_str("<br> ");
  }

  fn enter_inline_mono(&mut self, _children: &[InlineNode]) {
    self.push_str("<code>");
  }

  fn exit_inline_mono(&mut self, _children: &[InlineNode]) {
    self.push_str("</code>");
  }

  fn enter_inline_bold(&mut self, _children: &[InlineNode]) {
    self.push_str("<strong>");
  }

  fn exit_inline_bold(&mut self, _children: &[InlineNode]) {
    self.push_str("</strong>");
  }

  fn enter_inline_passthrough(&mut self, _children: &[InlineNode]) {}
  fn exit_inline_passthrough(&mut self, _children: &[InlineNode]) {}

  fn visit_button_macro(&mut self, text: &str) {
    self.push([r#"<b class="button">"#, text, "</b>"])
  }

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

  fn enter_link_macro(
    &mut self,
    target: &str,
    attrs: Option<&AttrList>,
    scheme: Option<UrlScheme>,
  ) {
    self.push_str("<a href=\"");
    if matches!(scheme, Some(UrlScheme::Mailto)) {
      self.push_str("mailto:");
    }
    self.push([target, "\""]);
    if attrs.is_none() && !matches!(scheme, Some(UrlScheme::Mailto)) {
      self.push_str(" class=\"bare\">");
    } else {
      self.push_ch('>');
    }
  }

  fn exit_link_macro(
    &mut self,
    target: &str,
    attrs: Option<&AttrList>,
    _scheme: Option<UrlScheme>,
  ) {
    if matches!(attrs.and_then(|a| a.positional.first()), None | Some(None)) {
      self.push_str(target);
    }
    self.push_str("</a>");
  }

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

  fn visit_inline_specialchar(&mut self, char: &SpecialCharKind) {
    match char {
      SpecialCharKind::Ampersand => self.push_str("&amp;"),
      SpecialCharKind::LessThan => self.push_str("&lt;"),
      SpecialCharKind::GreaterThan => self.push_str("&gt;"),
    }
  }

  fn enter_inline_highlight(&mut self, _children: &[InlineNode]) {
    self.push_str("<mark>");
  }

  fn exit_inline_highlight(&mut self, _children: &[InlineNode]) {
    self.push_str("</mark>");
  }

  fn enter_inline_subscript(&mut self, _children: &[InlineNode]) {
    self.push_str("<sub>");
  }

  fn exit_inline_subscript(&mut self, _children: &[InlineNode]) {
    self.push_str("</sub>");
  }

  fn enter_inline_superscript(&mut self, _children: &[InlineNode]) {
    self.push_str("<sup>");
  }

  fn exit_inline_superscript(&mut self, _children: &[InlineNode]) {
    self.push_str("</sup>");
  }

  fn enter_inline_quote(&mut self, kind: QuoteKind, _children: &[InlineNode]) {
    match kind {
      QuoteKind::Double => self.push_str("&#8220;"),
      QuoteKind::Single => self.push_str("&#8216;"),
    }
  }

  fn exit_inline_quote(&mut self, kind: QuoteKind, _children: &[InlineNode]) {
    match kind {
      QuoteKind::Double => self.push_str("&#8221;"),
      QuoteKind::Single => self.push_str("&#8217;"),
    }
  }

  fn visit_curly_quote(&mut self, kind: CurlyKind) {
    match kind {
      CurlyKind::LeftDouble => self.push_str("&#8221;"),
      CurlyKind::RightDouble => self.push_str("&#8220;"),
      CurlyKind::LeftSingle => self.push_str("&#8217;"),
      CurlyKind::RightSingle => self.push_str("&#8216;"),
      CurlyKind::LegacyImplicitApostrophe => self.push_str("&#8217;"),
    }
  }

  fn visit_inline_lit_mono(&mut self, text: &str) {
    self.push(["<code>", text, "</code>"]);
  }

  fn visit_multichar_whitespace(&mut self, _whitespace: &str) {
    self.push_ch(' ');
  }

  fn enter_admonition_block(&mut self, kind: AdmonitionKind, block: &Block) {
    let classes = &["admonitionblock", kind.lowercase_str()];
    self.open_element("div", classes, block.meta.attrs.as_ref());
    self.push_str(r#"<table><tr><td class="icon"><div class="title">"#);
    self.push_str(kind.str());
    self.push_str(r#"</div></td><td class="content">"#);
    self.render_block_title(&block.meta);
  }

  fn exit_admonition_block(&mut self, _kind: AdmonitionKind, _block: &Block) {
    self.push_str(r#"</td></tr></table></div>"#);
  }

  fn enter_image_block(&mut self, img_target: &str, img_attrs: &AttrList, block: &Block) {
    let alt = img_attrs.str_positional_at(0).unwrap_or({
      if let Some(captures) = REMOVE_FILE_EXT.captures(img_target) {
        captures.get(1).unwrap().as_str()
      } else {
        img_target
      }
    });
    self.open_element("div", &["imageblock"], block.meta.attrs.as_ref());
    self.push_str(r#"<div class="content">"#);
    let mut has_link = false;
    if let Some(href) = &block.named_attr("link") {
      self.push([r#"<a class="image" href=""#, *href, r#"">"#]);
      has_link = true;
    }
    self.push([r#"<img src=""#, img_target, r#"" alt=""#, alt, "\""]);
    if let Some(width) = img_attrs.str_positional_at(1) {
      self.push([r#" width=""#, width, "\""]);
    }
    if let Some(height) = img_attrs.str_positional_at(2) {
      self.push([r#" height=""#, height, "\""]);
    }
    self.push_ch('>');
    if has_link {
      self.push_str("</a>");
    }
    self.push_str(r#"</div>"#);
  }

  fn exit_image_block(&mut self, block: &Block) {
    let prefix = if self.doc_attrs.is_false("figure-caption") {
      None
    } else {
      self.fig_caption_num += 1;
      Some(Cow::Owned(format!("Figure {}. ", self.fig_caption_num)))
    };
    self.render_prefixed_block_title(&block.meta, prefix);
    self.push_str(r#"</div>"#);
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
    self
      .doc_attrs
      .insert(name.to_string(), AttrEntry::new(value.clone()));
  }

  fn enter_footnote(&mut self, _id: Option<&str>, _content: &[InlineNode]) {
    self.start_buffering();
  }

  fn exit_footnote(&mut self, id: Option<&str>, _content: &[InlineNode]) {
    let footnote = self.take_buffer();
    let num = (self.footnotes.len() + 1).to_string();
    self.push_str(r#"<sup class="footnote""#);
    if let Some(id) = id {
      self.push([r#" id="_footnote_"#, id, "\""]);
    }
    self.push_str(r#">[<a id="_footnoteref_"#);
    self.push([&num, r##"" class="footnote" href="#_footnotedef_"##, &num]);
    self.push([r#"" title="View footnote.">"#, &num, "</a>]</sup>"]);
    let id = id.unwrap_or(&num);
    self.footnotes.push((id.to_string(), footnote));
  }

  fn into_result(self) -> Result<Self::Output, Self::Error> {
    Ok(self.html)
  }

  fn result(&self) -> Result<&Self::Output, Self::Error> {
    Ok(&self.html)
  }
}

impl AsciidoctorHtml {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn into_string(self) -> String {
    self.html
  }

  pub(crate) fn push_str(&mut self, s: &str) {
    self.html.push_str(s);
  }

  pub(crate) fn push_buffered(&mut self) {
    let mut buffer = String::new();
    mem::swap(&mut buffer, &mut self.alt_html);
    self.push_str(&buffer);
  }

  pub(crate) fn push_ch(&mut self, c: char) {
    self.html.push(c);
  }

  pub(crate) fn push<const N: usize>(&mut self, strs: [&str; N]) {
    for s in strs {
      self.push_str(s);
    }
  }

  fn source_lang<'a>(&self, block: &'a Block) -> Option<Cow<'a, str>> {
    match block
      .meta
      .attrs
      .as_ref()
      .map(|a| (a.str_positional_at(0), a.str_positional_at(1)))
      .unwrap_or((None, None))
    {
      (None | Some("source"), Some(lang)) => Some(Cow::Borrowed(lang)),
      _ => self
        .doc_attrs
        .str("source-language")
        .map(|s| Cow::Owned(s.to_string())),
    }
  }

  fn render_block_title(&mut self, meta: &ChunkMeta) {
    if meta.title.is_some() {
      self.push_str(r#"<div class="title">"#);
      self.push_buffered();
      self.push_str("</div>");
    }
  }

  fn render_prefixed_block_title(&mut self, meta: &ChunkMeta, prefix: Option<Cow<str>>) {
    if meta.title.is_some() {
      self.push_str(r#"<div class="title">"#);
      if let Some(prefix) = prefix {
        self.push_str(&prefix);
      }
      self.push_buffered();
      self.push_str("</div>");
    }
  }

  pub(crate) fn open_element(&mut self, element: &str, classes: &[&str], attrs: Option<&AttrList>) {
    self.push_ch('<');
    self.push_str(element);
    if let Some(id) = attrs.as_ref().and_then(|a| a.id.as_ref()) {
      self.push_str(" id=\"");
      self.push_str(id);
      self.push_ch('"');
    }
    if !classes.is_empty() || attrs.as_ref().map_or(false, |a| !a.roles.is_empty()) {
      self.push_str(" class=\"");
      for class in classes {
        self.push_str(class);
        self.push_ch(' ');
      }
      if let Some(roles) = attrs.as_ref().map(|a| &a.roles) {
        for role in roles {
          self.push_str(role);
          self.push_ch(' ');
        }
      }
      self.html.pop();
      self.push_ch('"');
    }
    self.push_ch('>');
  }

  fn render_footnotes(&mut self) {
    self.push_str(r#"<div id="footnotes"><hr>"#);
    let footnotes = mem::take(&mut self.footnotes);
    for (index, (_id, footnote)) in footnotes.iter().enumerate() {
      let num = &(index + 1).to_string();
      self.push_str(r#"<div class="footnote" id="_footnotedef_"#);
      self.push([num, r##""><a href="#_footnoteref_"##, num, "\">"]);
      self.push([num, "</a>. ", footnote, "</div>"]);
    }
    self.push_str(r#"</div>"#);
    self.footnotes = footnotes;
  }

  fn render_favicon(&mut self, attrs: &AttrEntries) {
    match attrs.get("favicon") {
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

  fn render_authors(&mut self, header: &Option<DocHeader>) {
    let Some(DocHeader { authors, .. }) = header else {
      return;
    };
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

  fn render_title(&mut self, document: &Document, attrs: &AttrEntries) {
    self.push_str(r#"<title>"#);
    if let Some(title) = attrs.str("title") {
      self.push_str(title);
    } else if let Some(title) = document
      .header
      .as_ref()
      .and_then(|header| header.title.as_ref())
    {
      for s in title.heading.plain_text() {
        self.push_str(s);
      }
    } else {
      self.push_str("Untitled");
    }
    self.push_str(r#"</title>"#);
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

  fn add_classes(&mut self, classes: &[&str]) {
    if !classes.is_empty() {
      self.push_str(" class=\"");
      for class in classes.iter().take(classes.len() - 1) {
        self.push_str(class);
        self.push_ch(' ');
      }
      self.push_str(classes.last().unwrap());
      self.push_ch('"');
    }
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

  fn start_buffering(&mut self) {
    mem::swap(&mut self.html, &mut self.alt_html);
  }

  fn stop_buffering(&mut self) {
    mem::swap(&mut self.html, &mut self.alt_html);
  }

  fn take_buffer(&mut self) -> String {
    mem::swap(&mut self.alt_html, &mut self.html);
    let mut buffered = String::new();
    mem::swap(&mut buffered, &mut self.alt_html);
    buffered
  }

  // TODO: handle embedding images, data-uri, etc., this is a naive impl
  // @see https://github.com/jaredh159/asciidork/issues/7
  fn push_icon_uri(&mut self, name: &str, prefix: Option<&str>) {
    // PERF: we could work to prevent all these allocations w/ some caching
    // these might get rendered many times in a given document
    let icondir = self
      .doc_attrs
      .str_or("iconsdir", "./images/icons")
      .to_string();
    let ext = self.doc_attrs.str_or("icontype", "png").to_string();
    self.push([&icondir, "/", prefix.unwrap_or(""), name, ".", &ext]);
  }

  fn push_callout_number_font(&mut self, num: u8) {
    let n_str = &num_str!(num);
    self.push([r#"<i class="conum" data-value=""#, n_str, r#""></i>"#]);
    self.push([r#"<b>("#, n_str, ")</b>"]);
  }

  fn push_callout_number_img(&mut self, num: u8) {
    let n_str = &num_str!(num);
    self.push_str(r#"<img src=""#);
    self.push_icon_uri(n_str, Some("callouts/"));
    self.push([r#"" alt=""#, n_str, r#"">"#]);
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

pub(crate) use num_str;

lazy_static! {
  pub static ref REMOVE_FILE_EXT: Regex = Regex::new(r"^(.*)\.[^.]+$").unwrap();
}
