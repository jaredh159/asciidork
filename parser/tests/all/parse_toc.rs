use asciidork_ast::prelude::*;
use asciidork_parser::Parser;
use test_utils::*;

#[test]
fn test_basic_toc() {
  assert_toc!(
    adoc! {"
      :toc:

      == sect 1

      == sect 2
    "},
    TableOfContents {
      title: bstr!("Table of Contents"),
      nodes: vecb![
        TocNode {
          level: 1,
          id: Some(bstr!("_sect_1")),
          title: just!("sect 1", 10..16),
          children: vecb![],
        },
        TocNode {
          level: 1,
          id: Some(bstr!("_sect_2")),
          title: just!("sect 2", 21..27),
          children: vecb![],
        },
      ],
      position: TocPosition::Auto,
    }
  );
}

#[test]
fn test_toc_macro_block() {
  assert_blocks!(
    adoc! {"
      :toc: macro

      foo

      toc::[]
    "},
    &[
      simple_text_block!("foo", 13..16),
      Block {
        context: BlockContext::TableOfContents,
        content: BlockContent::Empty(EmptyMetadata::None),
        ..empty_block!(18)
      }
    ]
  );
}

#[test]
fn test_nested_toc() {
  assert_toc!(
    adoc! {"
      :toc:
      :toc-title: Ham Sandwich

      == sect 1

      === sect 1.1

      == sect 2
    "},
    TableOfContents {
      title: bstr!("Ham Sandwich"),
      nodes: vecb![
        TocNode {
          level: 1,
          id: Some(bstr!("_sect_1")),
          title: just!("sect 1", 35..41),
          children: vecb![TocNode {
            level: 2,
            id: Some(bstr!("_sect_1_1")),
            title: just!("sect 1.1", 47..55),
            children: vecb![],
          }],
        },
        TocNode {
          level: 1,
          id: Some(bstr!("_sect_2")),
          title: just!("sect 2", 60..66),
          children: vecb![],
        },
      ],
      position: TocPosition::Auto,
    }
  );
}

#[test]
fn test_deeply_nested_toc() {
  assert_toc!(
    adoc! {"
      :toc:
      :toclevels: 5

      == sect 1

      === sect 1.1

      ==== sect 1.1.1

      ===== sect 1.1.1.1

      ====== sect 1.1.1.1.1
    "},
    TableOfContents {
      title: bstr!("Table of Contents"),
      nodes: vecb![TocNode {
        level: 1,
        id: Some(bstr!("_sect_1")),
        title: just!("sect 1", 24..30),
        children: vecb![TocNode {
          level: 2,
          id: Some(bstr!("_sect_1_1")),
          title: just!("sect 1.1", 36..44),
          children: vecb![TocNode {
            level: 3,
            id: Some(bstr!("_sect_1_1_1")),
            title: just!("sect 1.1.1", 51..61),
            children: vecb![TocNode {
              level: 4,
              id: Some(bstr!("_sect_1_1_1_1")),
              title: just!("sect 1.1.1.1", 69..81),
              children: vecb![TocNode {
                level: 5,
                id: Some(bstr!("_sect_1_1_1_1_1")),
                title: just!("sect 1.1.1.1.1", 90..104),
                children: vecb![],
              }],
            },],
          },],
        }],
      },],
      position: TocPosition::Auto,
    }
  );
}

#[test]
fn test_toc_depth() {
  assert_toc!(
    adoc! {"
      :toc:
      :toclevels: 2

      == sect 1

      === sect 1.1

      ==== sect 1.1.1

      ===== sect 1.1.1.1

      ====== sect 1.1.1.1.1

      == sect 2
    "},
    TableOfContents {
      title: bstr!("Table of Contents"),
      nodes: vecb![
        TocNode {
          level: 1,
          id: Some(bstr!("_sect_1")),
          title: just!("sect 1", 24..30),
          children: vecb![TocNode {
            level: 2,
            id: Some(bstr!("_sect_1_1")),
            title: just!("sect 1.1", 36..44),
            children: vecb![],
          }],
        },
        TocNode {
          level: 1,
          id: Some(bstr!("_sect_2")),
          title: just!("sect 2", 109..115),
          children: vecb![],
        },
      ],
      position: TocPosition::Auto,
    }
  );
}

assert_error!(
  err_no_preamble,
  adoc! {"
    :toc: preamble

    == sect 1
  "},
  error! {"
    1: :toc: preamble
       ^^^^^^^^^^^^^^ Table of Contents set to `preamble` but no preamble found
  "}
);

assert_error!(
  err_no_macro,
  adoc! {"
    :toc: macro

    == sect 1
  "},
  error! {"
    1: :toc: macro
       ^^^^^^^^^^^ Table of Contents set to `macro` but macro (`toc::[]`) not found
  "}
);

assert_error!(
  err_macro_no_toc,
  adoc! {"
    :!toc:

    == sect 1

    toc::[]
  "},
  error! {"
    5: toc::[]
       ^^^^^^^ Found macro placing Table of Contents, but TOC not enabled
  "}
);
