use asciidork_ast::prelude::*;
use asciidork_ast::short::block::*;
use asciidork_ast::variants::inline::*;
use asciidork_parser::Parser;
use test_utils::{assert_eq, *};

#[test]
fn test_quoted_paragraph() {
  let input = adoc! {r#"
    "I hold it that a little blah,
    and as necessary in the blah."
    -- Thomas Jefferson, Papers of Thomas Jefferson: Volume 11
  "#};
  let expected = Block {
    context: Context::QuotedParagraph,
    content: Content::QuotedParagraph {
      quote: nodes![
        node!("I hold it that a little blah,"; 1..30),
        node!(JoiningNewline, 30..31),
        node!("and as necessary in the blah."; 31..60),
      ],
      attr: src("Thomas Jefferson", 65..81),
      cite: Some(src("Papers of Thomas Jefferson: Volume 11", 83..120)),
    },
    ..empty_block(0..120)
  };
  assert_block!(input, expected);
}

#[test]
fn test_quoted_paragraph_no_cite_w_attr_meta() {
  let input = adoc! {r#"
    .A Title
    [#foo]
    "I hold it that a little blah,
    and as necessary in the blah."
    -- Thomas Jefferson
  "#};
  let expected = Block {
    meta: ChunkMeta::new(
      Some(AttrList {
        id: Some(src("foo", 11..14)),
        ..attr_list!(9..15)
      }),
      Some(src("A Title", 1..8)),
      0,
    ),
    context: Context::QuotedParagraph,
    content: Content::QuotedParagraph {
      quote: nodes![
        node!("I hold it that a little blah,"; 17..46),
        node!(JoiningNewline, 46..47),
        node!("and as necessary in the blah."; 47..76),
      ],
      attr: src("Thomas Jefferson", 81..97),
      cite: None,
    },
    ..empty_block(0..97)
  };
  assert_block!(input, expected);
}

#[test]
fn test_simple_blockquote() {
  let input = adoc! {"
      [quote,author,location]
      foo
    "};
  let expected = Block {
    meta: ChunkMeta {
      attrs: Some(AttrList {
        positional: vecb![
          Some(nodes![node!("quote"; 1..6)]),
          Some(nodes![node!("author"; 7..13)]),
          Some(nodes![node!("location"; 14..22)]),
        ],
        ..attr_list!(0..23)
      }),
      ..ChunkMeta::default()
    },
    context: Context::BlockQuote,
    content: Content::Simple(nodes![node!("foo"; 24.. 27)]),
    ..empty_block(0..27)
  };
  assert_block!(input, expected,)
}

#[test]
fn test_parse_delimited_blockquote() {
  let input = adoc! {"
    [quote,author,location]
    ____
    foo
    ____
  "};
  let expected = Block {
    meta: ChunkMeta {
      attrs: Some(AttrList {
        positional: vecb![
          Some(nodes![node!("quote"; 1..6)]),
          Some(nodes![node!("author"; 7..13)]),
          Some(nodes![node!("location"; 14..22)]),
        ],
        ..attr_list!(0..23)
      }),
      ..ChunkMeta::default()
    },
    context: Context::BlockQuote,
    content: Content::Compound(vecb![Block {
      context: Context::Paragraph,
      content: Content::Simple(just("foo", 29..32)),
      ..empty_block(29..32)
    }]),
    ..empty_block(0..37)
  };
  assert_block!(input, expected);
}

#[test]
fn test_delimited_verse_block() {
  assert_block!(
    adoc! {"
      [verse,author,location]
      ____
      foo
      bar
      ____
    "},
    Block {
      meta: ChunkMeta {
        attrs: Some(AttrList {
          positional: vecb![
            Some(nodes![node!("verse"; 1..6)]),
            Some(nodes![node!("author"; 7..13)]),
            Some(nodes![node!("location"; 14..22)]),
          ],
          ..attr_list!(0..23)
        }),
        ..ChunkMeta::default()
      },
      context: Context::Verse,
      content: Content::Simple(nodes![
        node!("foo"; 29..32),
        node!(JoiningNewline, 32..33),
        node!("bar"; 33..36),
      ]),
      ..empty_block(0..41)
    }
  );
}
