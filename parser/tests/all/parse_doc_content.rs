use asciidork_ast::{AttrValue, prelude::*};
use asciidork_parser::prelude::*;
use pretty_assertions::assert_eq;
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
    DocContent::Sections(Sectioned {
      preamble: None,
      sections: vecb![Section {
        meta: chunk_meta!(0),
        level: 1,
        id: Some(bstr!("_sect_1")),
        heading: just!("Sect 1", 3..9),
        blocks: vecb![simple_text_block!("foobar", 33..39)],
        loc: (0..39).into()
      }]
    })
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
    DocContent::Sections(Sectioned {
      preamble: Some(vecb![simple_text_block!("Preamble", 0..8)]),
      sections: vecb![Section {
        meta: chunk_meta!(10),
        level: 1,
        id: Some(bstr!("_sect_1")),
        heading: just!("Sect 1", 13..19),
        blocks: vecb![simple_text_block!("Para 1", 21..27)],
        loc: (10..27).into()
      }]
    })
  );
}

#[test]
fn test_no_stack_overflow_for_malformed_book() {
  let input = adoc! {"
    :doctype: book

    == title

    [preface]
  "};
  let _ = test_parser!(input).parse();
}

#[test]
fn comment_only_preamble_discarded() {
  assert_doc_content!(
    adoc! {"
      // first

      == Sect 1

      Para 1
    "},
    DocContent::Sections(Sectioned {
      preamble: None,
      sections: vecb![Section {
        meta: chunk_meta!(10),
        level: 1,
        id: Some(bstr!("_sect_1")),
        heading: just!("Sect 1", 13..19),
        blocks: vecb![simple_text_block!("Para 1", 21..27)],
        loc: (10..27).into()
      }]
    })
  );
}

#[test]
fn test_sectioned_no_preamble() {
  assert_doc_content!(
    adoc! {"
      == Sect 1

      Para 1
    "},
    DocContent::Sections(Sectioned {
      preamble: None,
      sections: vecb![Section {
        meta: chunk_meta!(0),
        level: 1,
        id: Some(bstr!("_sect_1")),
        heading: just!("Sect 1", 3..9),
        blocks: vecb![simple_text_block!("Para 1", 11..17)],
        loc: (0..17).into()
      }]
    })
  );
}

#[test]
fn simple_book_part() {
  assert_doc_content!(
    adoc! {"
     = Book Title
     :doctype: book

     = Part 1

     == Chapter A

     content
    "},
    DocContent::Parts(MultiPartBook {
      preamble: None,
      opening_special_sects: vecb![],
      parts: vecb![Part {
        title: PartTitle {
          id: Some(bstr!("_part_1")),
          meta: chunk_meta!(29),
          text: just!("Part 1", 31..37)
        },
        intro: None,
        sections: vecb![Section {
          meta: chunk_meta!(39),
          level: 1,
          id: Some(bstr!("_chapter_a")),
          heading: just!("Chapter A", 42..51),
          blocks: vecb![simple_text_block!("content", 53..60)],
          loc: (39..60).into()
        }]
      }],
      closing_special_sects: vecb![],
    })
  );
}

assert_error!(
  book_part_with_no_sections_error,
  adoc! {"
    = Book
    :doctype: book

    = Part 1

    [partintro]
    intro
  "},
  error! {"
     --> test.adoc:4:1
      |
    4 | = Part 1
      | ^^^^^^^^ Invalid empty book part, must have at least one section
  "}
);

#[test]
fn test_invalid_subsections() {
  let book_input = adoc! {"
    = Document Title
    :doctype: book

    [preface]
    = Preface

    === Subsection of Preface

    allowed

    [colophon]
    = Colophon

    === Subsection of Colophon

    not allowed

    [dedication]
    = Dedication

    === Subsection of Dedication

    not allowed

    = Part 1

    [abstract]
    == Abstract

    === Subsection of Abstract

    allowed

    == Chapter 1

    === Subsection of Chapter

    allowed

    [appendix]
    = Appendix

    === Subsection of Appendix

    allowed

    [glossary]
    = Glossary

    === Subsection of Glossary

    not allowed

    [bibliography]
    = Bibliography

    === Subsection of Bibliography

    not allowed
  "};
  let warnings = parse_warnings!(book_input)
    .iter()
    .map(|diag| (diag.line.clone(), diag.message.clone()))
    .collect::<Vec<_>>();
  assert_eq!(
    warnings,
    [
      (
        "=== Subsection of Colophon".to_string(),
        "colophon sections do not support nested sections".to_string(),
      ),
      (
        "=== Subsection of Dedication".to_string(),
        "dedication sections do not support nested sections".to_string(),
      ),
      (
        "=== Subsection of Glossary".to_string(),
        "glossary sections do not support nested sections".to_string(),
      ),
      (
        "=== Subsection of Bibliography".to_string(),
        "bibliography sections do not support nested sections".to_string(),
      ),
    ]
  );

  let article_input = adoc! {"
      = Document Title
      :doctype: article

      == Section

      === Subsection of Section

      allowed

      [appendix]
      == Appendix

      === Subsection of Appendix

      allowed

      [glossary]
      == Glossary

      === Subsection of Glossary

      not allowed

      [bibliography]
      == Bibliography

      === Subsection of Bibliography

      not allowed
  "};
  let warnings = parse_warnings!(article_input)
    .iter()
    .map(|diag| (diag.line.clone(), diag.message.clone()))
    .collect::<Vec<_>>();
  assert_eq!(
    warnings,
    [
      (
        "=== Subsection of Glossary".to_string(),
        "glossary sections do not support nested sections".to_string(),
      ),
      (
        "=== Subsection of Bibliography".to_string(),
        "bibliography sections do not support nested sections".to_string(),
      ),
    ]
  )
}

#[test]
fn test_section_offset() {
  assert_doc_content!(
    adoc! {"
      == Sect 1

      :leveloffset: 1

      = Sect 2
    "},
    DocContent::Sections(Sectioned {
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
    })
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

assert_error!(
  book_chapter_out_of_sequence,
  adoc! {"
    = Document Title
    :doctype: book

    === Not a Chapter

    content
  "},
  error! {"
     --> test.adoc:4:1
      |
    4 | === Not a Chapter
      | ^^^ Section title out of sequence: expected level 1 `==`
  "}
);
