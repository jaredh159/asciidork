use bumpalo::Bump;
use lazy_static::lazy_static;
use regex::Regex;

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
macro_rules! assert_doc_content {
  ($input:expr, $expected:expr$(,)?) => {{
    let content = parse_doc_content!($input);
    assert_eq!(content, $expected);
  }};
}

#[macro_export]
macro_rules! assert_toc {
  ($input:expr, $expected:expr$(,)?) => {{
    let toc = parse_toc!($input);
    assert_eq!(toc, $expected);
  }};
}

#[macro_export]
macro_rules! assert_block {
  ($input:expr, $expected:expr$(,)?) => {{
    let block = parse_single_block!($input);
    assert_eq!(block, $expected);
  }};
}

#[macro_export]
macro_rules! assert_inlines {
  ($input:expr, $expected:expr$(,)?) => {{
    let inlines = parse_inline_nodes!($input);
    assert_eq!(inlines, $expected);
  }};
}

#[macro_export]
macro_rules! assert_blocks {
  ($input:expr, $expected:expr$(,)?) => {{
    let blocks = parse_blocks!($input);
    assert_eq!(blocks, $expected);
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
    InlineNode::new($node, SourceLocation::new($range.start, $range.end))
  };
  ($text:expr; $range:expr) => {
    InlineNode::new(
      Inline::Text(bstr!($text)),
      SourceLocation::new($range.start, $range.end),
    )
  };
}

#[macro_export]
macro_rules! just {
  ($text:expr, $range:expr$(,)?) => {{
    let mut vs = bumpalo::collections::Vec::new_in(leaked_bump());
    vs.push(node!($text; $range));
    vs.into()
  }}
}

#[macro_export]
macro_rules! empty_block {
  ($range:expr) => {
    Block {
      meta: ChunkMeta::empty($range.start),
      context: BlockContext::Paragraph,
      content: BlockContent::Simple(nodes![]),
      loc: SourceLocation::new($range.start, $range.end),
    }
  };
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

#[macro_export]
macro_rules! bstr {
  ($text:expr) => {{
    bumpalo::collections::String::from_str_in($text, leaked_bump())
  }};
}

#[macro_export]
macro_rules! src {
  ($text:expr, $range:expr$(,)?) => {
    SourceString::new(
      bumpalo::collections::String::from_str_in($text, leaked_bump()),
      SourceLocation::new($range.start, $range.end),
    )
  };
}

#[macro_export]
macro_rules! empty_list_item {
  () => {
    ListItem {
      marker: ListMarker::Star(1),
      marker_src: src!("", 0..0),
      principle: just!("", 0..0),
      type_meta: ListItemTypeMeta::None,
      blocks: vecb![],
    }
  };
}

#[macro_export]
macro_rules! simple_text_block {
  ($text:expr, $range:expr$(,)?) => {
    Block {
      context: BlockContext::Paragraph,
      content: BlockContent::Simple(nodes![node!($text; $range)]),
      ..empty_block!($range)
    }
  }
}

#[macro_export]
macro_rules! html {
  ($s:expr) => {{
    let expected = ::indoc::indoc!($s);
    test_utils::NEWLINES_RE
      .replace_all(expected, "")
      .to_string()
  }};
  ($outer:expr, $pre:expr) => {{
    let outer = ::indoc::indoc!($outer);
    let pre = ::indoc::indoc!($pre).trim();
    let sans_newlines = test_utils::NEWLINES_RE.replace_all(outer, "").to_string();
    sans_newlines.replace("{}", &pre).to_string()
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
macro_rules! error {
  ($s:expr) => {
    ::indoc::indoc!($s)
  };
}

#[macro_export]
macro_rules! assert_error {
  ($input:expr, $expected:expr) => {
    let err = parse_error!($input);
    assert_eq!(err.plain_text(), $expected, from: $input);
  };
}

#[macro_export]
macro_rules! test_error {
  ($name:ident, $input:expr, $expected:expr) => {
    #[test]
    fn $name() {
      assert_error!($input, $expected);
    }
  };
}

#[macro_export]
macro_rules! test_inlines {
  ($name:ident, $input:expr, $expected:expr) => {
    #[test]
    fn $name() {
      assert_inlines!($input, $expected);
    }
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
macro_rules! assert_html_contains {
  ($html:expr, $needle:expr, from: $adoc:expr$(,)?) => {{
    let newline = if $adoc.ends_with('\n') { "" } else { "\n" };
    assert!(
      $html.contains(&$needle),
      "\nhtml from adoc did not contain \x1b[32m{}\x1b[0m\n\n\x1b[2m```adoc\x1b[0m\n{}{}\x1b[2m```\x1b[0m\n\n\x1b[2m```html\x1b[0m\n{}\x1b[2m```\x1b",
      $needle,
      $adoc,
      newline,
      $html,
    );
  }};
}

#[macro_export]
macro_rules! parse_blocks {
  ($input:expr) => {
    parse_doc_content!($input)
      .blocks()
      .expect("expected blocks")
      .clone()
  };
}

#[macro_export]
macro_rules! parse_single_block {
  ($input:expr) => {{
    let blocks = parse_blocks!($input);
    if blocks.len() != 1 {
      panic!("expected one block, found {}", blocks.len());
    }
    blocks[0].clone()
  }};
}

#[macro_export]
macro_rules! parse_doc_content {
  ($input:expr) => {{
    let parser = Parser::new(leaked_bump(), $input);
    parser.parse().unwrap().document.content
  }};
}

#[macro_export]
macro_rules! parse_toc {
  ($input:expr) => {{
    let parser = Parser::new(leaked_bump(), $input);
    parser.parse().unwrap().document.toc.expect("expected toc")
  }};
}

#[macro_export]
macro_rules! parse_errors {
  ($input:expr) => {{
    let parser = Parser::new(leaked_bump(), $input);
    parser.parse().err().expect("expected parse failure")
  }};
}

#[macro_export]
macro_rules! parse_error {
  ($input:expr) => {{
    let parser = Parser::new(leaked_bump(), $input);
    let mut diagnostics = parser.parse().err().expect(&format!(
      indoc::indoc! {"
        expected PARSE ERROR, but got none. input was:

        \x1b[2m```adoc\x1b[0m
        {}{}\x1b[2m```\x1b[0m
      "},
      $input,
      if $input.ends_with('\n') { "" } else { "\n" }
    ));
    if diagnostics.len() != 1 {
      panic!("expected 1 diagnostic, found {}", diagnostics.len());
    }
    diagnostics.pop().unwrap()
  }};
}

#[macro_export]
macro_rules! list_block_data {
  ($block:expr) => {
    match $block.content {
      BlockContent::List { items, variant, depth } => {
        Some(($block.context, items.clone(), variant, depth))
      }
      _ => None,
    }
  };
}

#[macro_export]
macro_rules! parse_inline_nodes {
  ($input:expr) => {{
    let block = parse_single_block!($input);
    match block.content {
      BlockContent::Simple(nodes) => nodes,
      _ => panic!("expected simple block content"),
    }
  }};
}

#[macro_export]
macro_rules! parse_list {
  ($input:expr) => {{
    let block = parse_single_block!($input);
    list_block_data!(block).expect("expected list content")
  }};
}

#[macro_export]
macro_rules! parse_section {
  ($input:expr) => {{
    match parse_doc_content!($input) {
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
macro_rules! s {
  (in $bump:expr; $s:expr) => {
    bumpalo::collections::String::from_str_in($s, $bump)
  };
}
