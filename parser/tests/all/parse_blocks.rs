use crate::attrs;
use asciidork_ast::prelude::*;
use asciidork_ast::short::block::*;
use asciidork_ast::variants::inline::*;
use asciidork_parser::prelude::*;
use test_utils::*;

#[test]
fn test_parse_simple_block() {
  assert_block!(
    adoc! {"
      hello mamma,
      hello papa
    "},
    Block {
      context: Context::Paragraph,
      content: Content::Simple(nodes![
        node!("hello mamma,"; 0..12),
        node!(Newline, 12..13),
        node!("hello papa"; 13..23),
      ]),
      ..empty_block!(0)
    }
  );
}

#[test]
fn block_multiple_attr_lists() {
  assert_block!(
    adoc! {"
      [mustard]
      [%thingy]
      foobar
    "},
    Block {
      context: Context::Paragraph,
      content: Content::Simple(just!("foobar", 20..26)),
      meta: ChunkMeta::new(
        vecb![attrs::pos("mustard", 1..8), attrs::opt("thingy", 12..18)],
        None,
        0
      ),
    }
  );
}

#[test]
fn test_line_followed_by_comment_is_trimmed() {
  assert_block!(
    adoc! {"
      hello mamma
      // a comment
    "},
    simple_text_block!("hello mamma", 0..11)
  );
}

#[test]
fn test_parse_comment_block() {
  assert_block!(
    adoc! {"
      ////
      A comment block
      ////
    "},
    Block {
      context: Context::Comment,
      content: Content::Empty(EmptyMetadata::None),
      ..empty_block!(0)
    }
  );
}

#[test]
fn test_parse_comment_style_block() {
  assert_block!(
    adoc! {"
      [comment]
      --
      A comment block.

      Notice it's a delimited block.
      --
    "},
    Block {
      meta: ChunkMeta::new(vecb![attrs::pos("comment", 1..8)], None, 0),
      context: Context::Comment,
      content: Content::Empty(EmptyMetadata::None),
    }
  );
}

#[test]
fn test_parse_paragraph_comment_block() {
  assert_block!(
    adoc! {"
      [comment]
      A paragraph comment
      Like all paragraphs, the lines must be contiguous.
    "},
    Block {
      meta: ChunkMeta::new(vecb![attrs::pos("comment", 1..8)], None, 0),
      context: Context::Comment,
      content: Content::Empty(EmptyMetadata::None),
    }
  );
}

#[test]
fn test_parse_discrete_headings() {
  assert_blocks!(
    adoc! {"
      [discrete]
      ==== A discrete heading

      :leveloffset: 1

      [float]
      === Another discrete heading
    "},
    &[
      Block {
        meta: ChunkMeta::new(vecb![attrs::pos("discrete", 1..9)], None, 0),
        context: Context::DiscreteHeading,
        content: Content::Empty(EmptyMetadata::DiscreteHeading {
          level: 3,
          content: just!("A discrete heading", 16..34),
          id: Some(bstr!("_a_discrete_heading")),
        }),
      },
      Block {
        content: Content::DocumentAttribute("leveloffset".to_string(), "1".into()),
        context: Context::DocumentAttributeDecl,
        ..empty_block!(36)
      },
      Block {
        //                                    vvvvv - synonym for `discrete`
        meta: ChunkMeta::new(vecb![attrs::pos("float", 54..59)], None, 53),
        context: Context::DiscreteHeading,
        content: Content::Empty(EmptyMetadata::DiscreteHeading {
          level: 3, // <- discrete headings are subject to `leveloffset`
          content: just!("Another discrete heading", 65..89),
          id: Some(bstr!("_another_discrete_heading")),
        }),
      }
    ]
  );
}

#[test]
fn test_multi_blocks() {
  let cases = vec![
    adoc! {"
      foo
      [WARNING]
      bar
    "},
    adoc! {"
      foo
      [[block-anchor]]
      bar
    "},
    // middle line should be trimmed and considered an empty line
    // @see https://docs.asciidoctor.org/asciidoc/latest/normalization
    "foo\n   \nbar",
  ];
  for case in cases {
    assert_eq!(parse_blocks!(case).len(), 2);
  }
}

#[test]
fn test_incomplete_heading_doesnt_panic() {
  assert_block!("== ", simple_text_block!("== ", 0..3));
}

#[test]
fn test_parse_passthrough() {
  assert_block!(
    adoc! {"
      [pass]
      foo <bar>
    "},
    Block {
      meta: ChunkMeta::new(vecb![attrs::pos("pass", 1..5)], None, 0),
      context: Context::Passthrough,
      content: Content::Simple(just!("foo <bar>", 7..16)),
    }
  );
}

#[test]
fn test_parse_delimited_passthrough_block() {
  let input = adoc! {"
    ++++
    foo <bar>
    baz
    ++++
  "};
  let expected = Block {
    context: Context::Passthrough,
    content: Content::Simple(nodes![
      node!("foo <bar>"; 5..14),
      node!(Newline, 14..15),
      node!("baz"; 15..18),
    ]),
    ..empty_block!(0)
  };
  assert_block!(input, expected);
}

#[test]
fn test_parse_delimited_passthrough_block_subs_normal() {
  let input = adoc! {"
    [subs=normal]
    ++++
    foo & _<bar>_
    baz
    ++++
  "};
  let expected = Block {
    meta: ChunkMeta {
      attrs: vecb![attrs::named(&[("subs", 1..5, "normal", 6..12)])].into(),
      start: 0,
      title: None,
    },
    context: Context::Passthrough,
    content: Content::Simple(nodes![
      node!("foo "; 19..23),
      node!(SpecialChar(SpecialCharKind::Ampersand), 23..24),
      node!(" "; 24..25),
      node!(
        Italic(nodes![
          node!(SpecialChar(SpecialCharKind::LessThan), 26..27),
          node!("bar"; 27..30),
          node!(SpecialChar(SpecialCharKind::GreaterThan), 30..31),
        ]),
        25..32,
      ),
      node!(Newline, 32..33),
      node!("baz"; 33..36),
    ]),
  };
  assert_block!(input, expected);
}

#[test]
fn test_parse_block_titles() {
  let input = adoc! {"
    .My Title
    foo
  "};
  let expected = Block {
    meta: ChunkMeta {
      title: Some(just!("My Title", 1..9)),
      ..chunk_meta!(0)
    },
    context: Context::Paragraph,
    content: Content::Simple(nodes![node!("foo"; 10..13)]),
  };
  assert_block!(input, expected);
}

#[test]
fn test_parse_admonitions() {
  assert_block!(
    adoc! {"
      TIP: foo
    "},
    Block {
      context: Context::AdmonitionTip,
      content: Content::Simple(nodes![node!("foo"; 5..8)]),
      ..empty_block!(0)
    }
  );

  assert_block!(
    adoc! {"
      [pos]
      TIP: foo
    "},
    Block {
      meta: ChunkMeta::new(vecb![attrs::pos("pos", 1..4)], None, 0),
      context: Context::AdmonitionTip,
      content: Content::Simple(just!("foo", 11..14)),
    }
  );

  assert_block!(
    adoc! {"
      [WARNING]
      TIP: foo
    "},
    Block {
      meta: ChunkMeta::new(vecb![attrs::pos("WARNING", 1..8)], None, 0),
      context: Context::AdmonitionWarning,
      content: Content::Simple(just!("TIP: foo", 10..18)), // <-- attr list wins
    }
  );

  assert_block!(
    adoc! {"
      [WARNING]
      ====
      foo
      ====
    "},
    Block {
      meta: ChunkMeta::new(vecb![attrs::pos("WARNING", 1..8)], None, 0),
      context: Context::AdmonitionWarning, // <-- turns example into warning
      content: Content::Compound(vecb![Block {
        context: Context::Paragraph,
        content: Content::Simple(just!("foo", 15..18)),
        ..empty_block!(15)
      }]),
    }
  );

  assert_block!(
    adoc! {"
      [CAUTION]
      ====
      NOTE: foo
      ====
    "},
    Block {
      meta: ChunkMeta::new(vecb![attrs::pos("CAUTION", 1..8)], None, 0),
      context: Context::AdmonitionCaution,
      content: Content::Compound(vecb![Block {
        context: Context::AdmonitionNote,
        content: Content::Simple(just!("foo", 21..24)),
        ..empty_block!(15)
      }]),
    }
  );
}

#[test]
fn test_parse_comment_line_block() {
  assert_block!(
    "//-",
    Block {
      context: Context::Comment,
      content: Content::Empty(EmptyMetadata::None),
      ..empty_block!(0)
    }
  );
  assert_block!(
    "//key:: val", // <-- looks like desc list
    Block {
      context: Context::Comment,
      content: Content::Empty(EmptyMetadata::None),
      ..empty_block!(0)
    }
  );
}

#[test]
fn test_parse_image_block() {
  assert_block!(
    "image::name.png[]\n\n",
    Block {
      context: Context::Image,
      content: Content::Empty(EmptyMetadata::Image {
        target: src!("name.png", 7..15),
        attrs: attr_list!(15..17),
      }),
      ..empty_block!(0)
    }
  );
}

#[test]
fn test_parse_delimited_open_block() {
  assert_block!(
    adoc! {"
      --
      foo
      --
    "},
    Block {
      context: Context::Open,
      content: Content::Compound(vecb![Block {
        context: Context::Paragraph,
        content: Content::Simple(just!("foo", 3..6)),
        ..empty_block!(3)
      }]),
      ..empty_block!(0)
    }
  );
}

#[test]
fn test_parse_delimited_example_block() {
  assert_block!(
    adoc! {"
      ====
      foo
      ====
    "},
    Block {
      context: Context::Example,
      content: Content::Compound(vecb![Block {
        context: Context::Paragraph,
        content: Content::Simple(just!("foo", 5..8)),
        ..empty_block!(5)
      }]),
      ..empty_block!(0)
    },
  );
}

#[test]
fn test_undelimited_sidebar() {
  assert_block!(
    adoc! {"
      [sidebar]
      foo
    "},
    Block {
      meta: ChunkMeta::new(vecb![attrs::pos("sidebar", 1..8)], None, 0),
      context: Context::Sidebar,
      content: Content::Simple(just!("foo", 10..13)),
    }
  );
}

#[test]
fn test_parse_empty_delimited_block() {
  assert_block!(
    adoc! {"
      --
      --
    "},
    Block {
      context: Context::Open,
      content: Content::Compound(vecb![]),
      ..empty_block!(0)
    }
  );
}

#[test]
fn test_parse_delimited_sidebar_block() {
  assert_block!(
    adoc! {"
      ****
      foo
      ****
    "},
    Block {
      context: Context::Sidebar,
      content: Content::Compound(vecb![Block {
        context: Context::Paragraph,
        content: Content::Simple(just!("foo", 5..8)),
        ..empty_block!(5)
      }]),
      ..empty_block!(0)
    },
  );
}

#[test]
fn test_nested_delimiter_blocks() {
  assert_block!(
    adoc! {"
      ****
      --
      foo
      --
      ****
    "},
    Block {
      context: Context::Sidebar,
      content: Content::Compound(vecb![Block {
        context: Context::Open,
        content: Content::Compound(vecb![Block {
          context: Context::Paragraph,
          content: Content::Simple(just!("foo", 8..11)),
          ..empty_block!(8)
        }]),
        ..empty_block!(5)
      }]),
      ..empty_block!(0)
    }
  );

  assert_block!(
    adoc! {"
      ****

      .Bar
      --

      foo


      --

      ****
    "},
    Block {
      context: Context::Sidebar,
      content: Content::Compound(vecb![Block {
        meta: ChunkMeta {
          title: Some(just!("Bar", 7..10)),
          ..chunk_meta!(6)
        },
        context: Context::Open,
        content: Content::Compound(vecb![Block {
          context: Context::Paragraph,
          content: Content::Simple(just!("foo", 15..18)),
          ..empty_block!(15)
        }]),
      }]),
      ..empty_block!(0)
    }
  );
}

#[test]
fn test_parse_multi_para_delimited_sidebar_block() {
  assert_block!(
    adoc! {"
      ****
      This is content in a sidebar block.

      image::name.png[]

      This is more content in the sidebar block.
      ****
    "},
    Block {
      context: Context::Sidebar,
      content: Content::Compound(vecb![
        Block {
          context: Context::Paragraph,
          content: Content::Simple(just!("This is content in a sidebar block.", 5..40)),
          ..empty_block!(5)
        },
        Block {
          context: Context::Image,
          content: Content::Empty(EmptyMetadata::Image {
            target: src!("name.png", 49..57),
            attrs: attr_list!(57..59),
          }),
          ..empty_block!(42)
        },
        Block {
          context: Context::Paragraph,
          content: Content::Simple(just!("This is more content in the sidebar block.", 61..103)),
          ..empty_block!(61)
        },
      ]),
      ..empty_block!(0)
    }
  );
}

assert_error!(
  unclosed_delimited_block_err,
  adoc! {"
    --
    foo
  "},
  error! {"
     --> test.adoc:1:1
      |
    1 | --
      | ^^ This delimiter was never closed
  "}
);
