use asciidork_ast::prelude::*;
use asciidork_ast::short::block::*;
use asciidork_ast::variants::inline::*;
use asciidork_parser::Parser;
use test_utils::{assert_eq, *};

mod attrs;

#[test]
fn test_parse_literal_block() {
  assert_block!(
    adoc! {"
      [literal]
      foo `bar`
    "},
    Block {
      meta: ChunkMeta::new(Some(attrs::pos("literal", 1..8)), None, 0),
      context: Context::Literal,
      content: Content::Simple(just!("foo `bar`", 10..19)),
      ..empty_block!(0..19)
    }
  );
}

#[test]
fn test_parse_delimited_literal_block() {
  let input = adoc! {"
    ....
    foo `bar`
    baz
    ....
  "};
  assert_block!(
    input,
    Block {
      context: Context::Literal,
      content: Content::Simple(nodes![
        node!("foo `bar`"; 5..14),
        node!(JoiningNewline, 14..15),
        node!("baz"; 15..18),
      ]),
      ..empty_block!(0..23)
    }
  )
}

#[test]
fn test_parse_delimited_literal_block_w_double_newline() {
  let input = adoc! {"
    ....
    foo `bar`

    baz
    ....
  "};
  let expected = Block {
    context: Context::Literal,
    content: Content::Simple(nodes![
      node!("foo `bar`"; 5..14),
      node!(JoiningNewline, 14..15),
      node!(JoiningNewline, 15..16),
      node!("baz"; 16..19),
    ]),
    ..empty_block!(0..24)
  };
  assert_block!(input, expected);
}

#[test]
fn test_parse_listing_block() {
  assert_block!(
    adoc! {"
      [listing]
      foo `bar`
    "},
    Block {
      meta: ChunkMeta::new(Some(attrs::pos("listing", 1..8)), None, 0),
      context: Context::Listing,
      content: Content::Simple(nodes![node!("foo `bar`"; 10..19)]),
      ..empty_block!(0..19)
    }
  );
}

#[test]
fn test_parse_delimited_listing_block() {
  let input = adoc! {"
    ----
    foo `bar`
    baz
    ----
  "};
  let expected = Block {
    context: Context::Listing,
    content: Content::Simple(nodes![
      node!("foo `bar`"; 5..14),
      node!(JoiningNewline, 14..15),
      node!("baz"; 15..18),
    ]),
    ..empty_block!(0..23)
  };
  assert_block!(input, expected);
}

#[test]
fn test_parse_delimited_listing_block_w_double_newline() {
  let input = adoc! {"
    ----
    foo `bar`

    baz
    ----
  "};
  let expected = Block {
    context: Context::Listing,
    content: Content::Simple(nodes![
      node!("foo `bar`"; 5..14),
      node!(JoiningNewline, 14..15),
      node!(JoiningNewline, 15..16),
      node!("baz"; 16..19),
    ]),
    ..empty_block!(0..24)
  };
  assert_block!(input, expected);
}
