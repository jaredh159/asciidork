use asciidork_ast::{prelude::*, IncludeBoundaryKind as Boundary};
use asciidork_parser::Parser;
use test_utils::*;

#[test]
fn include_no_trailing_newline() {
  assert_doc_content!(
    resolving: b"Line-2!",
    adoc! {"
      Line-1
      include::some_file.adoc[]
    "},
    DocContent::Blocks(vecb![
      Block {
        context: BlockContext::Paragraph,
        content: BlockContent::Simple(nodes![
          node!("Line-1"; 0..6),
          node!(Inline::Newline, 6..7),
          node!(Inline::IncludeBoundary(Boundary::Begin, 1), 7..32),
          node!("Line-2!"; 0..7, depth: 1),
          // node!(Inline::Newline, 7..8, depth: 1),
          node!(Inline::IncludeBoundary(Boundary::End, 1), 7..32),
        ]),
        ..empty_block!(0)
      }
    ])
  );
  assert_doc_content!(
    resolving: b"Line-2!",
    adoc! {"
      Line-1
      include::some_file.adoc[]
      Line-3
    "},
    DocContent::Blocks(vecb![
      Block {
        context: BlockContext::Paragraph,
        content: BlockContent::Simple(nodes![
          node!("Line-1"; 0..6),
          node!(Inline::Newline, 6..7),
          node!(Inline::IncludeBoundary(Boundary::Begin, 1), 7..32),
          node!("Line-2!"; 0..7, depth: 1),
          node!(Inline::Newline, 7..8, depth: 1),
          node!(Inline::IncludeBoundary(Boundary::End, 1), 7..32),
          node!("Line-3"; 33..39),
        ]),
        ..empty_block!(0)
      }
    ])
  );
}

#[test]
fn include_with_trailing_newline() {
  assert_doc_content!(
    resolving: b"Line-2!\n",
    adoc! {"
      Line-1
      include::some_file.adoc[]
    "},
    DocContent::Blocks(vecb![
      Block {
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
  );
  assert_doc_content!(
    resolving: b"Line-2!\n",
    adoc! {"
      Line-1
      include::some_file.adoc[]
      Line-3
    "},
    DocContent::Blocks(vecb![
      Block {
        context: BlockContext::Paragraph,
        content: BlockContent::Simple(nodes![
          node!("Line-1"; 0..6),
          node!(Inline::Newline, 6..7),
          node!(Inline::IncludeBoundary(Boundary::Begin, 1), 7..32),
          node!("Line-2!"; 0..7, depth: 1),
          // node!(Inline::Newline, 7..8, depth: 1),
          node!(Inline::Newline, 7..8, depth: 1),
          node!(Inline::IncludeBoundary(Boundary::End, 1), 7..32),
          node!("Line-3"; 33..39),
        ]),
        ..empty_block!(0)
      }
    ])
  );
  let input = adoc! {"
    Line-1
    include::some_file.adoc[]
    Line-3
  "};
  assert_eq!("Line-3", &input[33..39]);
}

#[test]
fn include_with_2_trailing_newlines() {
  assert_doc_content!(
    resolving: b"Line-2!\n\n",
    adoc! {"
      Line-1
      include::some_file.adoc[]
      Line-3
    "},
    DocContent::Blocks(vecb![
      Block {
        context: BlockContext::Paragraph,
        content: BlockContent::Simple(nodes![
          node!("Line-1"; 0..6),
          node!(Inline::Newline, 6..7),
          node!(Inline::IncludeBoundary(Boundary::Begin, 1), 7..32),
          node!("Line-2!"; 0..7, depth: 1),
        ]),
        ..empty_block!(0)
      },
      Block {
        context: BlockContext::Paragraph,
        content: BlockContent::Simple(nodes![
          node!(Inline::IncludeBoundary(Boundary::End, 1), 7..32),
          node!("Line-3"; 33..39),
        ]),
        ..empty_block!(7)
      }
    ])
  );
}

// https://github.com/opendevise/asciidoc-parsing-lab/blob/main/test/tests/block/include/trailing-include-output.json
// https://github.com/opendevise/asciidoc-parsing-lab/blob/main/test/tests/block/include/trailing-newline-after-include-output.json

// docattr target: `include::{foo}[]`
// attrs, encodings, etc
// include on last line of para
// consecutive includes to define attrs
// error messages
