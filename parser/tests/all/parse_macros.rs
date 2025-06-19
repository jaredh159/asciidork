use crate::attrs;
use asciidork_ast::prelude::*;
use asciidork_ast::variants::{inline::*, r#macro::*};
use asciidork_ast::{Flow, PluginMacro};
use asciidork_parser::prelude::*;
use test_utils::*;

test_inlines_loose!(
  link_macro,
  "http://foo.com[bar]",
  nodes![node!(
    Macro(Link {
      scheme: Some(UrlScheme::Http),
      target: src!("http://foo.com", 0..14),
      attrs: Some(attrs::pos("bar", 15..18)),
      caret: false,
    }),
    0..19
  )]
);

test_inlines_loose!(
  link_macro_w_formatted_text,
  "http://foo.com[[.role]#bar#]",
  nodes![node!(
    Macro(Link {
      scheme: Some(UrlScheme::Http),
      target: src!("http://foo.com", 0..14),
      attrs: Some(AttrList {
        positional: vecb![Some(nodes![node!(
          TextSpan(
            AttrList {
              roles: vecb![src!("role", 17..21)],
              ..attr_list!(15..22)
            },
            just!("bar", 23..26)
          ),
          15..27
        )])],
        ..attr_list!(14..28)
      }),
      caret: false,
    }),
    0..28
  )]
);

test_inlines_loose!(
  xref_macro_alone,
  "xref:foo[]",
  nodes![node!(
    Macro(Xref {
      target: src!("foo", 5..8),
      linktext: None,
      kind: XrefKind::Macro
    }),
    0..10
  )]
);

test_inlines_loose!(
  xref_macro_target_w_colon,
  "xref::/c[] foo xref::/d[]",
  nodes![
    node!(
      Macro(Xref {
        target: src!(":/c", 5..8),
        linktext: None,
        kind: XrefKind::Macro
      }),
      0..10
    ),
    node!(" foo "; 10..15),
    node!(
      Macro(Xref {
        target: src!(":/d", 20..23),
        linktext: None,
        kind: XrefKind::Macro
      }),
      15..25
    )
  ]
);

test_inlines_loose!(
  inline_image_macro,
  "image:play.png[]",
  nodes![node!(
    Macro(Image {
      flow: Flow::Inline,
      target: src!("play.png", 6..14),
      attrs: attr_list!(14..16)
    }),
    0..16
  )]
);

test_inlines_loose!(
  inline_image_macro_attr,
  "image:{note-caption}.png[]",
  nodes![node!(
    Macro(Image {
      flow: Flow::Inline,
      target: src!("Note.png", 6..24),
      attrs: attr_list!(24..26)
    }),
    0..26
  )]
);

test_inlines_loose!(
  inline_image_macro_w_space_target,
  "image:p ay.png[]",
  nodes![node!(
    Macro(Image {
      flow: Flow::Inline,
      target: src!("p ay.png", 6..14),
      attrs: attr_list!(14..16)
    }),
    0..16
  )]
);

test_inlines_loose!(
  inline_anchor_macro,
  "anchor:some-id[]",
  nodes![node!(InlineAnchor(bstr!("some-id")), 7..14)]
);

test_inlines_loose!(
  inline_anchor_macro_reftext,
  "anchor:some-id[Some ref text here]",
  nodes![node!(InlineAnchor(bstr!("some-id")), 7..14)]
);

test_inlines_loose!(
  xref_macro_w_target,
  "xref:foo[bar _baz_]",
  nodes![node!(
    Macro(Xref {
      target: src!("foo", 5..8),
      linktext: Some(nodes![
        node!("bar "; 9..13),
        node!(Inline::Italic(just!("baz", 14..17)), 13..18)
      ]),
      kind: XrefKind::Macro
    }),
    0..19
  )]
);

test_inlines_loose!(
  xref_macro_empty_target,
  "xref:f-o[ ]",
  nodes![node!(
    Macro(Xref {
      target: src!("f-o", 5..8),
      linktext: Some(just!(" ", 9..10)),
      kind: XrefKind::Macro
    }),
    0..11
  )]
);

test_inlines_loose!(
  xref_macro_w_surrounding_text,
  "foo xref:bar[] baz",
  nodes![
    node!("foo "; 0..4),
    node!(
      Macro(Xref {
        target: src!("bar", 9..12),
        linktext: None,
        kind: XrefKind::Macro
      }),
      4..14
    ),
    node!(" baz"; 14..18)
  ]
);

test_inlines_loose!(
  test_xref_shorthand,
  "<<foo>>",
  nodes![node!(
    Macro(Xref {
      target: src!("foo", 2..5),
      linktext: None,
      kind: XrefKind::Shorthand
    }),
    0..7
  )]
);

test_inlines_loose!(
  test_xref_shorthand_explicit_id,
  "<<#foo>>",
  nodes![node!(
    Macro(Xref {
      target: src!("#foo", 2..6),
      linktext: None,
      kind: XrefKind::Shorthand
    }),
    0..8
  )]
);

test_inlines_loose!(
  xref_extra_lessthan,
  "<<<foo>>",
  nodes![
    node!(Inline::SpecialChar(SpecialCharKind::LessThan), 0..1),
    node!(
      Macro(Xref {
        target: src!("foo", 3..6),
        linktext: None,
        kind: XrefKind::Shorthand
      }),
      1..8
    )
  ]
);

test_inlines_loose!(
  xref_shorthand_w_target,
  "baz <<foo-rofl,so _cool_ wow>> end",
  nodes![
    node!("baz "; 0..4),
    node!(
      Macro(Xref {
        target: src!("foo-rofl", 6..14),
        linktext: Some(nodes![
          node!("so "; 15..18),
          node!(Inline::Italic(just!("cool", 19..23)), 18..24),
          node!(" wow"; 24..28)
        ]),
        kind: XrefKind::Shorthand
      }),
      4..30
    ),
    node!(" end"; 30..34)
  ]
);

assert_error!(
  xref_unknown_anchor,
  "<<foo>>",
  error! {r"
     --> test.adoc:1:3
      |
    1 | <<foo>>
      |   ^^^ Invalid cross reference, no anchor found for `foo`
  "}
);

assert_error!(
  xref_unknown_interdoc_anchor,
  adoc! {"
    [#foobar]
    == Foobar

    See <<test.adoc#foobaz>>.
  "},
  error! {"
     --> test.adoc:4:7
      |
    4 | See <<test.adoc#foobaz>>.
      |       ^^^^^^^^^^^^^^^^ Invalid cross reference, no anchor found for `test.adoc#foobaz`
  "}
);

macro_rules! plugin_macro_test {
  ($input:expr, $macro_name:expr) => {{
    let mut parser = test_parser!($input);
    parser.register_plugin_macros(&[$macro_name]);
    let document = parser.parse().unwrap().document;
    let mut blocks = document.content.blocks().unwrap().clone();
    assert_eq!(blocks.len(), 1);
    blocks.pop().unwrap()
  }};
}

#[test]
fn plugin_inline_macro() {
  let parsed = plugin_macro_test!("bob:[mustard]\n", "bob");
  expect_eq!(
    parsed,
    Block {
      meta: chunk_meta!(0),
      context: BlockContext::Paragraph,
      content: BlockContent::Simple(nodes![node!(
        Macro(Plugin(PluginMacro {
          name: bstr!("bob"),
          target: None,
          flow: Flow::Inline,
          attrs: attrs::pos("mustard", 5..12),
          source: src!("bob:[mustard]", 0..13)
        })),
        0..13
      )]),
      loc: (0..13).into(),
    }
  );
}

#[test]
fn plugin_block_macro() {
  let parsed = plugin_macro_test!("bob::[mustard]\n", "bob");
  expect_eq!(
    parsed,
    Block {
      meta: chunk_meta!(0),
      context: BlockContext::Paragraph,
      content: BlockContent::Simple(nodes![node!(
        Macro(Plugin(PluginMacro {
          name: bstr!("bob"),
          target: None,
          flow: Flow::Block,
          attrs: attrs::pos("mustard", 6..13),
          source: src!("bob::[mustard]", 0..14)
        })),
        0..14
      )]),
      loc: (0..14).into(),
    }
  );

  let parsed = plugin_macro_test!("bob::baz[]\n", "bob");
  expect_eq!(
    parsed,
    Block {
      meta: chunk_meta!(0),
      context: BlockContext::Paragraph,
      content: BlockContent::Simple(nodes![node!(
        Macro(Plugin(PluginMacro {
          name: bstr!("bob"),
          target: Some(src!("baz", 5..8)),
          flow: Flow::Block,
          attrs: attr_list!(8..10),
          source: src!("bob::baz[]", 0..10)
        })),
        0..10
      )]),
      loc: (0..10).into(),
    }
  );
}
