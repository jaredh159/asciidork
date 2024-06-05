use crate::attrs;
use asciidork_ast::prelude::*;
use asciidork_ast::short::block::*;
use asciidork_ast::{Inline, ListVariant};
use asciidork_parser::Parser;
use test_utils::{assert_eq, *};

#[test]
fn test_simple_unordered_list() {
  assert_list!(
    adoc! {"
      * one
      * two
    "},
    Context::UnorderedList,
    &[
      ListItem {
        marker: ListMarker::Star(1),
        marker_src: src!("*", 0..1),
        principle: just!("one", 2..5),
        ..empty_list_item!()
      },
      ListItem {
        marker: ListMarker::Star(1),
        marker_src: src!("*", 6..7),
        principle: just!("two", 8..11),
        ..empty_list_item!()
      },
    ]
  );
}

#[test]
fn test_simple_nested_list() {
  assert_list!(
    adoc! {"
      * one
      ** two
      * one again
    "},
    Context::UnorderedList,
    &[
      ListItem {
        marker: ListMarker::Star(1),
        marker_src: src!("*", 0..1),
        principle: just!("one", 2..5),
        type_meta: ListItemTypeMeta::None,
        blocks: vecb![Block {
          content: Content::List {
            variant: ListVariant::Unordered,
            depth: 2,
            items: vecb![ListItem {
              marker: ListMarker::Star(2),
              marker_src: src!("**", 6..8),
              principle: just!("two", 9..12),
              ..empty_list_item!()
            }],
          },
          context: Context::UnorderedList,
          ..empty_block!(6..12)
        }],
      },
      ListItem {
        marker: ListMarker::Star(1),
        marker_src: src!("*", 13..14),
        principle: just!("one again", 15..24),
        ..empty_list_item!()
      },
    ]
  );
}

#[test]
fn test_nested_list_separated_by_newlines() {
  assert_list!(
    adoc! {"
      * one


      ** two
    "},
    Context::UnorderedList,
    &[ListItem {
      marker: ListMarker::Star(1),
      marker_src: src!("*", 0..1),
      principle: just!("one", 2..5),
      type_meta: ListItemTypeMeta::None,
      blocks: vecb![Block {
        content: Content::List {
          variant: ListVariant::Unordered,
          depth: 2,
          items: vecb![ListItem {
            marker: ListMarker::Star(2),
            marker_src: src!("**", 8..10),
            principle: just!("two", 11..14),
            ..empty_list_item!()
          }],
        },
        context: Context::UnorderedList,
        ..empty_block!(8..14)
      }],
    },]
  );
}

#[test]
fn test_indented_principle_continuation() {
  assert_list!(
    adoc! {"
      * foo bar
        baz
    "},
    Context::UnorderedList,
    &[ListItem {
      marker: ListMarker::Star(1),
      marker_src: src!("*", 0..1),
      principle: nodes![
        node!("foo bar"; 2..9),
        node!(Inline::Newline, 9..10),
        node!("baz"; 12..15),
      ],
      ..empty_list_item!()
    }]
  );
}

#[test]
fn test_list_custom_marker() {
  assert_list!(
    adoc! {"
      * foo
      [circles]
      ** bar
    "},
    Context::UnorderedList,
    &[ListItem {
      marker: ListMarker::Star(1),
      marker_src: src!("*", 0..1),
      principle: just!("foo", 2..5),
      type_meta: ListItemTypeMeta::None,
      blocks: vecb![Block {
        meta: ChunkMeta::new(Some(attrs::pos("circles", 7..14)), None, 6),
        content: Content::List {
          depth: 2,
          variant: ListVariant::Unordered,
          items: vecb![ListItem {
            marker: ListMarker::Star(2),
            marker_src: src!("**", 16..18),
            principle: just!("bar", 19..22),
            ..empty_list_item!()
          }],
        },
        context: Context::UnorderedList,
        ..empty_block!(6..22)
      }],
    }]
  );
}

#[test]
fn test_checkbox_list() {
  assert_list!(
    adoc! {"
      * [*] checked
      * [x] also checked
      * [ ] not checked
    "},
    Context::UnorderedList,
    &[
      ListItem {
        marker: ListMarker::Star(1),
        marker_src: src!("*", 0..1),
        type_meta: ListItemTypeMeta::Checklist(true, src!("[*]", 2..5)),
        principle: just!(" checked", 5..13),
        ..empty_list_item!()
      },
      ListItem {
        marker: ListMarker::Star(1),
        marker_src: src!("*", 14..15),
        type_meta: ListItemTypeMeta::Checklist(true, src!("[x]", 16..19)),
        principle: just!(" also checked", 19..32),
        ..empty_list_item!()
      },
      ListItem {
        marker: ListMarker::Star(1),
        marker_src: src!("*", 33..34),
        type_meta: ListItemTypeMeta::Checklist(false, src!("[ ]", 35..38)),
        principle: just!(" not checked", 38..50),
        ..empty_list_item!()
      },
    ]
  );
}

#[test]
fn test_single_continuation() {
  assert_list!(
    adoc! {"
      * principle
      +
      with continuation
    "},
    Context::UnorderedList,
    &[ListItem {
      marker: ListMarker::Star(1),
      marker_src: src!("*", 0..1),
      type_meta: ListItemTypeMeta::None,
      principle: just!("principle", 2..11),
      blocks: vecb![Block {
        content: BlockContent::Simple(just!("with continuation", 14..31)),
        context: BlockContext::Paragraph,
        ..empty_block!(14..31)
      }],
    }]
  );
}

#[test]
fn test_double_continuation() {
  assert_list!(
    adoc! {"
      * principle
      +
      with continuation
      +
      and another
    "},
    Context::UnorderedList,
    &[ListItem {
      marker: ListMarker::Star(1),
      marker_src: src!("*", 0..1),
      type_meta: ListItemTypeMeta::None,
      principle: just!("principle", 2..11),
      blocks: vecb![
        Block {
          content: BlockContent::Simple(just!("with continuation", 14..31)),
          context: BlockContext::Paragraph,
          ..empty_block!(14..31)
        },
        Block {
          content: BlockContent::Simple(just!("and another", 34..45)),
          context: BlockContext::Paragraph,
          ..empty_block!(34..45)
        },
      ],
    }]
  );
}

#[test]
fn test_two_items_w_listing_continuations() {
  assert_list!(
    adoc! {"
      * principle
      +
      ----
      listing 1
      ----
      +
      some more principle

      * second principle
      +
      ----
      listing 2
      ----
    "},
    Context::UnorderedList,
    &[
      ListItem {
        marker: ListMarker::Star(1),
        marker_src: src!("*", 0..1),
        type_meta: ListItemTypeMeta::None,
        principle: just!("principle", 2..11),
        blocks: vecb![
          Block {
            content: BlockContent::Simple(just!("listing 1", 19..28)),
            context: BlockContext::Listing,
            ..empty_block!(14..33)
          },
          Block {
            content: BlockContent::Simple(just!("some more principle", 36..55)),
            context: BlockContext::Paragraph,
            ..empty_block!(36..55)
          },
        ],
      },
      ListItem {
        marker: ListMarker::Star(1),
        marker_src: src!("*", 57..58),
        type_meta: ListItemTypeMeta::None,
        principle: just!("second principle", 59..75),
        blocks: vecb![Block {
          content: BlockContent::Simple(just!("listing 2", 83..92)),
          context: BlockContext::Listing,
          ..empty_block!(78..97)
        }],
      },
    ]
  );
}

#[test]
fn test_incomplete_list_item() {
  assert_block_core!("*", Context::Paragraph, Content::Simple(just!("*", 0..1)));
  assert_block_core!("* ", Context::Paragraph, Content::Simple(just!("* ", 0..2)));
}
