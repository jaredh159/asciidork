use std::cell::RefCell;

use crate::internal::*;
use asciidork_backend::utils;

pub fn eval<B: Backend>(document: &Document, mut backend: B) -> Result<B::Output, B::Error> {
  visit(document, &mut backend);
  backend.into_result()
}

struct Ctx<'a, 'b> {
  doc: &'a Document<'b>,
  resolving_xref: RefCell<bool>,
}

pub fn visit<B: Backend>(doc: &Document, backend: &mut B) {
  let ctx = Ctx {
    doc,
    resolving_xref: RefCell::new(false),
  };
  backend.enter_document(ctx.doc);
  backend.enter_header();
  if let Some(doc_title) = doc.title() {
    backend.enter_document_title(&doc_title.main);
    doc_title
      .main
      .iter()
      .for_each(|node| eval_inline(node, &ctx, backend));
    backend.exit_document_title(&doc_title.main);
  }
  backend.exit_header();
  eval_toc_at(
    &[TocPosition::Auto, TocPosition::Left, TocPosition::Right],
    None,
    &ctx,
    backend,
  );
  eval_doc_content(&ctx, backend);
  backend.enter_footer();
  backend.exit_footer();
  backend.exit_document(ctx.doc);
}

fn eval_doc_content(ctx: &Ctx, backend: &mut impl Backend) {
  backend.enter_content();
  match &ctx.doc.content {
    DocContent::Blocks(blocks) => {
      blocks.iter().for_each(|b| eval_block(b, ctx, backend));
    }
    DocContent::Sections(content) => {
      if let Some(blocks) = &content.preamble {
        backend.enter_preamble(ctx.doc.title().is_some(), blocks);
        blocks.iter().for_each(|b| eval_block(b, ctx, backend));
        backend.exit_preamble(ctx.doc.title().is_some(), blocks);
        eval_toc_at(&[TocPosition::Preamble], None, ctx, backend);
      }
      content
        .sections
        .iter()
        .for_each(|s| eval_section(s, ctx, backend));
    }
    DocContent::Parts(book) => {
      eval_book(book, ctx, backend);
    }
  }
  backend.exit_content();
}

fn eval_book(book: &MultiPartBook, ctx: &Ctx, backend: &mut impl Backend) {
  if let Some(blocks) = &book.preamble {
    backend.enter_preamble(ctx.doc.title().is_some(), blocks);
    blocks.iter().for_each(|b| eval_block(b, ctx, backend));
    book
      .opening_special_sects
      .iter()
      .for_each(|sect| eval_section(sect, ctx, backend));
    backend.exit_preamble(ctx.doc.title().is_some(), blocks);
    eval_toc_at(&[TocPosition::Preamble], None, ctx, backend);
  } else {
    book
      .opening_special_sects
      .iter()
      .for_each(|sect| eval_section(sect, ctx, backend));
  }
  book
    .parts
    .iter()
    .for_each(|p| eval_book_part(p, ctx, backend));
  book
    .closing_special_sects
    .iter()
    .for_each(|sect| eval_section(sect, ctx, backend));
}

fn eval_book_part(part: &Part, ctx: &Ctx, backend: &mut impl Backend) {
  backend.enter_book_part(part);
  backend.enter_book_part_title(&part.title);
  part
    .title
    .text
    .iter()
    .for_each(|node| eval_inline(node, ctx, backend));
  backend.exit_book_part_title(&part.title);
  if let Some(blocks) = &part.intro {
    backend.enter_book_part_intro(part);
    if let Some(title) = &part.title.meta.title {
      title.iter().for_each(|n| eval_inline(n, ctx, backend));
    }
    backend.enter_book_part_intro_content(part);
    blocks.iter().for_each(|b| eval_block(b, ctx, backend));
    backend.exit_book_part_intro_content(part);
    backend.exit_book_part_intro(part);
  }
  part
    .sections
    .iter()
    .for_each(|section| eval_section(section, ctx, backend));
  backend.exit_book_part(part);
}

fn eval_section(section: &Section, ctx: &Ctx, backend: &mut impl Backend) {
  backend.enter_section(section);
  backend.enter_section_heading(section);
  section
    .heading
    .iter()
    .for_each(|node| eval_inline(node, ctx, backend));
  backend.exit_section_heading(section);
  section
    .blocks
    .iter()
    .for_each(|block| eval_block(block, ctx, backend));
  backend.exit_section(section);
}

fn eval_block(block: &Block, ctx: &Ctx, backend: &mut impl Backend) {
  if let Some(title) = &block.meta.title {
    backend.enter_meta_title(title);
    title.iter().for_each(|n| eval_inline(n, ctx, backend));
    backend.exit_meta_title(title);
  }
  match (block.context, &block.content) {
    (Context::Paragraph, Content::Simple(children)) => {
      backend.enter_paragraph_block(block);
      backend.enter_simple_block_content(children, block);
      children.iter().for_each(|n| eval_inline(n, ctx, backend));
      backend.exit_simple_block_content(children, block);
      backend.exit_paragraph_block(block);
    }
    (Context::Sidebar, Content::Simple(children)) => {
      backend.enter_sidebar_block(block, &block.content);
      backend.enter_simple_block_content(children, block);
      children.iter().for_each(|n| eval_inline(n, ctx, backend));
      backend.exit_simple_block_content(children, block);
      backend.exit_sidebar_block(block, &block.content);
    }
    (Context::Listing, Content::Simple(children)) => {
      backend.enter_listing_block(block, &block.content);
      backend.enter_simple_block_content(children, block);
      children.iter().for_each(|n| eval_inline(n, ctx, backend));
      backend.exit_simple_block_content(children, block);
      backend.exit_listing_block(block, &block.content);
    }
    (Context::Sidebar, Content::Compound(blocks)) => {
      backend.enter_sidebar_block(block, &block.content);
      backend.enter_compound_block_content(blocks, block);
      blocks.iter().for_each(|b| eval_block(b, ctx, backend));
      backend.exit_compound_block_content(blocks, block);
      backend.exit_sidebar_block(block, &block.content);
    }
    (Context::BlockQuote, Content::Simple(children)) => {
      backend.enter_quote_block(block, &block.content);
      backend.enter_simple_block_content(children, block);
      children.iter().for_each(|n| eval_inline(n, ctx, backend));
      backend.exit_simple_block_content(children, block);
      backend.exit_quote_block(block, &block.content);
    }
    (Context::BlockQuote, Content::Compound(blocks)) => {
      backend.enter_quote_block(block, &block.content);
      backend.enter_compound_block_content(blocks, block);
      blocks.iter().for_each(|b| eval_block(b, ctx, backend));
      backend.exit_compound_block_content(blocks, block);
      backend.exit_quote_block(block, &block.content);
    }
    (Context::Verse, Content::Simple(children)) => {
      backend.enter_verse_block(block, &block.content);
      backend.enter_simple_block_content(children, block);
      children.iter().for_each(|n| eval_inline(n, ctx, backend));
      backend.exit_simple_block_content(children, block);
      backend.exit_verse_block(block, &block.content);
    }
    (Context::QuotedParagraph, Content::QuotedParagraph { quote, attr, cite }) => {
      backend.enter_quoted_paragraph(block, attr, cite.as_deref());
      quote.iter().for_each(|n| eval_inline(n, ctx, backend));
      backend.exit_quoted_paragraph(block, attr, cite.as_deref());
    }
    (Context::Open, Content::Compound(blocks)) => {
      backend.enter_open_block(block, &block.content);
      backend.enter_compound_block_content(blocks, block);
      blocks.iter().for_each(|b| eval_block(b, ctx, backend));
      backend.exit_compound_block_content(blocks, block);
      backend.exit_open_block(block, &block.content);
    }
    (Context::Example, Content::Compound(blocks)) => {
      backend.enter_example_block(block, &block.content);
      backend.enter_compound_block_content(blocks, block);
      blocks.iter().for_each(|b| eval_block(b, ctx, backend));
      backend.exit_compound_block_content(blocks, block);
      backend.exit_example_block(block, &block.content);
    }
    (Context::Example, Content::Simple(children)) => {
      backend.enter_example_block(block, &block.content);
      backend.enter_simple_block_content(children, block);
      children.iter().for_each(|n| eval_inline(n, ctx, backend));
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
      children.iter().for_each(|n| eval_inline(n, ctx, backend));
      backend.exit_admonition_block(kind, block);
    }
    (
      Context::AdmonitionTip
      | Context::AdmonitionNote
      | Context::AdmonitionCaution
      | Context::AdmonitionWarning
      | Context::AdmonitionImportant,
      Content::Compound(blocks),
    ) => {
      let kind = AdmonitionKind::try_from(block.context).unwrap();
      backend.enter_admonition_block(kind, block);
      backend.enter_compound_block_content(blocks, block);
      blocks.iter().for_each(|b| eval_block(b, ctx, backend));
      backend.exit_compound_block_content(blocks, block);
      backend.exit_admonition_block(kind, block);
    }
    (Context::Image, Content::Empty(EmptyMetadata::Image { target, attrs })) => {
      backend.enter_image_block(target, attrs, block);
      backend.exit_image_block(target, attrs, block);
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
          .for_each(|node| eval_inline(node, ctx, backend));
        backend.exit_list_item_principal(item, *variant);
        backend.enter_list_item_blocks(&item.blocks, item, *variant);
        item.blocks.iter().for_each(|b| eval_block(b, ctx, backend));
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
          .for_each(|node| eval_inline(node, ctx, backend));
        backend.exit_list_item_principal(item, *variant);
        backend.enter_list_item_blocks(&item.blocks, item, *variant);
        item.blocks.iter().for_each(|b| eval_block(b, ctx, backend));
        backend.exit_list_item_blocks(&item.blocks, item, *variant);
      });
      backend.exit_unordered_list(block, items, *depth);
    }
    (Context::DescriptionList, Content::List { items, depth, .. }) => {
      backend.enter_description_list(block, items, *depth);
      items.iter().for_each(|item| {
        let ListItemTypeMeta::DescList { extra_terms, description } = &item.type_meta else {
          unreachable!("eval description list extract meta");
        };
        backend.enter_description_list_term(&item.principle, item);
        item
          .principle
          .iter()
          .for_each(|node| eval_inline(node, ctx, backend));
        backend.exit_description_list_term(&item.principle, item);
        extra_terms.iter().for_each(|(term, _)| {
          backend.enter_description_list_term(term, item);
          term.iter().for_each(|node| eval_inline(node, ctx, backend));
          backend.exit_description_list_term(term, item);
        });
        if description.is_some() || !item.blocks.is_empty() {
          backend.enter_description_list_description(item);
          if let Some(description) = description {
            backend.enter_description_list_description_text(description, item);
            eval_block(description, ctx, backend);
            backend.exit_description_list_description_text(description, item);
          }
          item.blocks.iter().for_each(|b| {
            backend.enter_description_list_description_block(b, item);
            eval_block(b, ctx, backend);
            backend.exit_description_list_description_block(b, item);
          });
          backend.exit_description_list_description(item);
        }
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
          .for_each(|node| eval_inline(node, ctx, backend));
        backend.exit_list_item_principal(item, *variant);
        backend.enter_list_item_blocks(&item.blocks, item, *variant);
        item.blocks.iter().for_each(|b| eval_block(b, ctx, backend));
        backend.exit_list_item_blocks(&item.blocks, item, *variant);
      });
      backend.exit_callout_list(block, items, *depth);
    }
    (Context::Section, Content::Section(section)) => {
      eval_section(section, ctx, backend);
    }
    (Context::Literal, Content::Simple(children)) => {
      backend.enter_literal_block(block, &block.content);
      backend.enter_simple_block_content(children, block);
      children.iter().for_each(|n| eval_inline(n, ctx, backend));
      backend.exit_simple_block_content(children, block);
      backend.exit_literal_block(block, &block.content);
    }
    (Context::Passthrough, Content::Simple(children)) => {
      backend.enter_passthrough_block(block, &block.content);
      backend.enter_simple_block_content(children, block);
      children.iter().for_each(|n| eval_inline(n, ctx, backend));
      backend.exit_simple_block_content(children, block);
      backend.exit_passthrough_block(block, &block.content);
    }
    (Context::Table, Content::Table(table)) => {
      backend.enter_table(table, block);
      if let Some(header_row) = &table.header_row {
        backend.enter_table_section(TableSection::Header);
        eval_table_row(header_row, TableSection::Header, ctx, backend);
        backend.exit_table_section(TableSection::Header);
      }
      if !table.rows.is_empty() {
        backend.enter_table_section(TableSection::Body);
        table
          .rows
          .iter()
          .for_each(|row| eval_table_row(row, TableSection::Body, ctx, backend));
        backend.exit_table_section(TableSection::Body);
      }
      if let Some(footer_row) = &table.footer_row {
        backend.enter_table_section(TableSection::Footer);
        eval_table_row(footer_row, TableSection::Footer, ctx, backend);
        backend.exit_table_section(TableSection::Footer);
      }
      backend.exit_table(table, block);
    }
    (
      Context::DiscreteHeading,
      Content::Empty(EmptyMetadata::DiscreteHeading { level, content, id }),
    ) => {
      backend.enter_discrete_heading(*level, id.as_deref(), block);
      content.iter().for_each(|n| eval_inline(n, ctx, backend));
      backend.exit_discrete_heading(*level, id.as_deref(), block);
    }
    (Context::ThematicBreak, _) => {
      backend.visit_thematic_break(block);
    }
    (Context::PageBreak, _) => {
      backend.visit_page_break(block);
    }
    (Context::TableOfContents, _) => eval_toc_at(&[TocPosition::Macro], Some(block), ctx, backend),
    (Context::Comment, _) => {}
    _ => {
      dbg!(block.context, &block.content);
      todo!();
    }
  }
}

fn eval_inline(inline: &InlineNode, ctx: &Ctx, backend: &mut impl Backend) {
  match &inline.content {
    Bold(children) => {
      backend.enter_inline_bold(children);
      children.iter().for_each(|n| eval_inline(n, ctx, backend));
      backend.exit_inline_bold(children);
    }
    Mono(children) => {
      backend.enter_inline_mono(children);
      children.iter().for_each(|n| eval_inline(n, ctx, backend));
      backend.exit_inline_mono(children);
    }
    InlinePassthru(children) => {
      backend.enter_inline_passthrough(children);
      children.iter().for_each(|n| eval_inline(n, ctx, backend));
      backend.exit_inline_passthrough(children);
    }
    SpecialChar(char) => backend.visit_inline_specialchar(char),
    Text(text) => backend.visit_inline_text(text.as_str()),
    Newline => backend.visit_joining_newline(),
    Italic(children) => {
      backend.enter_inline_italic(children);
      children.iter().for_each(|n| eval_inline(n, ctx, backend));
      backend.exit_inline_italic(children);
    }
    Highlight(children) => {
      backend.enter_inline_highlight(children);
      children.iter().for_each(|n| eval_inline(n, ctx, backend));
      backend.exit_inline_highlight(children);
    }
    Subscript(children) => {
      backend.enter_inline_subscript(children);
      children.iter().for_each(|n| eval_inline(n, ctx, backend));
      backend.exit_inline_subscript(children);
    }
    Superscript(children) => {
      backend.enter_inline_superscript(children);
      children.iter().for_each(|n| eval_inline(n, ctx, backend));
      backend.exit_inline_superscript(children);
    }
    Quote(kind, children) => {
      backend.enter_inline_quote(*kind, children);
      children.iter().for_each(|n| eval_inline(n, ctx, backend));
      backend.exit_inline_quote(*kind, children);
    }
    LitMono(text) => {
      backend.enter_inline_lit_mono(text);
      text.iter().for_each(|n| eval_inline(n, ctx, backend));
      backend.exit_inline_lit_mono(text);
    }
    CurlyQuote(kind) => backend.visit_curly_quote(*kind),
    MultiCharWhitespace(ws) => backend.visit_multichar_whitespace(ws.as_str()),
    Macro(Footnote { id, text }) => {
      backend.enter_footnote(id.as_deref(), text.as_ref().map(|t| t.as_slice()));
      if let Some(text) = text {
        text.iter().for_each(|node| eval_inline(node, ctx, backend));
      }
      backend.exit_footnote(id.as_deref(), text.as_ref().map(|t| t.as_slice()));
    }
    Macro(Image { target, attrs, .. }) => backend.visit_image_macro(target, attrs),
    Macro(Button(text)) => backend.visit_button_macro(text),
    Macro(Link { target, attrs, scheme, caret }) => {
      let in_xref = *ctx.resolving_xref.borrow();
      if let Some(Some(nodes)) = attrs.as_ref().and_then(|a| a.positional.first()) {
        backend.enter_link_macro(target, attrs.as_ref(), *scheme, in_xref, true, *caret);
        nodes.iter().for_each(|n| eval_inline(n, ctx, backend));
        backend.exit_link_macro(target, attrs.as_ref(), *scheme, in_xref, true);
      } else {
        backend.enter_link_macro(target, attrs.as_ref(), *scheme, in_xref, false, *caret);
        backend.exit_link_macro(target, attrs.as_ref(), *scheme, in_xref, false);
      }
    }
    Macro(Keyboard { keys, .. }) => {
      backend.visit_keyboard_macro(&keys.iter().map(|s| s.as_str()).collect::<Vec<&str>>())
    }
    Macro(Menu(items)) => {
      backend.visit_menu_macro(&items.iter().map(|s| s.src.as_str()).collect::<Vec<&str>>())
    }
    Macro(Xref { target, linktext, kind }) => {
      let anchors = ctx.doc.anchors.borrow();
      let anchor = anchors.get(utils::xref::get_id(&target.src));
      let is_biblio = anchor.map(|a| a.is_biblio).unwrap_or(false);
      backend.enter_xref(target, linktext.as_ref().map(|t| t.as_slice()), *kind);
      if ctx.resolving_xref.replace(true) {
        backend.visit_missing_xref(target, *kind, ctx.doc.title());
      } else if let Some(text) = anchor
        .map(|anchor| {
          anchor
            .reftext
            .as_ref()
            .unwrap_or(linktext.as_ref().unwrap_or(&anchor.title))
        })
        .filter(|text| !text.is_empty())
      {
        backend.enter_xref_text(text, is_biblio);
        text.iter().for_each(|node| eval_inline(node, ctx, backend));
        backend.exit_xref_text(text, is_biblio);
      } else if let Some(text) = linktext {
        backend.enter_xref_text(text, is_biblio);
        text.iter().for_each(|node| eval_inline(node, ctx, backend));
        backend.exit_xref_text(text, is_biblio);
      } else {
        backend.visit_missing_xref(target, *kind, ctx.doc.title());
      }
      ctx.resolving_xref.replace(false);
      backend.exit_xref(target, linktext.as_ref().map(|t| t.as_slice()), *kind);
    }
    Macro(Icon { target, attrs }) => backend.visit_icon_macro(target, attrs),
    Macro(Plugin(plugin_macro)) => backend.visit_plugin_macro(plugin_macro),
    InlineAnchor(id) => backend.visit_inline_anchor(id),
    BiblioAnchor(id) => {
      backend.visit_biblio_anchor(
        id,
        ctx
          .doc
          .anchors
          .borrow()
          .get(id)
          .and_then(|anchor| anchor.reftext.as_ref()?.single_text()),
      );
    }
    LineBreak => backend.visit_linebreak(),
    CalloutNum(callout) => backend.visit_callout(*callout),
    CalloutTuck(comment) => backend.visit_callout_tuck(comment),
    TextSpan(attrs, nodes) => {
      backend.enter_text_span(attrs, nodes);
      nodes.iter().for_each(|n| eval_inline(n, ctx, backend));
      backend.exit_text_span(attrs, nodes);
    }
    Symbol(kind) => backend.visit_symbol(*kind),
    LineComment(_) | Discarded => {}
  }
}

fn eval_table_row(row: &Row, section: TableSection, ctx: &Ctx, backend: &mut impl Backend) {
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
          paragraph.iter().for_each(|n| eval_inline(n, ctx, backend));
          backend.exit_cell_paragraph(cell, section);
        });
      }
      CellContent::Literal(nodes) => {
        nodes.iter().for_each(|n| eval_inline(n, ctx, backend));
      }
      CellContent::AsciiDoc(document) => {
        let mut cell_backend = backend.asciidoc_table_cell_backend();
        visit(document, &mut cell_backend);
        backend.visit_asciidoc_table_cell_result(cell_backend);
      }
    }
    backend.exit_table_cell(cell, section);
  });
  backend.exit_table_row(row, section);
}

fn eval_toc_at(
  positions: &[TocPosition],
  macro_block: Option<&Block>,
  ctx: &Ctx,
  backend: &mut impl Backend,
) {
  let Some(toc) = &ctx.doc.toc else {
    return;
  };
  if !positions.contains(&toc.position) || toc.nodes.is_empty() {
    return;
  }
  backend.enter_toc(toc, macro_block);
  eval_toc_level(&toc.nodes, ctx, backend);
  backend.exit_toc(toc);
}

fn eval_toc_level(nodes: &[TocNode], ctx: &Ctx, backend: &mut impl Backend) {
  if let Some(first) = nodes.first() {
    backend.enter_toc_level(first.level, nodes);
    nodes.iter().for_each(|node| {
      backend.enter_toc_node(node);
      backend.enter_toc_content(&node.title);
      node.title.iter().for_each(|n| eval_inline(n, ctx, backend));
      backend.exit_toc_content(&node.title);
      eval_toc_level(&node.children, ctx, backend);
      backend.exit_toc_node(node);
    });
    backend.exit_toc_level(first.level, nodes);
  }
}
