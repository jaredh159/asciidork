use ast::prelude::*;

use super::admonition::AdmonitionKind;

pub trait Backend {
  type Output;
  type Error;

  // document
  fn enter_document(&mut self, document: &Document, header_attrs: &AttrEntries, flags: Flags);
  fn exit_document(&mut self, document: &Document, header_attrs: &AttrEntries);
  fn visit_document_attribute_decl(&mut self, name: &str, entry: &AttrEntry);

  // blocks contexts
  fn enter_paragraph_block(&mut self, block: &Block);
  fn exit_paragraph_block(&mut self, block: &Block);
  fn enter_image_block(&mut self, img_target: &str, img_attrs: &AttrList, block: &Block);
  fn exit_image_block(&mut self, block: &Block);
  fn enter_admonition_block(&mut self, kind: AdmonitionKind, block: &Block);
  fn exit_admonition_block(&mut self, kind: AdmonitionKind, block: &Block);

  // block content
  fn enter_simple_block_content(&mut self, children: &[InlineNode], block: &Block);
  fn exit_simple_block_content(&mut self, children: &[InlineNode], block: &Block);

  /// inlines
  fn visit_inline_text(&mut self, text: &str);
  fn visit_inline_lit_mono(&mut self, text: &str);
  fn visit_joining_newline(&mut self);
  fn visit_curly_quote(&mut self, kind: CurlyKind);
  fn visit_multichar_whitespace(&mut self, whitespace: &str);
  fn visit_button_macro(&mut self, text: &str);
  fn visit_menu_macro(&mut self, items: &[&str]);
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

  // result
  fn into_result(self) -> Result<Self::Output, Self::Error>;
  fn result(&self) -> Result<&Self::Output, Self::Error>;
}

// todo: naming, which crate?... (settings?, meta?, opts?)
#[derive(Debug, Default, Clone, Copy)]
pub struct Flags {
  pub embedded: bool,
}

impl Flags {
  pub fn embedded() -> Self {
    Self { embedded: true }
  }
}
