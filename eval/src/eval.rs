use crate::internal::*;

pub fn eval<B: Backend>(document: &Document, mut backend: B) -> Result<B::Output, B::Error> {
  visit(document, &mut backend);
  backend.into_result()
}

pub fn visit<B: Backend>(doc: &Document, backend: &mut B) {
  backend.enter_document(doc);
  if let Some(title) = &doc.title {
    backend.enter_document_title(title);
    title
      .iter()
      .for_each(|node| eval_inline(node, doc, backend));
    backend.exit_document_title(title);
  }
  eval_toc_at(
    doc,
    &[TocPosition::Auto, TocPosition::Left, TocPosition::Right],
    backend,
  );
  eval_doc_content(doc, &doc.content, backend);
  backend.exit_document(doc);
}

fn eval_doc_content(doc: &Document, content: &DocContent, backend: &mut impl Backend) {
  match content {
    DocContent::Blocks(blocks) => {
      blocks.iter().for_each(|b| eval_block(b, doc, backend));
    }
    DocContent::Sectioned { sections, preamble } => {
      if let Some(blocks) = preamble {
        backend.enter_preamble(blocks);
        blocks.iter().for_each(|b| eval_block(b, doc, backend));
        backend.exit_preamble(blocks);
        eval_toc_at(doc, &[TocPosition::Preamble], backend);
      }
      sections.iter().for_each(|s| eval_section(s, doc, backend));
    }
  }
}

fn eval_section(section: &Section, doc: &Document, backend: &mut impl Backend) {
  backend.enter_section(section);
  backend.enter_section_heading(section);
  section
    .heading
    .iter()
    .for_each(|node| eval_inline(node, doc, backend));
  backend.exit_section_heading(section);
  section
    .blocks
    .iter()
    .for_each(|block| eval_block(block, doc, backend));
  backend.exit_section(section);
}

fn eval_block(block: &Block, doc: &Document, backend: &mut impl Backend) {
  if let Some(title) = &block.meta.title {
    backend.enter_block_title(title, block);
    title.iter().for_each(|n| eval_inline(n, doc, backend));
    backend.exit_block_title(title, block);
  }
  match (block.context, &block.content) {
    (Context::Paragraph, Content::Simple(children)) => {
      backend.enter_paragraph_block(block);
      backend.enter_simple_block_content(children, block);
      children.iter().for_each(|n| eval_inline(n, doc, backend));
      backend.exit_simple_block_content(children, block);
      backend.exit_paragraph_block(block);
    }
    (Context::Sidebar, Content::Simple(children)) => {
      backend.enter_sidebar_block(block, &block.content);
      backend.enter_simple_block_content(children, block);
      children.iter().for_each(|n| eval_inline(n, doc, backend));
      backend.exit_simple_block_content(children, block);
      backend.exit_sidebar_block(block, &block.content);
    }
    (Context::Listing, Content::Simple(children)) => {
      backend.enter_listing_block(block, &block.content);
      backend.enter_simple_block_content(children, block);
      children.iter().for_each(|n| eval_inline(n, doc, backend));
      backend.exit_simple_block_content(children, block);
      backend.exit_listing_block(block, &block.content);
    }
    (Context::Sidebar, Content::Compound(blocks)) => {
      backend.enter_sidebar_block(block, &block.content);
      backend.enter_compound_block_content(blocks, block);
      blocks.iter().for_each(|b| eval_block(b, doc, backend));
      backend.exit_compound_block_content(blocks, block);
      backend.exit_sidebar_block(block, &block.content);
    }
    (Context::BlockQuote, Content::Simple(children)) => {
      backend.enter_quote_block(block, &block.content);
      backend.enter_simple_block_content(children, block);
      children.iter().for_each(|n| eval_inline(n, doc, backend));
      backend.exit_simple_block_content(children, block);
      backend.exit_quote_block(block, &block.content);
    }
    (Context::BlockQuote, Content::Compound(blocks)) => {
      backend.enter_quote_block(block, &block.content);
      backend.enter_compound_block_content(blocks, block);
      blocks.iter().for_each(|b| eval_block(b, doc, backend));
      backend.exit_compound_block_content(blocks, block);
      backend.exit_quote_block(block, &block.content);
    }
    (Context::Verse, Content::Simple(children)) => {
      backend.enter_verse_block(block, &block.content);
      backend.enter_simple_block_content(children, block);
      children.iter().for_each(|n| eval_inline(n, doc, backend));
      backend.exit_simple_block_content(children, block);
      backend.exit_verse_block(block, &block.content);
    }
    (Context::QuotedParagraph, Content::QuotedParagraph { quote, attr, cite }) => {
      backend.enter_quoted_paragraph(block, attr, cite.as_deref());
      quote.iter().for_each(|n| eval_inline(n, doc, backend));
      backend.exit_quoted_paragraph(block, attr, cite.as_deref());
    }
    (Context::Open, Content::Compound(blocks)) => {
      backend.enter_open_block(block, &block.content);
      backend.enter_compound_block_content(blocks, block);
      blocks.iter().for_each(|b| eval_block(b, doc, backend));
      backend.exit_compound_block_content(blocks, block);
      backend.exit_open_block(block, &block.content);
    }
    (Context::Example, Content::Compound(blocks)) => {
      backend.enter_example_block(block, &block.content);
      backend.enter_compound_block_content(blocks, block);
      blocks.iter().for_each(|b| eval_block(b, doc, backend));
      backend.exit_compound_block_content(blocks, block);
      backend.exit_example_block(block, &block.content);
    }
    (Context::Example, Content::Simple(children)) => {
      backend.enter_example_block(block, &block.content);
      backend.enter_simple_block_content(children, block);
      children.iter().for_each(|n| eval_inline(n, doc, backend));
      backend.exit_simple_block_content(children, block);
      backend.exit_example_block(block, &block.content);
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
      children.iter().for_each(|n| eval_inline(n, doc, backend));
      backend.exit_admonition_block(kind, block);
    }
    (Context::Image, Content::Empty(EmptyMetadata::Image { target, attrs })) => {
      backend.enter_image_block(target, attrs, block);
      backend.exit_image_block(block);
    }
    (Context::DocumentAttributeDecl, Content::DocumentAttribute(name, entry)) => {
      backend.visit_document_attribute_decl(name, entry);
    }
    (Context::OrderedList, Content::List { items, depth, variant }) => {
      backend.enter_ordered_list(block, items, *depth);
      items.iter().for_each(|item| {
        backend.enter_list_item_principal(item, *variant);
        item
          .principle
          .iter()
          .for_each(|node| eval_inline(node, doc, backend));
        backend.exit_list_item_principal(item, *variant);
        backend.enter_list_item_blocks(&item.blocks, item, *variant);
        item.blocks.iter().for_each(|b| eval_block(b, doc, backend));
        backend.exit_list_item_blocks(&item.blocks, item, *variant);
      });
      backend.exit_ordered_list(block, items, *depth);
    }
    (Context::UnorderedList, Content::List { items, depth, variant }) => {
      backend.enter_unordered_list(block, items, *depth);
      items.iter().for_each(|item| {
        backend.enter_list_item_principal(item, *variant);
        item
          .principle
          .iter()
          .for_each(|node| eval_inline(node, doc, backend));
        backend.exit_list_item_principal(item, *variant);
        backend.enter_list_item_blocks(&item.blocks, item, *variant);
        item.blocks.iter().for_each(|b| eval_block(b, doc, backend));
        backend.exit_list_item_blocks(&item.blocks, item, *variant);
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
          .for_each(|node| eval_inline(node, doc, backend));
        backend.exit_description_list_term(item);
        backend.enter_description_list_description(&item.blocks, item);
        item.blocks.iter().for_each(|b| eval_block(b, doc, backend));
        backend.exit_description_list_description(&item.blocks, item);
      });
      backend.exit_description_list(block, items, *depth);
    }
    (Context::CalloutList, Content::List { items, depth, variant }) => {
      backend.enter_callout_list(block, items, *depth);
      items.iter().for_each(|item| {
        backend.enter_list_item_principal(item, *variant);
        item
          .principle
          .iter()
          .for_each(|node| eval_inline(node, doc, backend));
        backend.exit_list_item_principal(item, *variant);
        backend.enter_list_item_blocks(&item.blocks, item, *variant);
        item.blocks.iter().for_each(|b| eval_block(b, doc, backend));
        backend.exit_list_item_blocks(&item.blocks, item, *variant);
      });
      backend.exit_callout_list(block, items, *depth);
    }
    (Context::Section, Content::Section(section)) => {
      eval_section(section, doc, backend);
    }
    (Context::Literal, Content::Simple(children)) => {
      backend.enter_literal_block(block, &block.content);
      backend.enter_simple_block_content(children, block);
      children.iter().for_each(|n| eval_inline(n, doc, backend));
      backend.exit_simple_block_content(children, block);
      backend.exit_literal_block(block, &block.content);
    }
    (Context::Passthrough, Content::Simple(children)) => {
      backend.enter_passthrough_block(block, &block.content);
      backend.enter_simple_block_content(children, block);
      children.iter().for_each(|n| eval_inline(n, doc, backend));
      backend.exit_simple_block_content(children, block);
      backend.exit_passthrough_block(block, &block.content);
    }
    (Context::Table, Content::Table(table)) => {
      backend.enter_table(table, block);
      if let Some(header_row) = &table.header_row {
        backend.enter_table_section(TableSection::Header);
        eval_table_row(header_row, TableSection::Header, doc, backend);
        backend.exit_table_section(TableSection::Header);
      }
      backend.enter_table_section(TableSection::Body);
      table
        .rows
        .iter()
        .for_each(|row| eval_table_row(row, TableSection::Body, doc, backend));
      backend.exit_table_section(TableSection::Body);
      if let Some(footer_row) = &table.footer_row {
        backend.enter_table_section(TableSection::Footer);
        eval_table_row(footer_row, TableSection::Footer, doc, backend);
        backend.exit_table_section(TableSection::Footer);
      }
      backend.exit_table(table, block);
    }
    (
      Context::DiscreteHeading,
      Content::Empty(EmptyMetadata::DiscreteHeading { level, content, id }),
    ) => {
      backend.enter_discrete_heading(*level, id.as_deref(), block);
      content.iter().for_each(|n| eval_inline(n, doc, backend));
      backend.exit_discrete_heading(*level, id.as_deref(), block);
    }
    (Context::ThematicBreak, _) => {
      backend.visit_thematic_break(block);
    }
    (Context::PageBreak, _) => {
      backend.visit_page_break(block);
    }
    (Context::TableOfContents, _) => eval_toc_at(doc, &[TocPosition::Macro], backend),
    (Context::Comment, _) => {}
    _ => {
      dbg!(block.context, &block.content);
      todo!();
    }
  }
}

fn eval_inline(inline: &InlineNode, doc: &Document, backend: &mut impl Backend) {
  match &inline.content {
    Bold(children) => {
      backend.enter_inline_bold(children);
      children.iter().for_each(|n| eval_inline(n, doc, backend));
      backend.exit_inline_bold(children);
    }
    Mono(children) => {
      backend.enter_inline_mono(children);
      children.iter().for_each(|n| eval_inline(n, doc, backend));
      backend.exit_inline_mono(children);
    }
    InlinePassthrough(children) => {
      backend.enter_inline_passthrough(children);
      children.iter().for_each(|n| eval_inline(n, doc, backend));
      backend.exit_inline_passthrough(children);
    }
    SpecialChar(char) => backend.visit_inline_specialchar(char),
    Text(text) => backend.visit_inline_text(text.as_str()),
    Newline => backend.visit_joining_newline(),
    Italic(children) => {
      backend.enter_inline_italic(children);
      children.iter().for_each(|n| eval_inline(n, doc, backend));
      backend.exit_inline_italic(children);
    }
    Highlight(children) => {
      backend.enter_inline_highlight(children);
      children.iter().for_each(|n| eval_inline(n, doc, backend));
      backend.exit_inline_highlight(children);
    }
    Subscript(children) => {
      backend.enter_inline_subscript(children);
      children.iter().for_each(|n| eval_inline(n, doc, backend));
      backend.exit_inline_subscript(children);
    }
    Superscript(children) => {
      backend.enter_inline_superscript(children);
      children.iter().for_each(|n| eval_inline(n, doc, backend));
      backend.exit_inline_superscript(children);
    }
    Quote(kind, children) => {
      backend.enter_inline_quote(*kind, children);
      children.iter().for_each(|n| eval_inline(n, doc, backend));
      backend.exit_inline_quote(*kind, children);
    }
    LitMono(text) => backend.visit_inline_lit_mono(text),
    CurlyQuote(kind) => backend.visit_curly_quote(*kind),
    MultiCharWhitespace(ws) => backend.visit_multichar_whitespace(ws.as_str()),
    Macro(Footnote { id, text }) => {
      backend.enter_footnote(id.as_deref(), text);
      text.iter().for_each(|node| eval_inline(node, doc, backend));
      backend.exit_footnote(id.as_deref(), text);
    }
    Macro(Button(text)) => backend.visit_button_macro(text),
    Macro(Link { target, attrs, scheme }) => {
      backend.enter_link_macro(target, attrs.as_ref(), *scheme);
      if let Some(Some(nodes)) = attrs.as_ref().and_then(|a| a.positional.first()) {
        nodes.iter().for_each(|n| eval_inline(n, doc, backend));
      }
      backend.exit_link_macro(target, attrs.as_ref(), *scheme);
    }
    Macro(Pass { content, .. }) => {
      content.iter().for_each(|n| eval_inline(n, doc, backend));
    }
    Macro(Keyboard { keys, .. }) => {
      backend.visit_keyboard_macro(&keys.iter().map(|s| s.as_str()).collect::<Vec<&str>>())
    }
    Macro(Menu(items)) => {
      backend.visit_menu_macro(&items.iter().map(|s| s.src.as_str()).collect::<Vec<&str>>())
    }
    Macro(Xref { id, linktext }) => {
      backend.enter_xref(id, linktext.as_ref().map(|t| t.as_slice()));
      if let Some(anchor) = doc.anchors.get(&id.src) {
        let text = anchor
          .reftext
          .as_ref()
          .unwrap_or(linktext.as_ref().unwrap_or(&anchor.title));
        text.iter().for_each(|node| eval_inline(node, doc, backend));
      } else {
        backend.visit_missing_xref(id);
      }
      backend.exit_xref(id, linktext.as_ref().map(|t| t.as_slice()));
    }
    LineBreak => backend.visit_linebreak(),
    AttributeReference(name) => backend.visit_attribute_reference(name),
    CalloutNum(callout) => backend.visit_callout(*callout),
    CalloutTuck(comment) => backend.visit_callout_tuck(comment),
    TextSpan(attrs, nodes) => {
      backend.enter_text_span(attrs, nodes);
      nodes.iter().for_each(|n| eval_inline(n, doc, backend));
      backend.exit_text_span(attrs, nodes);
    }
    LineComment(_) | Discarded => {}
    _ => {
      println!("\nUnhandled inline node type:");
      println!("  -> {:?}\n", &inline.content);
      todo!();
    }
  }
}

fn eval_table_row(row: &Row, section: TableSection, doc: &Document, backend: &mut impl Backend) {
  backend.enter_table_row(row, section);
  row.cells.iter().for_each(|cell| {
    backend.enter_table_cell(cell, section);
    match &cell.content {
      CellContent::Default(paragraphs)
      | CellContent::Emphasis(paragraphs)
      | CellContent::Header(paragraphs)
      | CellContent::Monospace(paragraphs)
      | CellContent::Strong(paragraphs) => {
        paragraphs.iter().for_each(|paragraph| {
          backend.enter_cell_paragraph(cell, section);
          paragraph.iter().for_each(|n| eval_inline(n, doc, backend));
          backend.exit_cell_paragraph(cell, section);
        });
      }
      CellContent::Literal(nodes) => {
        nodes.iter().for_each(|n| eval_inline(n, doc, backend));
      }
      CellContent::AsciiDoc(document) => {
        let mut cell_backend = backend.asciidoc_table_cell_backend();
        visit(document, &mut cell_backend);
        backend.visit_asciidoc_table_cell_result(cell_backend.into_result());
      }
    }
    backend.exit_table_cell(cell, section);
  });
  backend.exit_table_row(row, section);
}

fn eval_toc_at(document: &Document, positions: &[TocPosition], backend: &mut impl Backend) {
  let Some(toc) = &document.toc else {
    return;
  };
  if !positions.contains(&toc.position) || toc.nodes.is_empty() {
    return;
  }
  backend.enter_toc(toc);
  eval_toc_level(&toc.nodes, document, backend);
  backend.exit_toc(toc);
}

fn eval_toc_level(nodes: &[TocNode], doc: &Document, backend: &mut impl Backend) {
  if let Some(first) = nodes.first() {
    backend.enter_toc_level(first.level, nodes);
    nodes.iter().for_each(|node| {
      backend.enter_toc_node(node);
      backend.enter_toc_content(&node.title);
      node.title.iter().for_each(|n| eval_inline(n, doc, backend));
      backend.exit_toc_content(&node.title);
      eval_toc_level(&node.children, doc, backend);
      backend.exit_toc_node(node);
    });
    backend.exit_toc_level(first.level, nodes);
  }
}
