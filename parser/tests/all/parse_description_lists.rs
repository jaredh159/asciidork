use asciidork_ast::prelude::*;
use asciidork_ast::short::block::*;
use asciidork_parser::prelude::*;
use test_utils::*;

#[test]
fn test_simple_description_list() {
  assert_list!(
    "foo:: bar",
    Context::DescriptionList,
    &[ListItem {
      marker: ListMarker::Colons(2),
      marker_src: src!("::", 3..5),
      principle: just!("foo", 0..3),
      type_meta: ListItemTypeMeta::DescList {
        description: Some(Block {
          content: Content::Simple(just!("bar", 6..9)),
          context: Context::Paragraph,
          ..empty_block!(6)
        }),
        extra_terms: vecb![]
      },
      blocks: vecb![],
    }]
  );
}

#[test]
fn test_two_line_description_list() {
  assert_list!(
    adoc! {"
      foo::
      bar
    "},
    Context::DescriptionList,
    &[ListItem {
      marker: ListMarker::Colons(2),
      marker_src: src!("::", 3..5),
      principle: just!("foo", 0..3),
      type_meta: ListItemTypeMeta::DescList {
        description: Some(Block {
          content: Content::Simple(just!("bar", 6..9)),
          context: Context::Paragraph,
          ..empty_block!(6)
        }),
        extra_terms: vecb![]
      },
      blocks: vecb![],
    }]
  );
}
