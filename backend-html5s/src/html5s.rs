#![allow(dead_code)]
#![allow(unused_variables)]

use crate::internal::*;

#[derive(Debug, Default)]
pub struct Html5s {
  doc_meta: DocumentMeta,
  html: String,
}

impl Backend for Html5s {
  type Output = String;
  type Error = std::convert::Infallible;
  const OUTFILESUFFIX: &'static str = ".html";

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
    todo!()
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
  }

  fn exit_paragraph_block(&mut self, block: &Block) {
    if self.doc_meta.get_doctype() == DocType::Inline {
      return;
    }
  }

  fn enter_sidebar_block(&mut self, block: &Block) {
    todo!()
  }

  fn exit_sidebar_block(&mut self, block: &Block) {
    todo!()
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
    todo!()
  }

  fn exit_listing_block(&mut self, block: &Block) {
    todo!()
  }

  fn enter_literal_block(&mut self, block: &Block) {
    todo!()
  }

  fn exit_literal_block(&mut self, block: &Block) {
    todo!()
  }

  fn enter_passthrough_block(&mut self, block: &Block) {
    todo!()
  }

  fn exit_passthrough_block(&mut self, block: &Block) {
    todo!()
  }

  fn enter_image_block(&mut self, img_target: &SourceString, img_attrs: &AttrList, block: &Block) {
    todo!()
  }

  fn exit_image_block(&mut self, img_target: &SourceString, img_attrs: &AttrList, block: &Block) {
    todo!()
  }

  fn enter_admonition_block(&mut self, kind: asciidork_backend::AdmonitionKind, block: &Block) {
    todo!()
  }

  fn exit_admonition_block(&mut self, kind: asciidork_backend::AdmonitionKind, block: &Block) {
    todo!()
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
    todo!()
  }

  fn exit_meta_title(&mut self) {
    todo!()
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
    todo!()
  }

  fn visit_multichar_whitespace(&mut self, whitespace: &str) {
    todo!()
  }

  fn visit_button_macro(&mut self, text: &SourceString) {
    todo!()
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
    todo!()
  }

  fn enter_inline_passthrough(&mut self) {
    todo!()
  }

  fn exit_inline_passthrough(&mut self) {
    todo!()
  }

  fn enter_inline_highlight(&mut self) {
    todo!()
  }

  fn exit_inline_highlight(&mut self) {
    todo!()
  }

  fn enter_inline_subscript(&mut self) {
    todo!()
  }

  fn exit_inline_subscript(&mut self) {
    todo!()
  }

  fn enter_inline_superscript(&mut self) {
    todo!()
  }

  fn exit_inline_superscript(&mut self) {
    todo!()
  }

  fn enter_inline_quote(&mut self, kind: QuoteKind) {
    todo!()
  }

  fn exit_inline_quote(&mut self, kind: QuoteKind) {
    todo!()
  }

  fn enter_footnote(&mut self, id: Option<&SourceString>, has_content: bool) {
    todo!()
  }

  fn exit_footnote(&mut self, id: Option<&SourceString>, has_content: bool) {
    todo!()
  }

  fn enter_text_span(&mut self, attrs: &AttrList) {
    todo!()
  }

  fn exit_text_span(&mut self, attrs: &AttrList) {
    todo!()
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
    todo!()
  }

  fn visit_biblio_anchor(&mut self, id: &str, reftext: Option<&str>) {
    todo!()
  }

  fn visit_symbol(&mut self, kind: SymbolKind) {
    todo!()
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
}

impl HtmlBuf for Html5s {
  fn htmlbuf(&mut self) -> &mut String {
    &mut self.html
  }
}
