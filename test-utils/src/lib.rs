use bumpalo::Bump;
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
  pub static ref NEWLINES_RE: Regex = Regex::new(r"(?m)\n\s*").unwrap();
}

#[must_use]
pub fn leaked_bump() -> &'static Bump {
  Box::leak(Box::new(Bump::new()))
}

#[macro_export]
macro_rules! read_line {
  ($input:expr) => {{
    let mut parser = test_parser!($input);
    parser.read_line().unwrap().unwrap()
  }};
}

#[macro_export]
macro_rules! test_parser {
  ($input:expr) => {{
    Parser::from_str(
      $input,
      SourceFile::Path(Path::new("test.adoc")),
      leaked_bump(),
    )
  }};
}

#[macro_export]
macro_rules! doc_meta {
  ($doctype:expr) => {{
    let mut m = DocumentMeta::default();
    m.set_doctype($doctype);
    _ = m.insert_job_attr("docname", JobAttr::readonly("test"));
    _ = m.insert_job_attr("docdir", JobAttr::readonly(""));
    _ = m.insert_job_attr("docfilesuffix", JobAttr::readonly(".adoc"));
    _ = m.insert_job_attr("docfile", JobAttr::readonly(""));
    _ = m.insert_job_attr("asciidork-docfilename", JobAttr::readonly("test.adoc"));
    m
  }};
}

#[macro_export]
macro_rules! test_lexer {
  ($input:expr) => {{
    Lexer::from_str(leaked_bump(), SourceFile::Tmp, $input)
  }};
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
    expect_eq!(content, $expected, from: $input);
  }};
  (resolving: $bytes:expr, $input:expr, $expected:expr$(,)?) => {{
    let content = parse_doc_content!($input, $bytes);
    expect_eq!(content, $expected, from: $input);
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
    expect_eq!(block, $expected, from: $input);
  }};
}

#[macro_export]
macro_rules! parse_table {
  ($input:expr) => {{
    let block = parse_single_block!($input);
    match block.content {
      BlockContent::Table(table) => table,
      _ => panic!("expected table block content"),
    }
  }};
}

#[macro_export]
macro_rules! assert_table {
  ($input:expr, $expected:expr$(,)?) => {{
    let table = parse_table!($input);
    expect_eq!(table, $expected, from: $input);
  }};
}

#[macro_export]
macro_rules! assert_table_loose {
  ($input:expr, $expected:expr$(,)?) => {{
    let block = parse_single_block_loose!($input);
    let table = match block.content {
      BlockContent::Table(table) => table,
      _ => panic!("expected table block content"),
    };
    expect_eq!(table, $expected, from: $input);
  }};
}

#[macro_export]
macro_rules! assert_inlines {
  ($input:expr, $expected:expr$(,)?) => {{
    let inlines = parse_inline_nodes!($input);
    eq!(inlines, $expected, from: $input);
  }};
}

#[macro_export]
macro_rules! assert_blocks {
  ($input:expr, $expected:expr$(,)?) => {
    let blocks = parse_blocks!($input);
    expect_eq!(blocks, $expected, from: $input);
  };
}

#[macro_export]
macro_rules! assert_section {
  ($input:expr, reftext: $reftext:expr, $expected:expr$(,)?) => {
    let (section, refs) = parse_section!($input);
    expect_eq!(section, $expected);
    let refs = refs.borrow();
    let xref = refs
      .get(&section.id.clone().expect("section id"))
      .expect("expected parsed section to have xref");
    expect_eq!(xref.title, section.heading);
    expect_eq!(xref.reftext, $reftext);
  };
  ($input:expr, $expected:expr$(,)?) => {
    assert_section!($input, reftext: None, $expected);
  };
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
    node!($node, $range, depth: 0)
  };
  ($node:expr, $range:expr, depth: $depth:expr$(,)?) => {
    InlineNode::new($node, SourceLocation::new_depth($range.start, $range.end, $depth))
  };
  ($text:expr; $range:expr, depth: $depth:expr) => {
    InlineNode::new(
      Inline::Text(bstr!($text)),
      SourceLocation::new_depth($range.start, $range.end, $depth),
    )
  };
  ($text:expr; $range:expr) => {
    node!($text; $range, depth: 0)
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
macro_rules! empty_cell {
  () => {
    Cell {
      content: CellContent::Default(vecb![]),
      col_span: 1,
      row_span: 1,
      h_align: HorizontalAlignment::Left,
      v_align: VerticalAlignment::Top,
    }
  };
}

#[macro_export]
macro_rules! cell {
  (d: $text:expr, $range:expr$(,)?) => {
    Cell {
      content: CellContent::Default(vecb![just!($text, $range)]),
      ..empty_cell!()
    }
  };
  (e: $text:expr, $range:expr$(,)?) => {
    Cell {
      content: CellContent::Emphasis(vecb![just!($text, $range)]),
      ..empty_cell!()
    }
  };
  (s: $text:expr, $range:expr$(,)?) => {
    Cell {
      content: CellContent::Strong(vecb![just!($text, $range)]),
      ..empty_cell!()
    }
  };
  (l: $text:expr, $range:expr$(,)?) => {
    Cell {
      content: CellContent::Literal(just!($text, $range)),
      ..empty_cell!()
    }
  };
}

#[macro_export]
macro_rules! empty_block {
  ($start:expr) => {
    Block {
      meta: ChunkMeta::empty($start),
      context: BlockContext::Paragraph,
      content: BlockContent::Simple(nodes![]),
    }
  };
}

#[macro_export]
macro_rules! empty_table {
  () => {
    Table {
      col_widths: ColWidths::new(vecb![]),
      header_row: None,
      rows: vecb![],
      footer_row: None,
    }
  };
}

#[macro_export]
macro_rules! empty_document {
  () => {
    Document::new(leaked_bump())
  };
}

#[macro_export]
macro_rules! assert_list {
  ($input:expr, $expected_ctx:expr, $expected_items:expr) => {
    let (context, items, ..) = parse_list!($input);
    expect_eq!(context, $expected_ctx, from: $input);
    expect_eq!(items, $expected_items, from: $input);
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
      ..empty_block!($range.start)
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
macro_rules! bytes {
  ($s:expr) => {
    ::indoc::indoc!($s).as_bytes()
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
  ($name:ident, $input:expr, $expected:expr) => {
    #[test]
    fn $name() {
      let err = parse_error!($input);
      expect_eq!(err.plain_text(), $expected, from: $input);
    }
  };
  ($name:ident, resolving: $bytes:expr, $input:expr, $expected:expr) => {
    #[test]
    fn $name() {
      let err = parse_error!($input, resolving: $bytes);
      expect_eq!(err.plain_text(), $expected, from: $input);
    }
  };
}

#[macro_export]
macro_rules! assert_no_error {
  ($name:ident, $input:expr) => {
    #[test]
    fn $name() {
      let mut parser = test_parser!($input);
      let result = parser.parse().expect("expected parse success");
      expect_eq!(result.warnings.is_empty(), true, from: $input);
    }
  };
  ($name:ident, resolving: $bytes:expr, $input:expr) => {
    #[test]
    fn $name() {
      let mut parser = test_parser!($input);
      parser.apply_job_settings(asciidork_core::JobSettings::r#unsafe());
      parser.set_resolver(Box::new(asciidork_parser::includes::ConstResolver(
        Vec::from($bytes),
      )));
      let result = parser.parse().expect("expected parse success");
      expect_eq!(result.warnings.is_empty(), true, from: $input);
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
macro_rules! test_inlines_loose {
  ($name:ident, $input:expr, $expected:expr) => {
    #[test]
    fn $name() {
      let mut settings = ::asciidork_core::JobSettings::embedded();
      settings.strict = false;
      let mut parser = test_parser!($input);
      parser.apply_job_settings(settings);
      let content = parser.parse().unwrap().document.content;
      let blocks = content.blocks().expect("expected blocks").clone();
      if blocks.len() != 1 {
        panic!("expected one block, found {}", blocks.len());
      }
      let inlines = match blocks[0].clone().content {
        BlockContent::Simple(nodes) => nodes,
        _ => panic!("expected simple block content"),
      };
      expect_eq!(inlines, $expected, from: $input);
    }
  };
}

#[macro_export]
macro_rules! expect_eq {
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
macro_rules! parse_blocks_loose {
  ($input:expr) => {
    parse_doc_content_loose!($input)
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
macro_rules! parse_single_block_loose {
  ($input:expr) => {{
    let blocks = parse_blocks_loose!($input);
    if blocks.len() != 1 {
      panic!("expected one block, found {}", blocks.len());
    }
    blocks[0].clone()
  }};
}

#[macro_export]
macro_rules! const_resolver {
  ($bytes:expr) => {{
    Box::new(asciidork_parser::includes::ConstResolver(Vec::from($bytes)))
  }};
}

#[macro_export]
macro_rules! error_resolver {
  ($error:expr) => {{
    Box::new(asciidork_parser::includes::ErrorResolver($error))
  }};
}

#[macro_export]
macro_rules! parse_doc_content {
  ($input:expr) => {{
    let parser = test_parser!($input);
    parser.parse().unwrap().document.content
  }};
  ($input:expr, $bytes:expr) => {{
    let mut parser = test_parser!($input);
    parser.apply_job_settings(asciidork_core::JobSettings::r#unsafe());
    parser.set_resolver(const_resolver!($bytes));
    parser.parse().unwrap().document.content
  }};
}

#[macro_export]
macro_rules! parse_doc_content_loose {
  ($input:expr) => {{
    let mut settings = ::asciidork_core::JobSettings::embedded();
    settings.strict = false;
    let mut parser = test_parser!($input);
    parser.apply_job_settings(settings);
    parser.parse().unwrap().document.content
  }};
}

#[macro_export]
macro_rules! parse {
  ($input:expr) => {{
    let parser = test_parser!($input);
    parser.parse()
  }};
}

#[macro_export]
macro_rules! parse_doc {
  ($input:expr) => {{
    let parser = test_parser!($input);
    parser.parse().unwrap().document
  }};
}

#[macro_export]
macro_rules! parse_toc {
  ($input:expr) => {{
    let parser = test_parser!($input);
    parser.parse().unwrap().document.toc.expect("expected toc")
  }};
}

#[macro_export]
macro_rules! parse_errors {
  ($input:expr) => {{
    let parser = Parser::from_str($input, leaked_bump());
    parser.parse().err().expect("expected parse failure")
  }};
}

#[macro_export]
macro_rules! parse_error {
  ($input:expr) => {{
    let parser = test_parser!($input);
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
  ($input:expr, resolving: $bytes:expr) => {{
    let mut parser = test_parser!($input);
    parser.apply_job_settings(asciidork_core::JobSettings::r#unsafe());
    parser.set_resolver(Box::new(asciidork_parser::includes::ConstResolver(
      Vec::from($bytes),
    )));
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
    let doc = parse_doc!($input);
    match doc.content {
      ::asciidork_ast::DocContent::Sectioned { mut sections, .. } => {
        if sections.len() != 1 {
          panic!("expected one section, found {}", sections.len());
        }
        (sections.remove(0), doc.anchors)
      }
      _ => panic!("expected block content"),
    }
  }};
}
