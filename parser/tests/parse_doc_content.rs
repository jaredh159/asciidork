use asciidork_ast::prelude::*;
// use asciidork_ast::short::block::*;
use asciidork_parser::Parser;
use test_utils::{assert_eq, *};

#[test]
fn test_not_sectioned() {
  assert_doc_content!(
    adoc! {"
      Para 1

      Para 2
    "},
    DocContent::Blocks(vecb![
      simple_text_block("Para 1", 0..6),
      simple_text_block("Para 2", 8..14),
    ])
  )
}

#[test]
fn test_sectioned_w_preamble() {
  assert_doc_content!(
    adoc! {"
      Preamble

      == Sect 1

      Para 1
    "},
    DocContent::Sectioned {
      preamble: Some(vecb![simple_text_block("Preamble", 0..8)]),
      sections: vecb![Section {
        meta: ChunkMeta::empty(10),
        level: 1,
        heading: just("Sect 1", 13..19),
        blocks: vecb![simple_text_block("Para 1", 21..27)]
      }]
    }
  )
}

#[test]
fn test_sectioned_no_preamble() {
  assert_doc_content!(
    adoc! {"
      == Sect 1

      Para 1
    "},
    DocContent::Sectioned {
      preamble: None,
      sections: vecb![Section {
        meta: ChunkMeta::empty(0),
        level: 1,
        heading: just("Sect 1", 3..9),
        blocks: vecb![simple_text_block("Para 1", 11..17)]
      }]
    }
  )
}

test_error!(
  section_title_out_of_sequence,
  adoc! {"
    == ch 1

    ==== ch 2
  "},
  error! {"
    3: ==== ch 2
       ^^^^ Section title out of sequence: expected level 2 `===`
  "}
);
