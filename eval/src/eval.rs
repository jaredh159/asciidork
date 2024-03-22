use crate::internal::*;

pub fn eval<B: Backend>(
  document: Document,
  opts: Opts,
  mut backend: B,
) -> Result<B::Output, B::Error> {
  visit(document, opts, &mut backend);
  backend.into_result()
}

pub fn visit<B: Backend>(document: Document, opts: Opts, backend: &mut B) {
  let empty_attrs = AttrEntries::new();
  let doc_attrs = document
    .header
    .as_ref()
    .map(|h| &h.attrs)
    .unwrap_or(&empty_attrs);
  backend.enter_document(&document, doc_attrs, opts);
  match &document.content {
    DocContent::Blocks(blocks) => {
      blocks.iter().for_each(|block| eval_block(block, backend));
    }
    DocContent::Sectioned { sections, preamble } => {
      if let Some(blocks) = preamble {
        blocks.iter().for_each(|block| eval_block(block, backend));
      }
      sections.iter().for_each(|sect| eval_section(sect, backend));
    }
  }
  backend.exit_document(&document, doc_attrs);
}

fn eval_section(section: &Section, backend: &mut impl Backend) {
  backend.enter_section(section);
  backend.enter_section_heading(section);
  section
    .heading
    .iter()
    .for_each(|node| eval_inline(node, backend));
  backend.exit_section_heading(section);
  section
    .blocks
    .iter()
    .for_each(|block| eval_block(block, backend));
  backend.exit_section(section);
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
    (Context::Sidebar, Content::Simple(children)) => {
      backend.enter_sidebar_block(block, &block.content);
      backend.enter_simple_block_content(children, block);
      children.iter().for_each(|node| eval_inline(node, backend));
      backend.exit_simple_block_content(children, block);
      backend.exit_sidebar_block(block, &block.content);
    }
    (Context::Listing, Content::Simple(children)) => {
      backend.enter_listing_block(block, &block.content);
      backend.enter_simple_block_content(children, block);
      children.iter().for_each(|node| eval_inline(node, backend));
      backend.exit_simple_block_content(children, block);
      backend.exit_listing_block(block, &block.content);
    }
    (Context::Sidebar, Content::Compound(blocks)) => {
      backend.enter_sidebar_block(block, &block.content);
      backend.enter_compound_block_content(blocks, block);
      blocks.iter().for_each(|block| eval_block(block, backend));
      backend.exit_compound_block_content(blocks, block);
      backend.exit_sidebar_block(block, &block.content);
    }
    (Context::BlockQuote, Content::Simple(children)) => {
      backend.enter_quote_block(block, &block.content);
      backend.enter_simple_block_content(children, block);
      children.iter().for_each(|node| eval_inline(node, backend));
      backend.exit_simple_block_content(children, block);
      backend.exit_quote_block(block, &block.content);
    }
    (Context::BlockQuote, Content::Compound(blocks)) => {
      backend.enter_quote_block(block, &block.content);
      backend.enter_compound_block_content(blocks, block);
      blocks.iter().for_each(|block| eval_block(block, backend));
      backend.exit_compound_block_content(blocks, block);
      backend.exit_quote_block(block, &block.content);
    }
    (Context::QuotedParagraph, Content::QuotedParagraph { quote, attr, cite }) => {
      backend.enter_quoted_paragraph(block, attr, cite.as_deref());
      quote.iter().for_each(|node| eval_inline(node, backend));
      backend.exit_quoted_paragraph(block, attr, cite.as_deref());
    }
    (Context::Open, Content::Compound(blocks)) => {
      backend.enter_open_block(block, &block.content);
      backend.enter_compound_block_content(blocks, block);
      blocks.iter().for_each(|block| eval_block(block, backend));
      backend.exit_compound_block_content(blocks, block);
      backend.exit_open_block(block, &block.content);
    }
    (Context::Example, Content::Compound(blocks)) => {
      backend.enter_example_block(block, &block.content);
      backend.enter_compound_block_content(blocks, block);
      blocks.iter().for_each(|block| eval_block(block, backend));
      backend.exit_compound_block_content(blocks, block);
      backend.exit_example_block(block, &block.content);
    }
    (Context::Listing, Content::Compound(blocks)) => {
      backend.enter_listing_block(block, &block.content);
      backend.enter_compound_block_content(blocks, block);
      blocks.iter().for_each(|block| eval_block(block, backend));
      backend.exit_compound_block_content(blocks, block);
      backend.exit_listing_block(block, &block.content);
    }
    (
      Context::AdmonitionTip
      | Context::AdmonitionNote
      | Context::AdmonitionCaution
      | Context::AdmonitionWarning
      | Context::AdmonitionImportant,
      Content::Simple(children),
    ) => {
      let kind = AdmonitionKind::try_from(block.context).unwrap();
      backend.enter_admonition_block(kind, block);
      children.iter().for_each(|node| eval_inline(node, backend));
      backend.exit_admonition_block(kind, block);
    }
    (Context::Image, Content::Empty(EmptyMetadata::Image { target, attrs })) => {
      backend.enter_image_block(target, attrs, block);
      backend.exit_image_block(block);
    }
    (Context::DocumentAttributeDecl, Content::DocumentAttribute(name, entry)) => {
      backend.visit_document_attribute_decl(name, entry);
    }
    (Context::OrderedList, Content::List { items, depth, .. }) => {
      backend.enter_ordered_list(block, items, *depth);
      items.iter().for_each(|item| {
        backend.enter_list_item_principal(item);
        item
          .principle
          .iter()
          .for_each(|node| eval_inline(node, backend));
        backend.exit_list_item_principal(item);
        backend.enter_list_item_blocks(&item.blocks, item);
        item
          .blocks
          .iter()
          .for_each(|block| eval_block(block, backend));
        backend.exit_list_item_blocks(&item.blocks, item);
      });
      backend.exit_ordered_list(block, items, *depth);
    }
    (Context::UnorderedList, Content::List { items, depth, .. }) => {
      backend.enter_unordered_list(block, items, *depth);
      items.iter().for_each(|item| {
        backend.enter_list_item_principal(item);
        item
          .principle
          .iter()
          .for_each(|node| eval_inline(node, backend));
        backend.exit_list_item_principal(item);
        backend.enter_list_item_blocks(&item.blocks, item);
        item
          .blocks
          .iter()
          .for_each(|block| eval_block(block, backend));
        backend.exit_list_item_blocks(&item.blocks, item);
      });
      backend.exit_unordered_list(block, items, *depth);
    }
    (Context::DescriptionList, Content::List { items, depth, .. }) => {
      backend.enter_description_list(block, items, *depth);
      items.iter().for_each(|item| {
        backend.enter_description_list_term(item);
        item
          .principle
          .iter()
          .for_each(|node| eval_inline(node, backend));
        backend.exit_description_list_term(item);
        backend.enter_description_list_description(&item.blocks, item);
        item
          .blocks
          .iter()
          .for_each(|block| eval_block(block, backend));
        backend.exit_description_list_description(&item.blocks, item);
      });
      backend.exit_description_list(block, items, *depth);
    }
    (Context::Section, Content::Section(section)) => {
      eval_section(section, backend);
    }
    (Context::Comment, _) => {}
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
    Subscript(children) => {
      backend.enter_inline_subscript(children);
      children.iter().for_each(|node| eval_inline(node, backend));
      backend.exit_inline_subscript(children);
    }
    Superscript(children) => {
      backend.enter_inline_superscript(children);
      children.iter().for_each(|node| eval_inline(node, backend));
      backend.exit_inline_superscript(children);
    }
    Quote(kind, children) => {
      backend.enter_inline_quote(*kind, children);
      children.iter().for_each(|node| eval_inline(node, backend));
      backend.exit_inline_quote(*kind, children);
    }
    LitMono(text) => backend.visit_inline_lit_mono(text),
    Curly(kind) => backend.visit_curly_quote(*kind),
    MultiCharWhitespace(ws) => backend.visit_multichar_whitespace(ws.as_str()),
    Macro(Footnote { id, text }) => {
      backend.enter_footnote(id.as_deref(), text);
      text.iter().for_each(|node| eval_inline(node, backend));
      backend.exit_footnote(id.as_deref(), text);
    }
    Macro(Button(text)) => backend.visit_button_macro(text),
    Macro(Menu(items)) => {
      backend.visit_menu_macro(&items.iter().map(|s| s.src.as_str()).collect::<Vec<&str>>())
    }
    AttributeReference(name) => backend.visit_attribute_reference(name),
    _ => {
      println!("\nUnhandled inline node type:");
      println!("  -> {:?}\n", &inline.content);
      todo!();
    }
  }
}
