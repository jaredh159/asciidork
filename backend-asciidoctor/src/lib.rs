use std::borrow::Cow;
use std::convert::Infallible;

use lazy_static::lazy_static;
use regex::Regex;

use ast::prelude::*;
use ast::short::block::*;
use ast::variants::inline::*;

pub trait Backend {
  type Output;
  type Error;

  // document
  fn enter_document(&mut self, document: &Document, header_attrs: &AttrEntries);
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
  doc_attrs: AttrEntries,
  fig_caption_num: usize,
}

impl AsciidoctorHtml {
  pub fn new() -> Self {
    Self {
      html: String::new(),
      doc_attrs: AttrEntries::new(),
      fig_caption_num: 0,
    }
  }

  pub fn into_string(self) -> String {
    self.html
  }

  fn push_str(&mut self, s: &str) {
    self.html.push_str(s);
  }

  fn push_ch(&mut self, c: char) {
    self.html.push(c);
  }

  fn push<const N: usize>(&mut self, strs: [&str; N]) {
    for s in strs {
      self.push_str(s);
    }
  }

  fn visit_block_title(&mut self, title: Option<&str>, prefix: Option<Cow<str>>) {
    if let Some(title) = title {
      self.push_str(r#"<div class="title">"#);
      if let Some(prefix) = prefix {
        self.push_str(prefix.as_ref());
      }
      self.push_str(title);
      self.push_str("</div>");
    }
  }

  fn open_element(&mut self, element: &str, classes: &[&str], attrs: &Option<AttrList>) {
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
}

impl Default for AsciidoctorHtml {
  fn default() -> Self {
    Self::new()
  }
}

impl Backend for AsciidoctorHtml {
  type Output = String;
  type Error = Infallible;

  fn enter_document(&mut self, _document: &Document, header_attrs: &AttrEntries) {
    self.doc_attrs = header_attrs.clone();
  }

  fn exit_document(&mut self, _document: &Document, _header_attrs: &AttrEntries) {}

  fn enter_paragraph_block(&mut self, block: &Block) {
    self.push_str(r#"<div class="paragraph">"#);
    self.visit_block_title(block.title.as_deref(), None);
  }

  fn exit_paragraph_block(&mut self, _block: &Block) {
    self.push_str("</div>");
  }

  fn enter_simple_block_content(&mut self, _children: &[InlineNode], _block: &Block) {
    self.push_str("<p>");
  }

  fn exit_simple_block_content(&mut self, _children: &[InlineNode], _block: &Block) {
    self.push_str("</p>");
  }

  fn enter_inline_italic(&mut self, _children: &[InlineNode]) {
    self.push_str("<em>");
  }

  fn exit_inline_italic(&mut self, _children: &[InlineNode]) {
    self.push_str("</em>");
  }

  fn visit_inline_text(&mut self, text: &str) {
    self.push_str(text);
  }

  fn visit_joining_newline(&mut self) {
    self.push_ch(' ');
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

  fn into_result(self) -> Result<Self::Output, Self::Error> {
    Ok(self.html)
  }

  fn result(&self) -> Result<&Self::Output, Self::Error> {
    Ok(&self.html)
  }

  fn enter_admonition_block(&mut self, kind: AdmonitionKind, block: &Block) {
    let classes = &["admonitionblock", kind.lowercase_str()];
    self.open_element("div", classes, &block.attrs);
    self.push_str(r#"<table><tr><td class="icon"><div class="title">"#);
    self.push_str(kind.str());
    self.push_str(r#"</div></td><td class="content">"#);
    self.visit_block_title(block.title.as_deref(), None);
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
    self.open_element("div", &["imageblock"], &block.attrs);
    self.push_str(r#"<div class="content">"#);
    let mut has_link = false;
    if let Some(href) = &block.attrs.as_ref().and_then(|attrs| attrs.named("link")) {
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
    let prefix = if self.doc_attrs.is_unset("figure-caption") {
      None
    } else {
      self.fig_caption_num += 1;
      Some(Cow::Owned(format!("Figure {}. ", self.fig_caption_num)))
    };
    self.visit_block_title(block.title.as_deref(), prefix);
    self.push_str(r#"</div>"#);
  }

  fn visit_document_attribute_decl(&mut self, name: &str, entry: &AttrEntry) {
    self.doc_attrs.insert(name.to_string(), entry.clone());
  }
}

pub fn eval<B: Backend>(document: Document, mut backend: B) -> Result<B::Output, B::Error> {
  visit(document, &mut backend);
  backend.into_result()
}

pub fn visit<B: Backend>(document: Document, backend: &mut B) {
  let empty_attrs = AttrEntries::new();
  let doc_attrs = document
    .header
    .as_ref()
    .map(|h| &h.attrs)
    .unwrap_or(&empty_attrs);
  backend.enter_document(&document, doc_attrs);
  match &document.content {
    DocContent::Blocks(blocks) => {
      for block in blocks {
        eval_block(block, backend);
      }
    }
    DocContent::Sectioned { .. } => todo!(),
  }
  backend.exit_document(&document, doc_attrs);
}

fn eval_block(block: &Block, backend: &mut impl Backend) {
  match (block.context, &block.content) {
    (Context::Paragraph, Content::Simple(children)) => {
      backend.enter_paragraph_block(block);
      backend.enter_simple_block_content(children, block);
      children.iter().for_each(|node| eval_inline(node, backend));
      backend.exit_simple_block_content(children, block);
      backend.exit_paragraph_block(block);
    }
    (
      Context::AdmonitionTip
      | Context::AdmonitionNote
      | Context::AdmonitionCaution
      | Context::AdmonitionWarning
      | Context::AdmonitionImportant,
      Content::Simple(children),
    ) => {
      let admonition = AdmonitionKind::try_from(block.context).unwrap();
      backend.enter_admonition_block(admonition, block);
      children.iter().for_each(|node| eval_inline(node, backend));
      backend.exit_admonition_block(admonition, block);
    }
    (Context::Image, Content::Empty(EmptyMetadata::Image { target, attrs })) => {
      backend.enter_image_block(target, attrs, block);
      backend.exit_image_block(block);
    }
    (Context::DocumentAttributeDecl, Content::DocumentAttribute(name, entry)) => {
      backend.visit_document_attribute_decl(name, entry);
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

lazy_static! {
  pub static ref REMOVE_FILE_EXT: Regex = Regex::new(r"^(.*)\.[^.]+$").unwrap();
}

#[derive(Copy, Debug, PartialEq, Eq, Clone)]
pub enum AdmonitionKind {
  Tip,
  Caution,
  Important,
  Note,
  Warning,
}

impl AdmonitionKind {
  pub fn lowercase_str(&self) -> &'static str {
    match self {
      AdmonitionKind::Tip => "tip",
      AdmonitionKind::Caution => "caution",
      AdmonitionKind::Important => "important",
      AdmonitionKind::Note => "note",
      AdmonitionKind::Warning => "warning",
    }
  }

  pub fn str(&self) -> &'static str {
    match self {
      AdmonitionKind::Tip => "Tip",
      AdmonitionKind::Caution => "Caution",
      AdmonitionKind::Important => "Important",
      AdmonitionKind::Note => "Note",
      AdmonitionKind::Warning => "Warning",
    }
  }
}

impl TryFrom<Context> for AdmonitionKind {
  type Error = &'static str;
  fn try_from(context: Context) -> Result<Self, Self::Error> {
    match context {
      Context::AdmonitionTip => Ok(AdmonitionKind::Tip),
      Context::AdmonitionCaution => Ok(AdmonitionKind::Caution),
      Context::AdmonitionImportant => Ok(AdmonitionKind::Important),
      Context::AdmonitionNote => Ok(AdmonitionKind::Note),
      Context::AdmonitionWarning => Ok(AdmonitionKind::Warning),
      _ => Err("not an admonition"),
    }
  }
}
