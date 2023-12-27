use std::convert::Infallible;

use ast::short::block::*;
use ast::variants::inline::*;
use ast::{Block, DocContent, Document, InlineNode, SpecialCharKind};

pub trait Backend {
  type Output;
  type Error;

  // document
  fn enter_document(&mut self, document: &Document);
  fn exit_document(&mut self, document: &Document);

  // blocks
  fn enter_paragraph_block(&mut self, block: &Block);
  fn exit_paragraph_block(&mut self, block: &Block);
  fn enter_simple_block_content(&mut self, children: &[InlineNode], block: &Block);
  fn exit_simple_block_content(&mut self, children: &[InlineNode], block: &Block);
  fn visit_block_title(&mut self, title: &str);

  /// inlines
  fn visit_inline_text(&mut self, text: &str);
  fn visit_joining_newline(&mut self);
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

  // result
  fn into_result(self) -> Result<Self::Output, Self::Error>;
  fn result(&self) -> Result<&Self::Output, Self::Error>;
}

pub struct AsciidoctorHtml {
  html: String,
}

impl AsciidoctorHtml {
  pub fn new() -> Self {
    Self { html: String::new() }
  }

  pub fn into_string(self) -> String {
    self.html
  }
}

impl Default for AsciidoctorHtml {
  fn default() -> Self {
    Self::new()
  }
}

impl Backend for AsciidoctorHtml {
  type Output = String;
  type Error = Infallible;

  fn enter_document(&mut self, _document: &Document) {}
  fn exit_document(&mut self, _document: &Document) {}

  fn enter_paragraph_block(&mut self, _block: &Block) {
    self.html.push_str(r#"<div class="paragraph">"#);
  }

  fn exit_paragraph_block(&mut self, _block: &Block) {
    self.html.push_str("</div>");
  }

  fn enter_simple_block_content(&mut self, _children: &[InlineNode], _block: &Block) {
    self.html.push_str("<p>");
  }

  fn exit_simple_block_content(&mut self, _children: &[InlineNode], _block: &Block) {
    self.html.push_str("</p>");
  }

  fn visit_block_title(&mut self, title: &str) {
    self.html.push_str(r#"<div class="title">"#);
    self.html.push_str(title);
    self.html.push_str("</div>");
  }

  fn enter_inline_italic(&mut self, _children: &[InlineNode]) {
    self.html.push_str("<em>");
  }

  fn exit_inline_italic(&mut self, _children: &[InlineNode]) {
    self.html.push_str("</em>");
  }

  fn visit_inline_text(&mut self, text: &str) {
    self.html.push_str(text);
  }

  fn visit_joining_newline(&mut self) {
    self.html.push(' ');
  }

  fn enter_inline_mono(&mut self, _children: &[InlineNode]) {
    self.html.push_str("<code>");
  }

  fn exit_inline_mono(&mut self, _children: &[InlineNode]) {
    self.html.push_str("</code>");
  }

  fn enter_inline_bold(&mut self, _children: &[InlineNode]) {
    self.html.push_str("<strong>");
  }

  fn exit_inline_bold(&mut self, _children: &[InlineNode]) {
    self.html.push_str("</strong>");
  }

  fn enter_inline_passthrough(&mut self, _children: &[InlineNode]) {}
  fn exit_inline_passthrough(&mut self, _children: &[InlineNode]) {}

  fn visit_inline_specialchar(&mut self, char: &SpecialCharKind) {
    match char {
      SpecialCharKind::Ampersand => self.html.push_str("&amp;"),
      SpecialCharKind::LessThan => self.html.push_str("&lt;"),
      SpecialCharKind::GreaterThan => self.html.push_str("&gt;"),
    }
  }

  fn enter_inline_highlight(&mut self, _children: &[InlineNode]) {
    self.html.push_str("<mark>");
  }

  fn exit_inline_highlight(&mut self, _children: &[InlineNode]) {
    self.html.push_str("</mark>");
  }

  fn into_result(self) -> Result<Self::Output, Self::Error> {
    Ok(self.html)
  }

  fn result(&self) -> Result<&Self::Output, Self::Error> {
    Ok(&self.html)
  }
}

pub fn eval<B: Backend>(document: Document, mut backend: B) -> Result<B::Output, B::Error> {
  visit(document, &mut backend);
  backend.into_result()
}

pub fn visit<B: Backend>(document: Document, backend: &mut B) {
  backend.enter_document(&document);
  match &document.content {
    DocContent::Blocks(blocks) => {
      for block in blocks {
        eval_block(block, backend);
      }
    }
    DocContent::Sectioned { .. } => todo!(),
  }
  backend.exit_document(&document);
}

fn eval_block(block: &Block, backend: &mut impl Backend) {
  match (block.context, &block.content) {
    (Context::Paragraph, Content::Simple(children)) => {
      backend.enter_paragraph_block(block);
      if let Some(title) = &block.title {
        backend.visit_block_title(title);
      }
      backend.enter_simple_block_content(children, block);
      children.iter().for_each(|node| eval_inline(node, backend));
      backend.exit_simple_block_content(children, block);
      backend.exit_paragraph_block(block);
    }
    _ => todo!(),
  }
}

fn eval_inline(inline: &InlineNode, backend: &mut impl Backend) {
  match &inline.content {
    Bold(children) => {
      backend.enter_inline_bold(children);
      children.iter().for_each(|node| eval_inline(node, backend));
      backend.exit_inline_bold(children);
    }
    Mono(children) => {
      backend.enter_inline_mono(children);
      children.iter().for_each(|node| eval_inline(node, backend));
      backend.exit_inline_mono(children);
    }
    InlinePassthrough(children) => {
      backend.enter_inline_passthrough(children);
      children.iter().for_each(|node| eval_inline(node, backend));
      backend.exit_inline_passthrough(children);
    }
    SpecialChar(char) => backend.visit_inline_specialchar(char),
    Text(text) => backend.visit_inline_text(text.as_str()),
    JoiningNewline => backend.visit_joining_newline(),
    Italic(children) => {
      backend.enter_inline_italic(children);
      children.iter().for_each(|node| eval_inline(node, backend));
      backend.exit_inline_italic(children);
    }
    Highlight(children) => {
      backend.enter_inline_highlight(children);
      children.iter().for_each(|node| eval_inline(node, backend));
      backend.exit_inline_highlight(children);
    }
    _ => {
      dbg!(inline);
      todo!();
    }
  }
}

#[cfg(test)]
mod tests {
  #[test]
  fn it_works() {
    assert_eq!(2 + 2, 4);
  }
}
