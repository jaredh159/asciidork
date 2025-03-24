use asciidork_ast::{prelude::*, AttrValue};
use asciidork_parser::prelude::*;
use test_utils::*;

#[test]
fn test_not_sectioned() {
  assert_doc_content!(
    adoc! {"
      Para 1

      Para 2
    "},
    DocContent::Blocks(vecb![
      simple_text_block!("Para 1", 0..6),
      simple_text_block!("Para 2", 8..14),
    ])
  );
}

#[test]
fn unattached_meta_skipped() {
  assert_doc_content!(
    strict: false,
    adoc! {"
      == Sect 1

      [[whoops-custom-id]]

      foobar
    "},
    DocContent::Sectioned {
      preamble: None,
      sections: vecb![Section {
        meta: chunk_meta!(0),
        level: 1,
        id: Some(bstr!("_sect_1")),
        heading: just!("Sect 1", 3..9),
        blocks: vecb![simple_text_block!("foobar", 33..39)],
        loc: (0..39).into()
      }]
    }
  );
}

assert_error!(
  unattached_meta_warning,
  adoc! {"
    == Sect 1

    [[whoops-custom-id]]

    foobar
  "},
  error! {"
     --> test.adoc:3:1
      |
    3 | [[whoops-custom-id]]
      | ^^^^^^^^^^^^^^^^^^^^ Unattached block metadata
  "}
);

assert_error!(
  unattached_meta_warning2,
  adoc! {"
    [[whoops-custom-id]]

    == Sect 1
  "},
  error! {"
     --> test.adoc:1:1
      |
    1 | [[whoops-custom-id]]
      | ^^^^^^^^^^^^^^^^^^^^ Unattached block metadata
  "}
);

assert_error!(
  error_in_asciidoc_cell_has_correct_line,
  adoc! {"
    [#foo]
    == Foo

    [cols=1a]
    |===
    |[[foo,Foo]]* not AsciiDoc

    | AsciiDoc
    |===
  "},
  error! {"
     --> test.adoc:6:3
      |
    6 | [[foo,Foo]]* not AsciiDoc
      |   ^^^ Duplicate anchor id
  "}
);

#[test]
fn test_sectioned_w_preamble() {
  assert_doc_content!(
    adoc! {"
      Preamble

      == Sect 1

      Para 1
    "},
    DocContent::Sectioned {
      preamble: Some(vecb![simple_text_block!("Preamble", 0..8)]),
      sections: vecb![Section {
        meta: chunk_meta!(10),
        level: 1,
        id: Some(bstr!("_sect_1")),
        heading: just!("Sect 1", 13..19),
        blocks: vecb![simple_text_block!("Para 1", 21..27)],
        loc: (10..27).into()
      }]
    }
  );
}

#[test]
fn comment_only_preamble_discarded() {
  assert_doc_content!(
    adoc! {"
      // first

      == Sect 1

      Para 1
    "},
    DocContent::Sectioned {
      preamble: None,
      sections: vecb![Section {
        meta: chunk_meta!(10),
        level: 1,
        id: Some(bstr!("_sect_1")),
        heading: just!("Sect 1", 13..19),
        blocks: vecb![simple_text_block!("Para 1", 21..27)],
        loc: (10..27).into()
      }]
    }
  );
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
        meta: chunk_meta!(0),
        level: 1,
        id: Some(bstr!("_sect_1")),
        heading: just!("Sect 1", 3..9),
        blocks: vecb![simple_text_block!("Para 1", 11..17)],
        loc: (0..17).into()
      }]
    }
  );
}

#[test]
fn test_section_offset() {
  assert_doc_content!(
    adoc! {"
      == Sect 1

      :leveloffset: 1

      = Sect 2
    "},
    DocContent::Sectioned {
      preamble: None,
      sections: vecb![
        Section {
          meta: chunk_meta!(0),
          level: 1,
          id: Some(bstr!("_sect_1")),
          heading: just!("Sect 1", 3..9),
          blocks: vecb![Block {
            context: BlockContext::DocumentAttributeDecl,
            content: BlockContent::DocumentAttribute(
              "leveloffset".to_string(),
              AttrValue::String("1".to_string())
            ),
            ..empty_block!(11, 26)
          }],
          loc: (0..26).into()
        },
        Section {
          meta: chunk_meta!(28),
          level: 1,
          id: Some(bstr!("_sect_2")),
          heading: just!("Sect 2", 30..36),
          blocks: vecb![],
          loc: (28..36).into()
        }
      ]
    }
  );
}

assert_error!(
  section_title_out_of_sequence,
  adoc! {"
    == ch 1

    ==== ch 2
  "},
  error! {"
     --> test.adoc:3:1
      |
    3 | ==== ch 2
      | ^^^^ Section title out of sequence: expected level 2 `===`
  "}
);
