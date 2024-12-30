use asciidork_ast::variants::{inline::*, r#macro::*};
use asciidork_ast::{prelude::*, InlineNodes};
use asciidork_parser::prelude::*;
use test_utils::*;

test_inlines_loose!(
  char_replacements,
  "(C)(TM)(R)...->=><-<=",
  nodes![
    node!(Symbol(SymbolKind::Copyright), 0..3),
    node!(Symbol(SymbolKind::Trademark), 3..7),
    node!(Symbol(SymbolKind::Registered), 7..10),
    node!(Symbol(SymbolKind::Ellipsis), 10..13),
    node!(Symbol(SymbolKind::SingleRightArrow), 13..15),
    node!(Symbol(SymbolKind::DoubleRightArrow), 15..17),
    node!(Symbol(SymbolKind::SingleLeftArrow), 17..19),
    node!(Symbol(SymbolKind::DoubleLeftArrow), 19..21),
  ]
);

test_inlines_loose!(
  escaped_char_replacements,
  "\\(C)\\(TM)\\(R)\\<-",
  nodes![
    node!(Discarded, 0..1),
    node!("(C)"; 1..4),
    node!(Discarded, 4..5),
    node!("(TM)"; 5..9),
    node!(Discarded, 9..10),
    node!("(R)"; 10..13),
    node!(Discarded, 13..14),
    node!("<-"; 14..16),
  ]
);

test_inlines_loose!(
  preserves_entity_refs,
  "&amp; &#169; &#10004; &#128512; &#x2022; &#x1f600;",
  nodes![node!("&amp; &#169; &#10004; &#128512; &#x2022; &#x1f600;"; 0..50)]
);

test_inlines_loose!(
  preserves_entity_refs_without_specialchars,
  "[subs=-specialchars]\n&amp; &#169;",
  nodes![node!("&amp; &#169;"; 21..33)]
);

#[test]
fn test_emdashes() {
  run(vec![
    (
      "foo--bar",
      nodes![
        node!("foo"; 0..3),
        node!(Symbol(SymbolKind::EmDash), 3..5),
        node!("bar"; 5..8),
      ],
    ),
    (
      "富--巴",
      nodes![
        node!("富"; 0..3),
        node!(Symbol(SymbolKind::EmDash), 3..5),
        node!("巴"; 5..8),
      ],
    ),
    (
      "-- foo",
      nodes![
        node!(Symbol(SymbolKind::SpacedEmDash), 0..3),
        node!("foo"; 3..6)
      ],
    ),
    (
      "foo --",
      nodes![
        node!("foo"; 0..3),
        node!(Symbol(SymbolKind::SpacedEmDash), 3..7),
      ],
    ),
    (
      "foo\\--bar foo \\-- bar",
      nodes![
        node!("foo"; 0..3),
        node!(Discarded, 3..4),
        node!("--bar foo "; 4..14),
        node!(Discarded, 14..15),
        node!("-- bar"; 15..21),
      ],
    ),
    (
      "line1\n-- foo",
      nodes![
        node!("line1"; 0..5),
        node!(Inline::Newline, 5..6),
        node!(Symbol(SymbolKind::SpacedEmDash), 6..9),
        node!("foo"; 9..12),
      ],
    ),
    (
      "foo -- bar",
      nodes![
        node!("foo"; 0..3),
        node!(Symbol(SymbolKind::SpacedEmDash), 3..7),
        node!("bar"; 7..10),
      ],
    ),
    ("!--!", nodes![node!("!--!"; 0..4)]),
  ]);
}

#[test]
fn test_button_menu_macro() {
  run(vec![
    (
      "press the btn:[OK] button",
      nodes![
        node!("press the "; 0..10),
        node!(Macro(Button(src!("OK", 15..17))), 10..18),
        node!(" button"; 18..25),
      ],
    ),
    (
      "btn:[Open]",
      nodes![node!(Macro(Button(src!("Open", 5..9))), 0..10)],
    ),
    (
      "select menu:File[Save].",
      nodes![
        node!("select "; 0..7),
        node!(
          Macro(Menu(vecb![src!("File", 12..16), src!("Save", 17..21)])),
          7..22,
        ),
        node!("."; 22..23),
      ],
    ),
    (
      "menu:View[Zoom > Reset]",
      nodes![node!(
        Macro(Menu(vecb![
          src!("View", 5..9),
          src!("Zoom", 10..14),
          src!("Reset", 17..22),
        ])),
        0..23,
      )],
    ),
  ]);
}

// helpers

fn run(cases: Vec<(&str, InlineNodes)>) {
  for (input, expected) in cases {
    expect_eq!(parse_inlines!(input), expected, from: input);
  }
}
