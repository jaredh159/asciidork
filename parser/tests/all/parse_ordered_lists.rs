use asciidork_ast::prelude::*;
use asciidork_ast::short::block::*;
use asciidork_ast::ListVariant;
use asciidork_parser::prelude::*;
use test_utils::*;

#[test]
fn test_basic_ordered_list() {
  assert_list!(
    adoc! {"
      . foo

      . bar
    "},
    Context::OrderedList,
    &[
      ListItem {
        marker: ListMarker::Dot(1),
        marker_src: src!(".", 0..1),
        principle: just!("foo", 2..5),
        ..empty_list_item!()
      },
      ListItem {
        marker: ListMarker::Dot(1),
        marker_src: src!(".", 7..8),
        principle: just!("bar", 9..12),
        ..empty_list_item!()
      },
    ]
  );
}

#[test]
fn test_ordered_w_nested_unordered() {
  assert_list!(
    adoc! {"
      . Linux

        * Fedora
        * Ubuntu
    "},
    Context::OrderedList,
    &[ListItem {
      marker: ListMarker::Dot(1),
      marker_src: src!(".", 0..1),
      principle: just!("Linux", 2..7),
      type_meta: ListItemTypeMeta::None,
      blocks: vecb![Block {
        meta: chunk_meta!(9),
        loc: (9..30).into(),
        context: Context::UnorderedList,
        content: Content::List {
          depth: 2,
          variant: ListVariant::Unordered,
          items: vecb![
            ListItem {
              marker: ListMarker::Star(1),
              marker_src: src!("*", 11..12),
              principle: just!("Fedora", 13..19),
              ..empty_list_item!()
            },
            ListItem {
              marker: ListMarker::Star(1),
              marker_src: src!("*", 22..23),
              principle: just!("Ubuntu", 24..30),
              ..empty_list_item!()
            },
          ],
        },
      }],
    }]
  );
}
