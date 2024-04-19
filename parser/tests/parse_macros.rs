use asciidork_ast::prelude::*;
use asciidork_ast::variants::{inline::*, r#macro::*};
use asciidork_parser::Parser;
use test_utils::{assert_eq, *};

test_inlines!(
  xref_macro_alone,
  "xref:foo[]",
  nodes![node!(
    Macro(Xref {
      id: src!("foo", 5..8),
      target: vecb![]
    }),
    0..10
  )]
);

test_inlines!(
  xref_macro_w_target,
  "xref:foo[bar _baz_]",
  nodes![node!(
    Macro(Xref {
      id: src!("foo", 5..8),
      target: nodes![
        node!("bar "; 9..13),
        node!(Inline::Italic(just!("baz", 14..17)), 13..18)
      ]
    }),
    0..19
  )]
);
test_inlines!(
  xref_macro_empty_target,
  "xref:f-o[ ]",
  nodes![node!(
    Macro(Xref {
      id: src!("f-o", 5..8),
      target: just!(" ", 9..10)
    }),
    0..11
  )]
);

test_inlines!(
  xref_macro_w_surrounding_text,
  "foo xref:bar[] baz",
  nodes![
    node!("foo "; 0..4),
    node!(
      Macro(Xref {
        id: src!("bar", 9..12),
        target: vecb![]
      }),
      4..14
    ),
    node!(" baz"; 14..18)
  ]
);

test_inlines!(
  test_xref_shorthand,
  "<<foo>>",
  nodes![node!(
    Macro(Xref {
      id: src!("foo", 2..5),
      target: vecb![]
    }),
    0..7
  )]
);

test_inlines!(
  xref_extra_lessthan,
  "<<<foo>>",
  nodes![
    node!(Inline::SpecialChar(SpecialCharKind::LessThan), 0..1),
    node!(
      Macro(Xref {
        id: src!("foo", 3..6),
        target: vecb![]
      }),
      1..8
    )
  ]
);

test_inlines!(
  xref_shorthand_w_target,
  "baz <<foo-rofl,so _cool_ wow>> end",
  nodes![
    node!("baz "; 0..4),
    node!(
      Macro(Xref {
        id: src!("foo-rofl", 6..14),
        target: nodes![
          node!("so "; 15..18),
          node!(Inline::Italic(just!("cool", 19..23)), 18..24),
          node!(" wow"; 24..28)
        ]
      }),
      4..30
    ),
    node!(" end"; 30..34)
  ]
);
