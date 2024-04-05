use std::ops::Range;

use asciidork_ast::{prelude::*, *};
use bumpalo::collections::String as BumpString;
use bumpalo::Bump;
use lazy_static::lazy_static;
use regex::Regex;

pub mod ast_helpers;
pub use ast_helpers::*;

lazy_static! {
  pub static ref NEWLINES_RE: Regex = Regex::new(r"(?m)\n\s*").unwrap();
}

pub fn leaked_bump() -> &'static Bump {
  Box::leak(Box::new(Bump::new()))
}

#[macro_export]
macro_rules! assert_block_core {
  ($input:expr, $expected_ctx:expr, $expected_content:expr$(,)?) => {
    let block = parse_single_block!($input);
    assert_eq!(block.context, $expected_ctx);
    assert_eq!(block.content, $expected_content);
  };
}

#[macro_export]
macro_rules! assert_block {
  ($input:expr, $expected:expr$(,)?) => {{
    let block = parse_single_block!($input);
    assert_eq!(block, $expected);
  }};
}

#[macro_export]
macro_rules! assert_section {
  ($input:expr, $expected:expr$(,)?) => {{
    let block = parse_section!($input);
    assert_eq!(block, $expected);
  }};
}

#[macro_export]
macro_rules! nodes {
  () => (
    bumpalo::collections::Vec::new_in(leaked_bump()).into()
  );
  ($($x:expr),+ $(,)?) => ({
    let mut vs = bumpalo::collections::Vec::new_in(leaked_bump());
    $(vs.push($x);)+
    vs.into()
  });
}

#[macro_export]
macro_rules! vecb {
  () => (
    bumpalo::collections::Vec::new_in(leaked_bump()).into()
  );
  ($($x:expr),+ $(,)?) => ({
    let mut vs = bumpalo::collections::Vec::new_in(leaked_bump());
    $(vs.push($x);)+
    vs
  });
}

#[macro_export]
macro_rules! node {
  ($node:expr, $range:expr$(,)?) => {
    asciidork_ast::InlineNode::new(
      $node,
      asciidork_ast::SourceLocation::new($range.start, $range.end),
    )
  };
  ($text:expr; $range:expr) => {
    n_text($text, $range.start, $range.end, leaked_bump())
  };
}

pub fn just(text: &'static str, range: Range<usize>) -> InlineNodes<'static> {
  let mut vs = bumpalo::collections::Vec::new_in(leaked_bump());
  vs.push(node!(text; range));
  vs.into()
}

pub fn empty_block(range: Range<usize>) -> Block<'static> {
  Block {
    meta: asciidork_ast::ChunkMeta::empty(range.start),
    context: asciidork_ast::BlockContext::Paragraph,
    content: asciidork_ast::BlockContent::Simple(nodes![]),
    loc: asciidork_ast::SourceLocation::new(range.start, range.end),
  }
}

pub fn empty_list_item() -> ListItem<'static> {
  ListItem {
    marker: ListMarker::Star(1),
    marker_src: src("", 0..0),
    principle: just("", 0..0),
    checklist: None,
    blocks: vecb![],
  }
}

#[macro_export]
macro_rules! assert_list {
  ($input:expr, $expected_ctx:expr, $expected_items:expr) => {
    let (context, items, ..) = parse_list!($input);
    assert_eq!(context, $expected_ctx, from: $input);
    assert_eq!(items, $expected_items, from: $input);
  };
}

#[macro_export]
macro_rules! attr_list {
  ($range:expr) => {
    asciidork_ast::AttrList::new(
      asciidork_ast::SourceLocation::new($range.start, $range.end),
      leaked_bump(),
    )
  };
  ($range:expr, named: $($pairs:expr),+ $(,)?) => {{
    let mut named = asciidork_ast::Named::new_in(leaked_bump());
    $(named.insert($pairs.0, $pairs.1);)+
    AttrList { named, ..attr_list!($range.start..$range.end) }
  }};
}

pub fn bstr(text: &'static str) -> BumpString<'static> {
  BumpString::from_str_in(text, leaked_bump())
}

pub fn src(text: &'static str, range: Range<usize>) -> SourceString<'static> {
  SourceString::new(
    BumpString::from_str_in(text, leaked_bump()),
    SourceLocation::new(range.start, range.end),
  )
}

#[macro_export]
macro_rules! html {
  ($s:expr) => {{
    let expected = ::indoc::indoc!($s);
    test_utils::NEWLINES_RE
      .replace_all(expected, "")
      .to_string()
  }};
}

#[macro_export]
macro_rules! adoc {
  ($s:expr) => {
    ::indoc::indoc!($s)
  };
}

#[macro_export]
macro_rules! raw_html {
  ($s:expr) => {
    ::indoc::indoc!($s)
  };
}

#[macro_export]
macro_rules! assert_eq {
  ($left:expr, $right:expr$(,)?) => {{
    ::pretty_assertions::assert_eq!(@ $left, $right, "", "");
  }};
  ($left:expr, $right:expr, from: $adoc:expr) => {{
    ::pretty_assertions::assert_eq!(
      $left,
      $right,
      "input was:\n\n\x1b[2m```adoc\x1b[0m\n{}{}\x1b[2m```\x1b[0m\n",
      $adoc,
      if $adoc.ends_with('\n') { "" } else { "\n" }
    );
  }};
}

#[macro_export]
macro_rules! parse_single_block {
  ($input:expr) => {{
    let mut parser = Parser::new($crate::leaked_bump(), $input);
    let doc_content = parser.parse().unwrap().document.content;
    match doc_content {
      ::asciidork_ast::DocContent::Blocks(mut blocks) => {
        if blocks.len() != 1 {
          panic!("expected one block, found {}", blocks.len());
        }
        blocks.remove(0)
      }
      _ => panic!("expected block content"),
    }
  }};
}

#[macro_export]
macro_rules! parse_list {
  ($input:expr) => {{
    let block = parse_single_block!($input);
    match block.content {
      ::asciidork_ast::BlockContent::List { variant, depth, items } => {
        (block.context, items, variant, depth)
      }
      _ => panic!("expected list content"),
    }
  }};
}

#[macro_export]
macro_rules! parse_section {
  ($input:expr) => {{
    let mut parser = Parser::new($crate::leaked_bump(), $input);
    let doc_content = parser.parse().unwrap().document.content;
    match doc_content {
      ::asciidork_ast::DocContent::Sectioned { mut sections, .. } => {
        if sections.len() != 1 {
          panic!("expected one section, found {}", sections.len());
        }
        sections.remove(0)
      }
      _ => panic!("expected block content"),
    }
  }};
}

#[macro_export]
macro_rules! parse_list_old {
  ($input:expr, $list:ident, $bump:ident) => {
    let $bump = &Bump::new();
    let mut parser = Parser::new($bump, $input);
    let lines = parser.read_lines().unwrap();
    let $list = parser.parse_list(lines, None).unwrap();
  };
}

#[macro_export]
macro_rules! s {
  (in $bump:expr; $s:expr) => {
    bumpalo::collections::String::from_str_in($s, $bump)
  };
}

pub mod attrs {
  use super::*;

  pub fn named(
    pairs: &[(&'static str, Range<usize>, &'static str, Range<usize>)],
  ) -> asciidork_ast::AttrList<'static> {
    let mut attrs = AttrList::new(SourceLocation::new(0, 0), leaked_bump());
    attrs.loc.start = pairs[0].1.start - 1;
    attrs.loc.end = pairs[pairs.len() - 1].3.end + 1;
    for (name, name_range, value, value_range) in pairs {
      attrs.named.insert(
        src(name, name_range.clone()),
        src(value, value_range.clone()),
      );
    }
    attrs
  }

  pub fn pos(text: &'static str, range: Range<usize>) -> asciidork_ast::AttrList<'static> {
    let mut attrs = AttrList::new(SourceLocation::new(0, 0), leaked_bump());
    attrs.loc.start = range.start - 1;
    attrs.loc.end = range.end + 1;
    attrs.positional.push(Some(nodes![node!(text; range)]));
    attrs
  }
}
