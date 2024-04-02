use super::admonition::AdmonitionKind;
use ast::prelude::*;
use opts::Opts;

pub trait Backend {
  type Output;
  type Error;

  // document
  fn enter_document(&mut self, document: &Document, header_attrs: &AttrEntries, opts: Opts);
  fn exit_document(&mut self, document: &Document, header_attrs: &AttrEntries);
  fn visit_document_attribute_decl(&mut self, name: &str, entry: &AttrEntry);

  // sections
  fn enter_section(&mut self, section: &Section);
  fn exit_section(&mut self, section: &Section);
  fn enter_section_heading(&mut self, section: &Section);
  fn exit_section_heading(&mut self, section: &Section);

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
  fn enter_listing_block(&mut self, block: &Block, content: &BlockContent);
  fn exit_listing_block(&mut self, block: &Block, content: &BlockContent);
  fn enter_literal_block(&mut self, block: &Block, content: &BlockContent);
  fn exit_literal_block(&mut self, block: &Block, content: &BlockContent);
  fn enter_passthrough_block(&mut self, block: &Block, content: &BlockContent);
  fn exit_passthrough_block(&mut self, block: &Block, content: &BlockContent);
  fn enter_image_block(&mut self, img_target: &str, img_attrs: &AttrList, block: &Block);
  fn exit_image_block(&mut self, block: &Block);
  fn enter_admonition_block(&mut self, kind: AdmonitionKind, block: &Block);
  fn exit_admonition_block(&mut self, kind: AdmonitionKind, block: &Block);
  fn enter_quoted_paragraph(&mut self, block: &Block, attr: &str, cite: Option<&str>);
  fn exit_quoted_paragraph(&mut self, block: &Block, attr: &str, cite: Option<&str>);

  // lists
  fn enter_unordered_list(&mut self, block: &Block, items: &[ListItem], depth: u8);
  fn exit_unordered_list(&mut self, block: &Block, items: &[ListItem], depth: u8);
  fn enter_ordered_list(&mut self, block: &Block, items: &[ListItem], depth: u8);
  fn exit_ordered_list(&mut self, block: &Block, items: &[ListItem], depth: u8);
  fn enter_description_list(&mut self, block: &Block, items: &[ListItem], depth: u8);
  fn exit_description_list(&mut self, block: &Block, items: &[ListItem], depth: u8);
  fn enter_description_list_term(&mut self, item: &ListItem);
  fn exit_description_list_term(&mut self, item: &ListItem);
  fn enter_description_list_description(&mut self, blocks: &[Block], item: &ListItem);
  fn exit_description_list_description(&mut self, blocks: &[Block], item: &ListItem);
  fn enter_list_item_principal(&mut self, item: &ListItem);
  fn exit_list_item_principal(&mut self, item: &ListItem);
  fn enter_list_item_blocks(&mut self, blocks: &[Block], item: &ListItem);
  fn exit_list_item_blocks(&mut self, blocks: &[Block], item: &ListItem);

  // block content
  fn enter_simple_block_content(&mut self, children: &[InlineNode], block: &Block);
  fn exit_simple_block_content(&mut self, children: &[InlineNode], block: &Block);
  fn enter_compound_block_content(&mut self, children: &[Block], block: &Block);
  fn exit_compound_block_content(&mut self, children: &[Block], block: &Block);

  /// inlines
  fn visit_inline_text(&mut self, text: &str);
  fn visit_inline_lit_mono(&mut self, text: &str);
  fn visit_joining_newline(&mut self);
  fn visit_curly_quote(&mut self, kind: CurlyKind);
  fn visit_multichar_whitespace(&mut self, whitespace: &str);
  fn visit_button_macro(&mut self, text: &str);
  fn visit_menu_macro(&mut self, items: &[&str]);
  fn visit_attribute_reference(&mut self, name: &str);
  fn visit_callout_number(&mut self, number: u8);
  fn enter_inline_italic(&mut self, children: &[InlineNode]);
  fn exit_inline_italic(&mut self, children: &[InlineNode]);
  fn enter_inline_mono(&mut self, children: &[InlineNode]);
  fn exit_inline_mono(&mut self, children: &[InlineNode]);
  fn enter_inline_bold(&mut self, children: &[InlineNode]);
  fn exit_inline_bold(&mut self, children: &[InlineNode]);
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
  fn enter_footnote(&mut self, id: Option<&str>, content: &[InlineNode]);
  fn exit_footnote(&mut self, id: Option<&str>, content: &[InlineNode]);
  fn visit_linebreak(&mut self);

  // result
  fn into_result(self) -> Result<Self::Output, Self::Error>;
  fn result(&self) -> Result<&Self::Output, Self::Error>;
}
