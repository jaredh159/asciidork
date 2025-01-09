use crate::attrs;
use asciidork_ast::short::block::*;
use asciidork_ast::{prelude::*, Inline::Newline};
use asciidork_parser::prelude::*;
use test_utils::*;

#[test]
fn test_basic_thematic_break() {
  assert_block!(
    "'''",
    Block {
      context: Context::ThematicBreak,
      content: Content::Empty(EmptyMetadata::None),
      ..empty_block!(0)
    }
  );
}

#[test]
fn test_themmatic_break_w_attrs() {
  assert_block!(
    adoc! {"
      [.fancy]
      '''
    "},
    Block {
      meta: ChunkMeta::new(vecb![attrs::role("fancy", 2..7)], None, 0),
      context: Context::ThematicBreak,
      content: Content::Empty(EmptyMetadata::None),
    }
  );
}

#[test]
fn test_break_in_flow_of_blocks() {
  assert_blocks!(
    adoc! {"
      Hello

      '''

      World
    "},
    &[
      simple_text_block!("Hello", 0..5),
      Block {
        context: Context::ThematicBreak,
        content: Content::Empty(EmptyMetadata::None),
        ..empty_block!(7)
      },
      simple_text_block!("World", 12..17),
    ]
  );
}

#[test]
fn test_break_not_followed_by_empty_line() {
  assert_blocks!(
    adoc! {"
      Hello

      '''
      World
    "},
    &[
      simple_text_block!("Hello", 0..5),
      Block {
        context: Context::ThematicBreak,
        content: Content::Empty(EmptyMetadata::None),
        ..empty_block!(7)
      },
      simple_text_block!("World", 11..16),
    ]
  );
}

#[test]
fn test_invalid_break_not_preceded_by_empty_line() {
  assert_blocks!(
    adoc! {"
      foo
      '''
      bar
    "},
    &[Block {
      context: Context::Paragraph,
      content: Content::Simple(nodes![
        node!("foo"; 0..3),
        node!(Newline, 3..4),
        node!("'''"; 4..7),
        node!(Newline, 7..8),
        node!("bar"; 8..11),
      ]),
      ..empty_block!(0)
    }],
  );
}

#[test]
fn test_page_break() {
  assert_block!(
    "<<<",
    Block {
      context: Context::PageBreak,
      content: Content::Empty(EmptyMetadata::None),
      ..empty_block!(0)
    }
  );
}

#[test]
fn test_page_break_w_attrs() {
  assert_block!(
    adoc! {"
      [%always]
      <<<
    "},
    Block {
      meta: ChunkMeta::new(vecb![attrs::opt("always", 2..8)], None, 0),
      context: Context::PageBreak,
      content: Content::Empty(EmptyMetadata::None),
    }
  );
}
