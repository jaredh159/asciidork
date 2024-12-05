use crate::attrs;
use asciidork_ast::prelude::*;
use asciidork_ast::variants::{inline::*, r#macro::*};
use asciidork_ast::Flow;
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
      id: src!("foo", 5..8),
      linktext: None
    }),
    0..10
  )]
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
      id: src!("foo", 5..8),
      linktext: Some(nodes![
        node!("bar "; 9..13),
        node!(Inline::Italic(just!("baz", 14..17)), 13..18)
      ])
    }),
    0..19
  )]
);

test_inlines_loose!(
  xref_macro_empty_target,
  "xref:f-o[ ]",
  nodes![node!(
    Macro(Xref {
      id: src!("f-o", 5..8),
      linktext: Some(just!(" ", 9..10))
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
        id: src!("bar", 9..12),
        linktext: None
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
      id: src!("foo", 2..5),
      linktext: None
    }),
    0..7
  )]
);

test_inlines_loose!(
  test_xref_shorthand_explicit_id,
  "<<#foo>>",
  nodes![node!(
    Macro(Xref {
      id: src!("foo", 3..6),
      linktext: None
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
        id: src!("foo", 3..6),
        linktext: None
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
        id: src!("foo-rofl", 6..14),
        linktext: Some(nodes![
          node!("so "; 15..18),
          node!(Inline::Italic(just!("cool", 19..23)), 18..24),
          node!(" wow"; 24..28)
        ])
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
