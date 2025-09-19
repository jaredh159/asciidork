#![allow(dead_code)]
#![allow(unused_variables)]

use crate::internal::*;

#[derive(Debug, Default)]
pub struct Html5s {
  doc_meta: DocumentMeta,
  html: String,
  alt_html: String,
  in_source_block: bool,
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
  }

  fn exit_document(&mut self, document: &Document) {}

  fn enter_header(&mut self) {}

  fn exit_header(&mut self) {}

  fn enter_content(&mut self) {}

  fn exit_content(&mut self) {}

  fn enter_footer(&mut self) {}

  fn exit_footer(&mut self) {}

  fn visit_document_attribute_decl(&mut self, name: &str, value: &AttrValue) {
    _ = self.doc_meta.insert_doc_attr(name, value.clone());
  }

  fn enter_preamble(&mut self, doc_has_title: bool, blocks: &[Block]) {
    todo!()
  }

  fn exit_preamble(&mut self, doc_has_title: bool, blocks: &[Block]) {
    todo!()
  }

  fn enter_document_title(&mut self) {
    todo!()
  }

  fn exit_document_title(&mut self) {
    todo!()
  }

  fn enter_section(&mut self, section: &Section) {
    todo!()
  }

  fn exit_section(&mut self, section: &Section) {
    todo!()
  }

  fn enter_section_heading(&mut self, section: &Section) {
    todo!()
  }

  fn exit_section_heading(&mut self, section: &Section) {
    todo!()
  }

  fn enter_book_part(&mut self, part: &Part) {
    todo!()
  }

  fn exit_book_part(&mut self, part: &Part) {
    todo!()
  }

  fn enter_book_part_title(&mut self, title: &PartTitle) {
    todo!()
  }

  fn exit_book_part_title(&mut self, title: &PartTitle) {
    todo!()
  }

  fn enter_book_part_intro(&mut self, part: &Part) {
    todo!()
  }

  fn exit_book_part_intro(&mut self, part: &Part) {
    todo!()
  }

  fn enter_book_part_intro_content(&mut self, part: &Part) {
    todo!()
  }

  fn exit_book_part_intro_content(&mut self, part: &Part) {
    todo!()
  }

  fn enter_paragraph_block(&mut self, block: &Block) {
    if self.doc_meta.get_doctype() == DocType::Inline {
      return;
    }
    if block.meta.title.is_some() {
      self.push_str(r#"<section class="paragraph">"#);
    }
    self.render_buffered_block_title(block);
    // if block.meta.attrs.special_sect() == Some(SpecialSection::Abstract) {
    //   self.push_str("<blockquote>");
    // } else {
    // self.push_str("<p>");
    self.open_element("p", &[], &block.meta.attrs);
    // }
  }

  fn exit_paragraph_block(&mut self, block: &Block) {
    if self.doc_meta.get_doctype() == DocType::Inline {
      return;
    }
    // if block.meta.attrs.special_sect() == Some(SpecialSection::Abstract) {
    //   self.push_str("</blockquote>\n");
    // } else {
    self.push_str("</p>");
    if block.meta.title.is_some() {
      self.push_str(r#"</section>"#);
    }
    // }
  }

  fn enter_sidebar_block(&mut self, block: &Block) {
    self.open_element("aside", &["sidebar"], &block.meta.attrs);
    // self.render_buffered_block_title(block);
  }

  fn exit_sidebar_block(&mut self, block: &Block) {
    self.push_str("</aside>");
  }

  fn enter_open_block(&mut self, block: &Block) {
    todo!()
  }

  fn exit_open_block(&mut self, block: &Block) {
    todo!()
  }

  fn enter_example_block(&mut self, block: &Block) {
    todo!()
  }

  fn exit_example_block(&mut self, block: &Block) {
    todo!()
  }

  fn enter_quote_block(&mut self, block: &Block, has_attribution: bool) {
    todo!()
  }

  fn exit_quote_block(&mut self, block: &Block, has_attribution: bool) {
    todo!()
  }

  fn enter_verse_block(&mut self, block: &Block, has_attribution: bool) {
    todo!()
  }

  fn exit_verse_block(&mut self, block: &Block, has_attribution: bool) {
    todo!()
  }

  fn enter_listing_block(&mut self, block: &Block) {
    self.open_element("div", &["listingblock"], &block.meta.attrs);
    // self.render_buffered_block_title(block);
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

  fn exit_listing_block(&mut self, _block: &Block) {
    // if self.state.remove(&IsSourceBlock) {
    if self.in_source_block {
      self.in_source_block = false;
      self.push_str("</code>");
    }
    self.push_str("</pre></div>");
    // self.newlines = self.default_newlines;
  }

  fn enter_literal_block(&mut self, block: &Block) {
    todo!()
  }

  fn exit_literal_block(&mut self, block: &Block) {
    todo!()
  }

  fn enter_passthrough_block(&mut self, block: &Block) {}
  fn exit_passthrough_block(&mut self, block: &Block) {}

  fn enter_image_block(&mut self, img_target: &SourceString, img_attrs: &AttrList, block: &Block) {
    todo!()
  }

  fn exit_image_block(&mut self, img_target: &SourceString, img_attrs: &AttrList, block: &Block) {
    todo!()
  }

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

  fn exit_admonition_block(&mut self, _kind: AdmonitionKind, _block: &Block) {
    self.push_str(r#"</td></tr></table></div>"#);
  }

  fn enter_quote_attribution(&mut self, block: &Block, has_cite: bool) {
    todo!()
  }

  fn exit_quote_attribution(&mut self, block: &Block, has_cite: bool) {
    todo!()
  }

  fn enter_quote_cite(&mut self, block: &Block, has_attribution: bool) {
    todo!()
  }

  fn exit_quote_cite(&mut self, block: &Block, has_attribution: bool) {
    todo!()
  }

  fn enter_quoted_paragraph(&mut self, block: &Block) {
    todo!()
  }

  fn exit_quoted_paragraph(&mut self, block: &Block) {
    todo!()
  }

  fn enter_discrete_heading(&mut self, level: u8, id: Option<&str>, block: &Block) {
    todo!()
  }

  fn exit_discrete_heading(&mut self, level: u8, id: Option<&str>, block: &Block) {
    todo!()
  }

  fn enter_unordered_list(&mut self, block: &Block, items: &[ListItem], depth: u8) {
    todo!()
  }

  fn exit_unordered_list(&mut self, block: &Block, items: &[ListItem], depth: u8) {
    todo!()
  }

  fn enter_ordered_list(&mut self, block: &Block, items: &[ListItem], depth: u8) {
    todo!()
  }

  fn exit_ordered_list(&mut self, block: &Block, items: &[ListItem], depth: u8) {
    todo!()
  }

  fn enter_callout_list(&mut self, block: &Block, items: &[ListItem], depth: u8) {
    todo!()
  }

  fn exit_callout_list(&mut self, block: &Block, items: &[ListItem], depth: u8) {
    todo!()
  }

  fn enter_description_list(&mut self, block: &Block, items: &[ListItem], depth: u8) {
    todo!()
  }

  fn exit_description_list(&mut self, block: &Block, items: &[ListItem], depth: u8) {
    todo!()
  }

  fn enter_description_list_term(&mut self, item: &ListItem) {
    todo!()
  }

  fn exit_description_list_term(&mut self, item: &ListItem) {
    todo!()
  }

  fn enter_description_list_description(&mut self, item: &ListItem) {
    todo!()
  }

  fn exit_description_list_description(&mut self, item: &ListItem) {
    todo!()
  }

  fn enter_description_list_description_text(&mut self, text: &Block, item: &ListItem) {
    todo!()
  }

  fn exit_description_list_description_text(&mut self, text: &Block, item: &ListItem) {
    todo!()
  }

  fn enter_description_list_description_block(&mut self, block: &Block, item: &ListItem) {
    todo!()
  }

  fn exit_description_list_description_block(&mut self, block: &Block, item: &ListItem) {
    todo!()
  }

  fn enter_list_item_principal(&mut self, item: &ListItem, variant: ListVariant) {
    todo!()
  }

  fn exit_list_item_principal(&mut self, item: &ListItem, variant: ListVariant) {
    todo!()
  }

  fn enter_list_item_blocks(&mut self, blocks: &[Block], item: &ListItem, variant: ListVariant) {
    todo!()
  }

  fn exit_list_item_blocks(&mut self, blocks: &[Block], item: &ListItem, variant: ListVariant) {
    todo!()
  }

  fn enter_table(&mut self, table: &Table, block: &Block) {
    todo!()
  }

  fn exit_table(&mut self, table: &Table, block: &Block) {
    todo!()
  }

  fn enter_table_section(&mut self, section: TableSection) {
    todo!()
  }

  fn exit_table_section(&mut self, section: TableSection) {
    todo!()
  }

  fn enter_table_row(&mut self, row: &Row, section: TableSection) {
    todo!()
  }

  fn exit_table_row(&mut self, row: &Row, section: TableSection) {
    todo!()
  }

  fn enter_table_cell(&mut self, cell: &Cell, section: TableSection) {
    todo!()
  }

  fn exit_table_cell(&mut self, cell: &Cell, section: TableSection) {
    todo!()
  }

  fn enter_cell_paragraph(&mut self, cell: &Cell, section: TableSection) {
    todo!()
  }

  fn exit_cell_paragraph(&mut self, cell: &Cell, section: TableSection) {
    todo!()
  }

  fn asciidoc_table_cell_backend(&mut self) -> Self {
    todo!()
  }

  fn visit_asciidoc_table_cell_result(&mut self, cell_backend: Self) {
    todo!()
  }

  fn enter_meta_title(&mut self) {
    self.start_buffering();
  }

  fn exit_meta_title(&mut self) {
    self.stop_buffering();
  }

  fn enter_simple_block_content(&mut self, block: &Block) {}

  fn exit_simple_block_content(&mut self, block: &Block) {}

  fn enter_compound_block_content(&mut self, children: &[Block], block: &Block) {
    todo!()
  }

  fn exit_compound_block_content(&mut self, children: &[Block], block: &Block) {
    todo!()
  }

  fn visit_thematic_break(&mut self, block: &Block) {
    todo!()
  }

  fn visit_page_break(&mut self, block: &Block) {
    todo!()
  }

  fn visit_inline_text(&mut self, text: &str) {
    self.push_str(text);
  }

  fn visit_joining_newline(&mut self) {
    self.push_ch('\n');
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
    todo!()
  }

  fn visit_callout_tuck(&mut self, comment: &str) {
    todo!()
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
    todo!()
  }

  fn exit_footnote(&mut self, id: Option<&SourceString>, has_content: bool) {
    todo!()
  }

  fn enter_text_span(&mut self, attrs: &AttrList) {
    self.open_element("span", &[], attrs);
  }

  fn exit_text_span(&mut self, attrs: &AttrList) {
    self.push_str("</span>");
  }

  fn enter_xref(&mut self, target: &SourceString, has_reftext: bool, kind: XrefKind) {
    todo!()
  }

  fn exit_xref(&mut self, target: &SourceString, has_reftext: bool, kind: XrefKind) {
    todo!()
  }

  fn visit_missing_xref(
    &mut self,
    target: &SourceString,
    kind: XrefKind,
    doc_title: Option<&DocTitle>,
  ) {
    todo!()
  }

  fn visit_inline_anchor(&mut self, id: &str) {
    self.push(["<a id=\"", id, "\" aria-hidden=\"true\"></a>"]);
  }

  fn visit_biblio_anchor(&mut self, id: &str, reftext: Option<&str>) {
    todo!()
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
    todo!()
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

  fn render_buffered_block_title(&mut self, block: &Block) {
    if block.meta.title.is_some() {
      let buf = self.take_buffer();
      self.render_block_title(&buf, block);
    }
  }

  fn render_block_title(&mut self, title: &str, block: &Block) {
    // self.push_str(r#"<section class="paragraph">"#);
    self.push_str(r#"<h6 class="block-title">"#);
    // if let Some(custom_caption) = block.meta.attrs.named("caption") {
    //   self.push_str(custom_caption);
    // } else if let Some(caption) = block
    //   .context
    //   .caption_attr_name()
    //   .and_then(|attr_name| self.doc_meta.string(attr_name))
    // {
    //   self.push_str(&caption);
    //   self.push_ch(' ');
    //   let num = match block.context {
    //     BlockContext::Table => incr(&mut self.table_caption_num),
    //     BlockContext::Image => incr(&mut self.fig_caption_num),
    //     BlockContext::Example => incr(&mut self.example_caption_num),
    //     BlockContext::Listing => incr(&mut self.listing_caption_num),
    //     _ => unreachable!(),
    //   };
    //   self.push_str(&num.to_string());
    //   self.push_str(". ");
    // }
    self.push_str(title);
    self.push_str(r#"</h6>"#);
  }

  fn push_admonition_img(&mut self, kind: AdmonitionKind) {
    self.push_str(r#"<img src=""#);
    backend::html::util::push_icon_uri(self, kind.lowercase_str(), None);
    self.push([r#"" alt=""#, kind.str(), r#"">"#]);
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
