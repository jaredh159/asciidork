use asciidork_ast::prelude::*;
use asciidork_parser::Parser;
use test_utils::{assert_eq, *};

#[test]
fn test_parse_section() {
  assert_section!(
    adoc! {"
        == foo

        bar
      "},
    Section {
      meta: ChunkMeta::empty(0),
      level: 1,
      id: Some(bstr!("_foo")),
      heading: nodes![node!("foo"; 3..6)],
      blocks: vecb![Block {
        context: BlockContext::Paragraph,
        content: BlockContent::Simple(nodes![node!("bar"; 8..11)]),
        ..empty_block!(8..11)
      }]
    }
  );
}

#[test]
fn test_parse_section_w_reftext() {
  assert_section!(
    adoc! {r#"
      [reftext=so _baz_]
      == foo

      bar
    "#},
    reftext: Some(nodes![
      node!("so "; 9..12),
      node!(Inline::Italic(just!("baz", 13..16)), 12..17),
    ]),
    Section {
      meta: ChunkMeta {
        attrs: Some(AttrList {
          named: Named::from(vecb![
             (src!("reftext", 1..8), nodes![
               node!("so "; 9..12),
               node!(Inline::Italic(just!("baz", 13..16)), 12..17),
             ])
          ]),
          ..attr_list!(0..18)
        }),
        title: None,
        start: 0
      },
      level: 1,
      id: Some(bstr!("_foo")),
      heading: nodes![node!("foo"; 22..25)],
      blocks: vecb![Block {
        context: BlockContext::Paragraph,
        content: BlockContent::Simple(nodes![node!("bar"; 27..30)]),
        ..empty_block!(27..30)
      }]
    }
  );
}

#[test]
fn test_parse_nested_section() {
  assert_section!(
    adoc! {"
      == one

      === two

      bar
    "},
    Section {
      meta: ChunkMeta::empty(0),
      level: 1,
      id: Some(bstr!("_one")),
      heading: nodes![node!("one"; 3..6)],
      blocks: vecb![Block {
        meta: ChunkMeta::empty(8),
        context: BlockContext::Section,
        content: BlockContent::Section(Section {
          meta: ChunkMeta::empty(8),
          level: 2,
          id: Some(bstr!("_two")),
          heading: nodes![node!("two"; 12..15)],
          blocks: vecb![Block {
            context: BlockContext::Paragraph,
            content: BlockContent::Simple(nodes![node!("bar"; 17..20)]),
            ..empty_block!(17..20)
          }]
        }),
        ..empty_block!(8..21)
      }]
    }
  );
}
