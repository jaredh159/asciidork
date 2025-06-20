use super::admonition::AdmonitionKind;
use crate::prelude::*;
use ast::prelude::*;

macro_rules! warn_unimplemented {
  ($x:ident) => {
    eprintln!(
      "WARN: Backend::{}(...) called but not implemented",
      stringify!($x)
    );
  };
}

pub trait Backend {
  type Output;
  type Error;

  const OUTFILESUFFIX: &'static str;

  fn set_job_attrs(attrs: &mut asciidork_core::JobAttrs);

  // document
  fn enter_document(&mut self, document: &Document);
  fn exit_document(&mut self, document: &Document);
  fn enter_header(&mut self);
  fn exit_header(&mut self);
  fn enter_content(&mut self);
  fn exit_content(&mut self);
  fn enter_footer(&mut self);
  fn exit_footer(&mut self);
  fn visit_document_attribute_decl(&mut self, name: &str, value: &AttrValue);
  fn enter_preamble(&mut self, doc_has_title: bool, blocks: &[Block]);
  fn exit_preamble(&mut self, doc_has_title: bool, blocks: &[Block]);
  fn enter_document_title(&mut self, nodes: &[InlineNode]);
  fn exit_document_title(&mut self, nodes: &[InlineNode]);

  // table of contents
  fn enter_toc(&mut self, _toc: &TableOfContents, _macro_block: Option<&Block>) {}
  fn exit_toc(&mut self, _toc: &TableOfContents) {}
  fn enter_toc_level(&mut self, _level: u8, _nodes: &[TocNode]) {}
  fn exit_toc_level(&mut self, _level: u8, _nodes: &[TocNode]) {}
  fn enter_toc_node(&mut self, _node: &TocNode) {}
  fn exit_toc_node(&mut self, _node: &TocNode) {}
  fn enter_toc_content(&mut self, _content: &[InlineNode]) {}
  fn exit_toc_content(&mut self, _content: &[InlineNode]) {}

  // sections
  fn enter_section(&mut self, section: &Section);
  fn exit_section(&mut self, section: &Section);
  fn enter_section_heading(&mut self, section: &Section);
  fn exit_section_heading(&mut self, section: &Section);

  // books
  fn enter_book_part(&mut self, part: &Part);
  fn exit_book_part(&mut self, part: &Part);
  fn enter_book_part_title(&mut self, title: &PartTitle);
  fn exit_book_part_title(&mut self, title: &PartTitle);
  fn enter_book_part_intro(&mut self, part: &Part);
  fn exit_book_part_intro(&mut self, part: &Part);
  fn enter_book_part_intro_content(&mut self, part: &Part);
  fn exit_book_part_intro_content(&mut self, part: &Part);

  // blocks contexts
  fn enter_paragraph_block(&mut self, block: &Block);
  fn exit_paragraph_block(&mut self, block: &Block);
  fn enter_sidebar_block(&mut self, block: &Block, content: &BlockContent);
  fn exit_sidebar_block(&mut self, block: &Block, content: &BlockContent);
  fn enter_open_block(&mut self, block: &Block, content: &BlockContent);
  fn exit_open_block(&mut self, block: &Block, content: &BlockContent);
  fn enter_example_block(&mut self, block: &Block, content: &BlockContent);
  fn exit_example_block(&mut self, block: &Block, content: &BlockContent);
  fn enter_quote_block(&mut self, block: &Block, content: &BlockContent);
  fn exit_quote_block(&mut self, block: &Block, content: &BlockContent);
  fn enter_verse_block(&mut self, block: &Block, content: &BlockContent);
  fn exit_verse_block(&mut self, block: &Block, content: &BlockContent);
  fn enter_listing_block(&mut self, block: &Block, content: &BlockContent);
  fn exit_listing_block(&mut self, block: &Block, content: &BlockContent);
  fn enter_literal_block(&mut self, block: &Block, content: &BlockContent);
  fn exit_literal_block(&mut self, block: &Block, content: &BlockContent);
  fn enter_passthrough_block(&mut self, block: &Block, content: &BlockContent);
  fn exit_passthrough_block(&mut self, block: &Block, content: &BlockContent);
  fn enter_image_block(&mut self, img_target: &SourceString, img_attrs: &AttrList, block: &Block);
  fn exit_image_block(&mut self, img_target: &SourceString, img_attrs: &AttrList, block: &Block);
  fn enter_admonition_block(&mut self, kind: AdmonitionKind, block: &Block);
  fn exit_admonition_block(&mut self, kind: AdmonitionKind, block: &Block);
  fn enter_quoted_paragraph(
    &mut self,
    block: &Block,
    attr: &SourceString,
    cite: Option<&SourceString>,
  );
  fn exit_quoted_paragraph(
    &mut self,
    block: &Block,
    attr: &SourceString,
    cite: Option<&SourceString>,
  );
  fn enter_discrete_heading(&mut self, level: u8, id: Option<&str>, block: &Block);
  fn exit_discrete_heading(&mut self, level: u8, id: Option<&str>, block: &Block);

  // lists
  fn enter_unordered_list(&mut self, block: &Block, items: &[ListItem], depth: u8);
  fn exit_unordered_list(&mut self, block: &Block, items: &[ListItem], depth: u8);
  fn enter_ordered_list(&mut self, block: &Block, items: &[ListItem], depth: u8);
  fn exit_ordered_list(&mut self, block: &Block, items: &[ListItem], depth: u8);
  fn enter_callout_list(&mut self, block: &Block, items: &[ListItem], depth: u8);
  fn exit_callout_list(&mut self, block: &Block, items: &[ListItem], depth: u8);
  fn enter_description_list(&mut self, block: &Block, items: &[ListItem], depth: u8);
  fn exit_description_list(&mut self, block: &Block, items: &[ListItem], depth: u8);
  fn enter_description_list_term(&mut self, term: &[InlineNode], item: &ListItem);
  fn exit_description_list_term(&mut self, term: &[InlineNode], item: &ListItem);
  fn enter_description_list_description(&mut self, item: &ListItem);
  fn exit_description_list_description(&mut self, item: &ListItem);
  fn enter_description_list_description_text(&mut self, text: &Block, item: &ListItem);
  fn exit_description_list_description_text(&mut self, text: &Block, item: &ListItem);
  fn enter_description_list_description_block(&mut self, block: &Block, item: &ListItem);
  fn exit_description_list_description_block(&mut self, block: &Block, item: &ListItem);
  fn enter_list_item_principal(&mut self, item: &ListItem, variant: ListVariant);
  fn exit_list_item_principal(&mut self, item: &ListItem, variant: ListVariant);
  fn enter_list_item_blocks(&mut self, blocks: &[Block], item: &ListItem, variant: ListVariant);
  fn exit_list_item_blocks(&mut self, blocks: &[Block], item: &ListItem, variant: ListVariant);

  // tables
  fn enter_table(&mut self, table: &Table, block: &Block);
  fn exit_table(&mut self, table: &Table, block: &Block);
  fn enter_table_section(&mut self, section: TableSection);
  fn exit_table_section(&mut self, section: TableSection);
  fn enter_table_row(&mut self, row: &Row, section: TableSection);
  fn exit_table_row(&mut self, row: &Row, section: TableSection);
  fn enter_table_cell(&mut self, cell: &Cell, section: TableSection);
  fn exit_table_cell(&mut self, cell: &Cell, section: TableSection);
  fn enter_cell_paragraph(&mut self, cell: &Cell, section: TableSection);
  fn exit_cell_paragraph(&mut self, cell: &Cell, section: TableSection);
  fn asciidoc_table_cell_backend(&mut self) -> Self;
  fn visit_asciidoc_table_cell_result(&mut self, cell_backend: Self);

  // block content
  fn enter_meta_title(&mut self, title: &[InlineNode]);
  fn exit_meta_title(&mut self, title: &[InlineNode]);
  fn enter_simple_block_content(&mut self, children: &[InlineNode], block: &Block);
  fn exit_simple_block_content(&mut self, children: &[InlineNode], block: &Block);
  fn enter_compound_block_content(&mut self, children: &[Block], block: &Block);
  fn exit_compound_block_content(&mut self, children: &[Block], block: &Block);
  fn visit_thematic_break(&mut self, block: &Block);
  fn visit_page_break(&mut self, block: &Block);

  /// inlines
  fn visit_inline_text(&mut self, text: &str);
  fn visit_joining_newline(&mut self);
  fn visit_curly_quote(&mut self, kind: CurlyKind);
  fn visit_multichar_whitespace(&mut self, whitespace: &str);
  fn visit_button_macro(&mut self, text: &SourceString);
  fn visit_menu_macro(&mut self, items: &[SourceString]);
  fn visit_image_macro(&mut self, target: &SourceString, attrs: &AttrList);
  fn visit_icon_macro(&mut self, target: &SourceString, attrs: &AttrList);

  fn visit_plugin_macro(&mut self, plugin_macro: &PluginMacro) {
    _ = plugin_macro;
    warn_unimplemented!(visit_plugin_macro);
  }

  fn visit_keyboard_macro(&mut self, keys: &[&str]) {
    _ = keys;
    warn_unimplemented!(visit_keyboard_macro);
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
    _ = (
      target,
      attrs,
      scheme,
      has_link_text,
      resolving_xref,
      blank_window_shorthand,
    );
    warn_unimplemented!(enter_link_macro);
  }

  fn exit_link_macro(
    &mut self,
    target: &SourceString,
    attrs: Option<&AttrList>,
    scheme: Option<UrlScheme>,
    resolving_xref: bool,
    has_link_text: bool,
  ) {
    _ = (target, attrs, scheme, resolving_xref, has_link_text);
    warn_unimplemented!(exit_link_macro);
  }

  fn visit_callout(&mut self, callout: Callout);
  fn visit_callout_tuck(&mut self, comment: &str);
  fn enter_inline_italic(&mut self, children: &[InlineNode]);
  fn exit_inline_italic(&mut self, children: &[InlineNode]);
  fn enter_inline_mono(&mut self, children: &[InlineNode]);
  fn exit_inline_mono(&mut self, children: &[InlineNode]);
  fn enter_inline_bold(&mut self, children: &[InlineNode]);
  fn exit_inline_bold(&mut self, children: &[InlineNode]);
  fn enter_inline_lit_mono(&mut self, children: &[InlineNode]);
  fn exit_inline_lit_mono(&mut self, children: &[InlineNode]);
  fn visit_inline_specialchar(&mut self, char: &SpecialCharKind);
  fn enter_inline_passthrough(&mut self, children: &[InlineNode]);
  fn exit_inline_passthrough(&mut self, children: &[InlineNode]);
  fn enter_inline_highlight(&mut self, children: &[InlineNode]);
  fn exit_inline_highlight(&mut self, children: &[InlineNode]);
  fn enter_inline_subscript(&mut self, children: &[InlineNode]);
  fn exit_inline_subscript(&mut self, children: &[InlineNode]);
  fn enter_inline_superscript(&mut self, children: &[InlineNode]);
  fn exit_inline_superscript(&mut self, children: &[InlineNode]);
  fn enter_inline_quote(&mut self, kind: QuoteKind, children: &[InlineNode]);
  fn exit_inline_quote(&mut self, kind: QuoteKind, children: &[InlineNode]);
  fn enter_footnote(&mut self, id: Option<&SourceString>, content: Option<&[InlineNode]>);
  fn exit_footnote(&mut self, id: Option<&SourceString>, content: Option<&[InlineNode]>);
  fn enter_text_span(&mut self, attrs: &AttrList, children: &[InlineNode]);
  fn exit_text_span(&mut self, attrs: &AttrList, children: &[InlineNode]);
  fn enter_xref(&mut self, target: &SourceString, reftext: Option<&[InlineNode]>, kind: XrefKind);
  fn exit_xref(&mut self, target: &SourceString, reftext: Option<&[InlineNode]>, kind: XrefKind);
  fn visit_missing_xref(
    &mut self,
    target: &SourceString,
    kind: XrefKind,
    doc_title: Option<&DocTitle>,
  );
  fn visit_inline_anchor(&mut self, id: &str);
  fn visit_biblio_anchor(&mut self, id: &str, reftext: Option<&str>);
  fn visit_symbol(&mut self, kind: SymbolKind);
  fn visit_linebreak(&mut self);
  fn enter_xref_text(&mut self, text: &[InlineNode], is_biblio: bool) {
    _ = (text, is_biblio);
  }
  fn exit_xref_text(&mut self, text: &[InlineNode], is_biblio: bool) {
    _ = (text, is_biblio);
  }

  // result
  fn into_result(self) -> Result<Self::Output, Self::Error>;
  fn result(&self) -> Result<&Self::Output, Self::Error>;
}
