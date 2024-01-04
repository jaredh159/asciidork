use crate::internal::*;

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
    _ => {
      dbg!(block.context);
      todo!();
    }
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
      println!("Unhandled inline node type ^^^");
      todo!();
    }
  }
}
