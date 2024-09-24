use crate::attrs;
use asciidork_ast::prelude::*;
use asciidork_ast::short::block::*;
use asciidork_ast::variants::inline::*;
use asciidork_parser::prelude::*;
use test_utils::*;

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
        node!(Newline, 14..15),
        node!("baz"; 15..18),
      ]),
      ..empty_block!(0)
    }
  );
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
      node!(Newline, 14..15),
      node!(Newline, 15..16),
      node!("baz"; 16..19),
    ]),
    ..empty_block!(0)
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
      node!(Newline, 14..15),
      node!("baz"; 15..18),
    ]),
    ..empty_block!(0)
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
      node!(Newline, 14..15),
      node!(Newline, 15..16),
      node!("baz"; 16..19),
    ]),
    ..empty_block!(0)
  };
  assert_block!(input, expected);
}

#[test]
fn test_parse_indented_literal_block() {
  assert_block!(
    " foo bar",
    Block {
      context: Context::Literal,
      content: Content::Simple(just!("foo bar", 1..8)),
      ..empty_block!(0)
    }
  );

  assert_block!(
    "  foo bar", // 2 spaces
    Block {
      context: Context::Literal,
      content: Content::Simple(just!("foo bar", 2..9)),
      ..empty_block!(0)
    }
  );

  assert_block!(
    // second line not indented, this matches asciidoctor
    adoc! {"
       foo
      bar
    "},
    Block {
      context: Context::Literal, // <-- still literal...
      content: Content::Simple(nodes![
        node!(" foo"; 0..4), // <-- ... but preserve leading space
        node!(Newline, 4..5),
        node!("bar"; 5..8),
      ]),
      ..empty_block!(0)
    }
  );

  // [normal] overrides spacing
  assert_block!(
    adoc! {"
      [normal]
       foo
       bar
    "},
    Block {
      meta: ChunkMeta::new(Some(attrs::pos("normal", 1..7)), None, 0),
      context: Context::Paragraph,
      content: Content::Simple(nodes![
        node!("foo"; 10..13),
        node!(Newline, 13..14),
        node!("bar"; 14..18),
      ]),
    }
  );
}
