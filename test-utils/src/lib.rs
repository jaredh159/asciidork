use std::ops::Range;

use asciidork_ast::prelude::*;
use lazy_static::lazy_static;
use regex::Regex;

pub mod ast_helpers;
pub use ast_helpers::*;

lazy_static! {
  pub static ref NEWLINES_RE: Regex = Regex::new(r"(?m)\n\s*").unwrap();
}

#[macro_export]
macro_rules! leak_test_bump {
  () => {
    Box::leak(Box::new(bumpalo::Bump::new()))
  };
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
macro_rules! nodes {
  () => (
    bumpalo::collections::Vec::new_in(leak_test_bump!()).into()
  );
  ($($x:expr),+ $(,)?) => ({
    let mut vs = bumpalo::collections::Vec::new_in(leak_test_bump!());
    $(vs.push($x);)+
    vs.into()
  });
}

#[macro_export]
macro_rules! vecb {
  () => (
    bumpalo::collections::Vec::new_in(leak_test_bump!()).into()
  );
  ($($x:expr),+ $(,)?) => ({
    let mut vs = bumpalo::collections::Vec::new_in(leak_test_bump!());
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
    n_text($text, $range.start, $range.end, leak_test_bump!())
  };
}

#[macro_export]
macro_rules! empty_block {
  ($range:expr) => {
    Block {
      meta: asciidork_ast::ChunkMeta::empty($range.start),
      context: asciidork_ast::BlockContext::Paragraph,
      content: asciidork_ast::BlockContent::Simple(nodes![]),
      loc: asciidork_ast::SourceLocation::new($range.start, $range.end),
    }
  };
}

#[macro_export]
macro_rules! attr_list {
  ($range:expr) => {
    asciidork_ast::AttrList::new(
      asciidork_ast::SourceLocation::new($range.start, $range.end),
      leak_test_bump!(),
    )
  };
  ($range:expr, named: $($pairs:expr),+ $(,)?) => {{
    let mut named = asciidork_ast::Named::new_in(leak_test_bump!());
    $(named.insert($pairs.0, $pairs.1);)+
    AttrList { named, ..attr_list!($range.start..$range.end) }
  }};
}

#[macro_export]
macro_rules! bstr {
  ($text:expr) => {
    bumpalo::collections::String::from_str_in($text, leak_test_bump!())
  };
}

#[macro_export]
macro_rules! src {
  ($text:expr, $range:expr) => {
    asciidork_ast::SourceString::new(
      bstr!($text),
      asciidork_ast::SourceLocation::new($range.start, $range.end),
    )
  };
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
    let mut parser = Parser::new(leak_test_bump!(), $input);
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
macro_rules! parse_block {
  ($input:expr, $block:ident, $bump:ident) => {
    let $bump = &bumpalo::Bump::new();
    let mut parser = Parser::new($bump, $input);
    let doc_content = parser.parse().unwrap().document.content;
    let $block = match doc_content {
      ::asciidork_ast::DocContent::Blocks(mut blocks) => {
        if blocks.len() != 1 {
          panic!("expected one block, found {}", blocks.len());
        }
        blocks.remove(0)
      }
      _ => panic!("expected block content"),
    };
  };
}

#[macro_export]
macro_rules! parse_list {
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
    let mut attrs = AttrList::new(SourceLocation::new(0, 0), leak_test_bump!());
    attrs.loc.start = pairs[0].1.start - 1;
    attrs.loc.end = pairs[pairs.len() - 1].3.end + 1;
    for (name, name_range, value, value_range) in pairs {
      attrs
        .named
        .insert(src!(*name, *name_range), src!(*value, *value_range));
    }
    attrs
  }

  pub fn pos(text: &'static str, range: Range<usize>) -> asciidork_ast::AttrList<'static> {
    let mut attrs = AttrList::new(SourceLocation::new(0, 0), leak_test_bump!());
    attrs.loc.start = range.start - 1;
    attrs.loc.end = range.end + 1;
    attrs.positional.push(Some(nodes![node!(text; range)]));
    attrs
  }
}
