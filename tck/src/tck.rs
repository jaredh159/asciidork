use serde_json::{Map, Value};

use asciidork_ast::prelude::*;
use asciidork_ast::InlineNodes;
use asciidork_parser::prelude::*;

use crate::loc::*;

pub fn gen_asg_doc(adoc: &str) -> String {
  let bump = &Bump::with_capacity(adoc.len() * 4);
  let mut tck = Tck::new(adoc, bump);
  let doc = tck.gen_doc();
  serde_json::to_string(&doc).unwrap()
}

pub fn gen_asg_inline(adoc: &str) -> String {
  let bump = &Bump::with_capacity(adoc.len() * 4);
  let mut tck = Tck::new(adoc, bump);
  let inline = tck.gen_single_inline();
  serde_json::to_string(&inline).unwrap()
}

struct Tck<'arena> {
  pub bump: &'arena Bump,
  pub doc: Document<'arena>,
  pub src: Vec<u8>,
}

impl<'arena> Tck<'arena> {
  fn new(adoc: &str, bump: &'arena Bump) -> Self {
    let src: Vec<u8> = adoc.trim_end().bytes().collect();
    let parser = Parser::from_str(adoc, SourceFile::Tmp, bump);
    let doc = parser.parse().unwrap().document;
    Self { bump, doc, src }
  }

  fn gen_doc(&mut self) -> Value {
    let mut doc = Map::new();
    doc.set("name", "document");
    doc.set("type", "block");
    if let Some(header) = &self.doc.header {
      self.gen_doc_header(header.clone(), &mut doc);
    }

    match self.doc.content.clone() {
      DocContent::Blocks(blocks) => {
        if !blocks.is_empty() {
          let blocks = blocks.iter().map(|b| self.gen_block(b)).collect();
          doc.set_val("blocks", Value::Array(blocks));
        }
      }
      DocContent::Sections(Sectioned { sections, .. }) => {
        let sections = sections.iter().map(|s| self.gen_section(s)).collect();
        doc.set_val("blocks", Value::Array(sections));
      }
      _ => todo!("unhandled doc content: {:?}", self.doc.content),
    }
    let loc_span = LocSpan::new(
      Loc::new(1, 1),
      self.loc_from_pos(self.src.len() as u32 - 1).incr_column(),
    );
    self.push_locspan(loc_span, &mut doc);
    Value::Object(doc)
  }

  fn gen_single_inline(&mut self) -> Value {
    let doc = self.gen_doc();
    let Value::Object(mut doc) = doc else {
      panic!("expected object");
    };
    let mut blocks = doc.remove("blocks").unwrap();
    let Value::Array(ref mut blocks) = blocks else {
      panic!("expected blocks");
    };
    assert_eq!(blocks.len(), 1);
    let block = &mut blocks[0];
    let Value::Object(ref mut block) = block else {
      panic!("expected block object");
    };
    block.remove("inlines").unwrap()
  }

  fn gen_doc_header(&mut self, ast_header: DocHeader<'arena>, doc: &mut Map<String, Value>) {
    let header_attrs: Vec<(String, String)> = self
      .doc
      .meta
      .header_attrs()
      .iter()
      .filter(|(key, _)| !key.starts_with("_derived_"))
      .map(|(key, value)| (key.to_string(), value.str().unwrap_or("").to_string()))
      .collect();
    let mut attrs = Map::new();
    for (key, value) in header_attrs.iter() {
      attrs.set(key, value);
    }
    doc.set_val("attributes", Value::Object(attrs));
    let mut header = Map::new();
    if let Some(doc_title) = &ast_header.title {
      header.set_val("title", Value::Array(self.node_values(&doc_title.main)));
    }
    self.push_srcloc(ast_header.loc, &mut header);
    doc.set_val("header", Value::Object(header));
  }

  fn gen_section(&mut self, ast_section: &Section<'arena>) -> Value {
    let mut section = Map::new();
    section.set("name", "section");
    section.set("type", "block");
    section.set_val("level", Value::Number(ast_section.level.into()));
    section.set_val(
      "title",
      Value::Array(self.node_values(&ast_section.heading)),
    );
    let blocks = ast_section
      .blocks
      .iter()
      .map(|b| self.gen_block(b))
      .collect();
    section.set_val("blocks", Value::Array(blocks));
    self.push_multiloc(&ast_section.loc, &mut section);
    Value::Object(section)
  }

  fn gen_block(&mut self, ast_block: &Block<'arena>) -> Value {
    let mut block = Map::new();
    block.set("type", "block");
    self.push_multiloc(&ast_block.loc, &mut block);
    match (&ast_block.context, &ast_block.content) {
      (BlockContext::Paragraph, BlockContent::Simple(nodes)) => {
        block.set("name", "paragraph");
        block.set_val("inlines", Value::Array(self.node_values(nodes)));
      }
      (BlockContext::UnorderedList, BlockContent::List { items, .. }) => {
        block.set("name", "list");
        block.set("variant", "unordered");
        block.set("marker", "*");
        let items = items.iter().map(|i| self.gen_list_item(i)).collect();
        block.set_val("items", Value::Array(items));
      }
      (BlockContext::Listing, BlockContent::Simple(nodes)) => {
        block.set("name", "listing");
        block.set("form", "delimited");
        block.set("delimiter", "----");
        block.set_val("inlines", Value::Array(self.node_values(nodes)));
      }
      (BlockContext::Sidebar, BlockContent::Compound(blocks)) => {
        block.set("name", "sidebar");
        block.set("form", "delimited");
        block.set("delimiter", "****");
        let blocks = blocks.iter().map(|b| self.gen_block(b)).collect();
        block.set_val("blocks", Value::Array(blocks));
      }
      _ => todo!("unhandled block context: {:?}", ast_block.context),
    };
    Value::Object(block)
  }

  fn gen_list_item(&mut self, item: &ListItem<'arena>) -> Value {
    let mut map = Map::new();
    map.set("name", "listItem");
    map.set("type", "block");
    map.set("marker", "*");
    map.set_val("principal", Value::Array(self.node_values(&item.principle)));
    let last = item.principle.last_loc().unwrap();
    self.push_locspan(self.locspan_from_pair(item.marker_src.loc, last), &mut map);
    Value::Object(map)
  }

  fn gen_inline_node(&mut self, ast_node: &InlineNode<'arena>) -> Value {
    let mut node = Map::new();
    match &ast_node.content {
      Inline::Text(text) => {
        node.set("name", "text");
        node.set("type", "string");
        node.set("value", text.as_str());
      }
      Inline::Bold(nodes) => {
        node.set("name", "span");
        node.set("type", "inline");
        node.set("variant", "strong");
        node.set("form", "constrained");
        node.set_val("inlines", Value::Array(self.node_values(nodes)));
      }
      _ => todo!("unhandled inline content: {:?}", &ast_node.content),
    };
    self.push_srcloc(ast_node.loc, &mut node);
    Value::Object(node)
  }

  fn node_values(&mut self, nodes: &InlineNodes<'arena>) -> Vec<Value> {
    self
      .consolidate(nodes)
      .iter()
      .map(|node| self.gen_inline_node(node))
      .collect()
  }

  fn consolidate(&self, nodes: &InlineNodes<'arena>) -> InlineNodes<'arena> {
    let mut consolidated = InlineNodes::new(self.bump);
    for node in nodes.iter() {
      if let Some(last) = consolidated.last_mut() {
        match (&mut last.content, &node.content) {
          (Inline::Text(ref mut prev), Inline::Text(current)) => {
            prev.push_str(current.as_str());
            last.loc.end = node.loc.end;
          }
          (Inline::Text(ref mut prev), Inline::Newline) => {
            prev.push('\n');
            last.loc.end = node.loc.end;
          }
          _ => consolidated.push(node.clone()),
        }
      } else {
        consolidated.push(node.clone());
      }
    }
    consolidated
  }

  fn loc_from_pos(&self, pos: u32) -> Loc {
    Loc::from_pos(pos, &self.src)
  }

  fn locspan_from_pair(&self, first: SourceLocation, last: SourceLocation) -> LocSpan {
    LocSpan::from_src_pair(first, last, &self.src)
  }

  fn push_multiloc(&mut self, multiloc: &MultiSourceLocation, map: &mut Map<String, Value>) {
    self.push_locspan(LocSpan::from_multi_loc(multiloc, &self.src), map);
  }

  fn push_locspan(&mut self, loc_span: LocSpan, map: &mut Map<String, Value>) {
    map.set_val("location", loc_span.into_value());
  }

  fn push_srcloc(&mut self, loc: SourceLocation, map: &mut Map<String, Value>) {
    self.push_locspan(LocSpan::from_src_loc(loc, &self.src), map);
  }
}

trait MapExt {
  fn set(&mut self, key: &str, value: &str);
  fn set_val(&mut self, key: &str, value: Value);
}

impl MapExt for Map<String, Value> {
  fn set(&mut self, key: &str, value: &str) {
    self.insert(key.into(), value.into());
  }
  fn set_val(&mut self, key: &str, value: Value) {
    self.insert(key.into(), value);
  }
}
