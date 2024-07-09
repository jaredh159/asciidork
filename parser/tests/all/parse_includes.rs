use asciidork_ast::{prelude::*, IncludeBoundaryKind as Boundary};
use asciidork_parser::Parser;
use test_utils::{assert_eq, *};

#[test]
fn basic_include() {
  assert_doc_content!(
    resolving: b"Line-2!",
    adoc! {"
      Line-1
      include::some_file.adoc[]
    "},
    DocContent::Blocks(vecb![Block {
        context: BlockContext::Paragraph,
        content: BlockContent::Simple(nodes![
          node!("Line-1"; 0..6),
          node!(Inline::Newline, 6..7),
          node!(Inline::IncludeBoundary(Boundary::Begin, 1), 7..32),
          node!("Line-2!"; 0..7, depth: 1),
          node!(Inline::IncludeBoundary(Boundary::End, 1), 7..32),
        ]),
        ..empty_block!(0)
      }
    ])
  )
}

// https://github.com/opendevise/asciidoc-parsing-lab/blob/main/test/tests/block/include/trailing-include-output.json
// https://github.com/opendevise/asciidoc-parsing-lab/blob/main/test/tests/block/include/trailing-newline-after-include-output.json

// docattr target: `include::{foo}[]`
// attrs, encodings, etc
// include on last line of para
// consecutive includes to define attrs
// error messages
