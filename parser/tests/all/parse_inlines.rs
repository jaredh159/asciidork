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

#[test]
fn test_inline_passthrus() {
  run(vec![
    (
      "+_foo_&+ bar",
      nodes![
        node!(
          InlinePassthru(nodes![
            node!("_foo_"; 1..6),
            node!(SpecialChar(SpecialCharKind::Ampersand), 6..7),
          ]),
          0..8,
        ),
        node!(" bar"; 8..12),
      ],
    ),
    (
      "baz ++_foo_&++ bar",
      nodes![
        node!("baz "; 0..4),
        node!(
          InlinePassthru(nodes![
            node!("_foo_"; 6..11),
            node!(SpecialChar(SpecialCharKind::Ampersand), 11..12),
          ]),
          4..14,
        ),
        node!(" bar"; 14..18),
      ],
    ),
    (
      "baz +++_foo_&+++ bar", // no specialchars subs on +++
      nodes![
        node!("baz "; 0..4),
        node!(InlinePassthru(nodes![node!("_foo_&"; 7..13)]), 4..16,),
        node!(" bar"; 16..20),
      ],
    ),
    (
      "+foo+ bar +baz+", // two passthrus on one line
      nodes![
        node!(InlinePassthru(just!("foo", 1..4)), 0..5),
        node!(" bar "; 5..10),
        node!(InlinePassthru(just!("baz", 11..14)), 10..15),
      ],
    ),
    (
      "+foo+bar", // single plus = not unconstrained, not a passthrough
      just!("+foo+bar", 0..8),
    ),
    (
      "+foo\nbar+ baz", // multi-line
      nodes![
        node!(
          InlinePassthru(nodes![
            node!("foo"; 1..4),
            node!(Inline::Newline, 4..5),
            node!("bar"; 5..8),
          ]),
          0..9
        ),
        node!(" baz"; 9..13),
      ],
    ),
    (
      "+foo\nbar+baz", // multi-line constrained can't terminate within word
      nodes![
        // no InlinePassthrough
        node!("+foo"; 0..4),
        node!(Inline::Newline, 4..5),
        node!("bar+baz"; 5..12),
      ],
    ),
    (
      "++foo\nbar++", // multi-line unconstrained
      nodes![node!(
        InlinePassthru(nodes![
          node!("foo"; 2..5),
          node!(Inline::Newline, 5..6),
          node!("bar"; 6..9),
        ]),
        0..11
      )],
    ),
    (
      "pass:[_foo_]",
      nodes![node!(InlinePassthru(just!("_foo_", 6..11)), 0..12)],
    ),
    (
      "pass:q[_foo_] bar", // subs=quotes
      nodes![
        node!(
          InlinePassthru(nodes![node!(Italic(just!("foo", 8..11)), 7..12)]),
          0..13
        ),
        node!(" bar"; 13..17),
      ],
    ),
    (
      "pass:a,c[_foo_\nbar]",
      nodes![node!(
        InlinePassthru(nodes![
          node!("_foo_"; 9..14),
          node!(Inline::Newline, 14..15),
          node!("bar"; 15..18),
        ]),
        0..19
      )],
    ),
  ]);
}

#[test]
fn test_line_comments() {
  run(vec![(
    "foo\n// baz\nbar",
    nodes![
      node!("foo"; 0..3),
      node!(Inline::Newline, 3..4),
      node!(LineComment(bstr!(" baz")), 4..11),
      node!("bar"; 11..14),
    ],
  )]);
}

#[test]
fn test_joining_newlines() {
  run(vec![
    ("{foo}", just!("{foo}", 0..5)),
    (
      "\\{foo}",
      nodes![node!(Discarded, 0..1), node!("{foo}"; 1..6)],
    ),
    ("{attribute-missing}", just!("skip", 0..19)),
    (
      "\\{attribute-missing}",
      nodes![node!(Discarded, 0..1), node!("{attribute-missing}"; 1..20)],
    ),
    (
      "_foo_\nbar",
      nodes![
        node!(Italic(nodes![node!("foo"; 1..4)]), 0..5),
        node!(Inline::Newline, 5..6),
        node!("bar"; 6..9),
      ],
    ),
    (
      "__foo__\nbar",
      nodes![
        node!(Italic(nodes![node!("foo"; 2..5)]), 0..7),
        node!(Inline::Newline, 7..8),
        node!("bar"; 8..11),
      ],
    ),
    (
      "foo \"`bar`\"\nbaz",
      nodes![
        node!("foo "; 0..4),
        node!(Quote(QuoteKind::Double, nodes![node!("bar"; 6..9)]), 4..11),
        node!(Inline::Newline, 11..12),
        node!("baz"; 12..15),
      ],
    ),
    (
      "\"`foo\nbar`\"\nbaz",
      nodes![
        node!(
          Quote(
            QuoteKind::Double,
            nodes![
              node!("foo"; 2..5),
              node!(Inline::Newline, 5..6),
              node!("bar"; 6..9),
            ],
          ),
          0..11,
        ),
        node!(Inline::Newline, 11..12),
        node!("baz"; 12..15),
      ],
    ),
    (
      "bar`\"\nbaz",
      nodes![
        node!("bar"; 0..3),
        node!(CurlyQuote(CurlyKind::RightDouble), 3..5),
        node!(Inline::Newline, 5..6),
        node!("baz"; 6..9),
      ],
    ),
    (
      "^foo^\nbar",
      nodes![
        node!(Superscript(nodes![node!("foo"; 1..4)]), 0..5),
        node!(Inline::Newline, 5..6),
        node!("bar"; 6..9),
      ],
    ),
    (
      "~foo~\nbar",
      nodes![
        node!(Subscript(nodes![node!("foo"; 1..4)]), 0..5),
        node!(Inline::Newline, 5..6),
        node!("bar"; 6..9),
      ],
    ),
    (
      "`+{name}+`\nbar",
      nodes![
        node!(LitMono(src!("{name}", 2..8)), 0..10),
        node!(Inline::Newline, 10..11),
        node!("bar"; 11..14),
      ],
    ),
    (
      "+_foo_+\nbar",
      nodes![
        node!(InlinePassthru(nodes![node!("_foo_"; 1..6)]), 0..7,),
        node!(Inline::Newline, 7..8),
        node!("bar"; 8..11),
      ],
    ),
    (
      "+++_<foo>&_+++\nbar",
      nodes![
        node!(InlinePassthru(nodes![node!("_<foo>&_"; 3..11)]), 0..14,),
        node!(Inline::Newline, 14..15),
        node!("bar"; 15..18),
      ],
    ),
  ]);
}

#[test]
fn test_line_breaks() {
  run(vec![
    (
      "foo +\nbar",
      nodes![
        node!("foo"; 0..3),
        node!(LineBreak, 3..6),
        node!("bar"; 6..9),
      ],
    ),
    (
      "foo+\nbar", // not valid linebreak
      nodes![
        node!("foo+"; 0..4),
        node!(Inline::Newline, 4..5),
        node!("bar"; 5..8),
      ],
    ),
  ]);
}

#[test]
fn test_inline_anchors() {
  run(vec![
    (
      "[[foo]]bar",
      nodes![node!(InlineAnchor(bstr!("foo")), 0..7), node!("bar"; 7..10),],
    ),
    (
      "bar[[foo]]",
      nodes![node!("bar"; 0..3), node!(InlineAnchor(bstr!("foo")), 3..10),],
    ),
  ]);
}

#[test]
fn test_parse_inlines() {
  run(vec![
    (
      "+_foo_+",
      nodes![node!(InlinePassthru(nodes![node!("_foo_"; 1..6)]), 0..7,)],
    ),
    (
      "+_{foo}_+",
      nodes![node!(InlinePassthru(nodes![node!("_{foo}_"; 1..8)]), 0..9,)],
    ),
    (
      "+_{attribute-missing}_+",
      nodes![node!(
        InlinePassthru(nodes![node!("_{attribute-missing}_"; 1..22)]),
        0..23,
      )],
    ),
    (
      "`*_foo_*`",
      nodes![node!(
        Mono(nodes![node!(
          Bold(nodes![node!(Italic(nodes![node!("foo"; 3..6)]), 2..7)]),
          1..8,
        )]),
        0..9,
      )],
    ),
    (
      "+_foo\nbar_+",
      // not sure if this is "spec", but it's what asciidoctor currently does
      nodes![node!(
        InlinePassthru(nodes![
          node!("_foo"; 1..5),
          node!(Inline::Newline, 5..6),
          node!("bar_"; 6..10),
        ]),
        0..11,
      )],
    ),
    (
      "+_<foo>&_+",
      nodes![node!(
        InlinePassthru(nodes![
          node!("_"; 1..2),
          node!(SpecialChar(SpecialCharKind::LessThan), 2..3),
          node!("foo"; 3..6),
          node!(SpecialChar(SpecialCharKind::GreaterThan), 6..7),
          node!(SpecialChar(SpecialCharKind::Ampersand), 7..8),
          node!("_"; 8..9),
        ]),
        0..10,
      )],
    ),
    (
      "rofl +_foo_+ lol",
      nodes![
        node!("rofl "; 0..5),
        node!(InlinePassthru(nodes![node!("_foo_"; 6..11)]), 5..12,),
        node!(" lol"; 12..16),
      ],
    ),
    // here
    (
      "++_foo_++bar",
      nodes![
        node!(InlinePassthru(nodes![node!("_foo_"; 2..7)]), 0..9,),
        node!("bar"; 9..12),
      ],
    ),
    (
      "+++_<foo>&_+++ bar",
      nodes![
        node!(InlinePassthru(nodes![node!("_<foo>&_"; 3..11)]), 0..14,),
        node!(" bar"; 14..18),
      ],
    ),
    (
      "foo #bar#",
      nodes![
        node!("foo "; 0..4),
        node!(Highlight(nodes![node!("bar"; 5..8)]), 4..9),
      ],
    ),
    (
      "foo ##bar##baz",
      nodes![
        node!("foo "; 0..4),
        node!(Highlight(nodes![node!("bar"; 6..9)]), 4..11),
        node!("baz"; 11..14),
      ],
    ),
    (
      "foo `bar`",
      nodes![
        node!("foo "; 0..4),
        node!(Mono(nodes![node!("bar"; 5..8)]), 4..9),
      ],
    ),
    (
      "foo b``ar``",
      nodes![
        node!("foo b"; 0..5),
        node!(Mono(nodes![node!("ar"; 7..9)]), 5..11),
      ],
    ),
    (
      "foo *bar*",
      nodes![
        node!("foo "; 0..4),
        node!(Bold(nodes![node!("bar"; 5..8)]), 4..9),
      ],
    ),
    (
      "foo b**ar**",
      nodes![
        node!("foo b"; 0..5),
        node!(Bold(nodes![node!("ar"; 7..9)]), 5..11),
      ],
    ),
    (
      "foo ~bar~ baz",
      nodes![
        node!("foo "; 0..4),
        node!(Subscript(nodes![node!("bar"; 5..8)]), 4..9),
        node!(" baz"; 9..13),
      ],
    ),
    (
      "foo _bar\nbaz_",
      nodes![
        node!("foo "; 0..4),
        node!(
          Italic(nodes![
            node!("bar"; 5..8),
            node!(Inline::Newline, 8..9),
            node!("baz"; 9..12),
          ]),
          4..13,
        ),
      ],
    ),
    ("foo __bar", nodes![node!("foo __bar"; 0..9)]),
    (
      "foo _bar baz_",
      nodes![
        node!("foo "; 0..4),
        node!(Italic(nodes![node!("bar baz"; 5..12)]), 4..13),
      ],
    ),
    (
      "foo _bar_",
      nodes![
        node!("foo "; 0..4),
        node!(Italic(nodes![node!("bar"; 5..8)]), 4..9),
      ],
    ),
    (
      "foo b__ar__",
      nodes![
        node!("foo b"; 0..5),
        node!(Italic(nodes![node!("ar"; 7..9)]), 5..11),
      ],
    ),
    ("foo 'bar'", nodes![node!("foo 'bar'"; 0..9)]),
    ("foo \"bar\"", nodes![node!("foo \"bar\""; 0..9)]),
    (
      "foo `\"bar\"`",
      nodes![
        node!("foo "; 0..4),
        node!(Mono(nodes![node!("\"bar\""; 5..10)]), 4..11),
      ],
    ),
    (
      "foo `'bar'`",
      nodes![
        node!("foo "; 0..4),
        node!(Mono(nodes![node!("'bar'"; 5..10)]), 4..11),
      ],
    ),
    (
      "foo \"`bar`\"",
      nodes![
        node!("foo "; 0..4),
        node!(Quote(QuoteKind::Double, nodes![node!("bar"; 6..9)]), 4..11,),
      ],
    ),
    (
      "foo \"`bar baz`\"",
      nodes![
        node!("foo "; 0..4),
        node!(
          Quote(QuoteKind::Double, nodes![node!("bar baz"; 6..13)]),
          4..15,
        ),
      ],
    ),
    (
      "foo \"`bar\nbaz`\"",
      nodes![
        node!("foo "; 0..4),
        node!(
          Quote(
            QuoteKind::Double,
            nodes![
              node!("bar"; 6..9),
              node!(Inline::Newline, 9..10),
              node!("baz"; 10..13),
            ],
          ),
          4..15,
        ),
      ],
    ),
    (
      "foo '`bar`'",
      nodes![
        node!("foo "; 0..4),
        node!(Quote(QuoteKind::Single, nodes![node!("bar"; 6..9)]), 4..11,),
      ],
    ),
    (
      "Olaf's wrench",
      nodes![
        node!("Olaf"; 0..4),
        node!(CurlyQuote(CurlyKind::LegacyImplicitApostrophe), 4..5),
        node!("s wrench"; 5..13),
      ],
    ),
    (
      "foo   bar",
      nodes![
        node!("foo"; 0..3),
        node!(MultiCharWhitespace(bstr!("   ")), 3..6),
        node!("bar"; 6..9),
      ],
    ),
    (
      "`+{name}+`",
      nodes![node!(LitMono(src!("{name}", 2..8)), 0..10)],
    ),
    (
      "`+_foo_+`",
      nodes![node!(LitMono(src!("_foo_", 2..7)), 0..9)],
    ),
    (
      "foo <bar> & lol",
      nodes![
        node!("foo "; 0..4),
        node!(SpecialChar(SpecialCharKind::LessThan), 4..5),
        node!("bar"; 5..8),
        node!(SpecialChar(SpecialCharKind::GreaterThan), 8..9),
        node!(" "; 9..10),
        node!(SpecialChar(SpecialCharKind::Ampersand), 10..11),
        node!(" lol"; 11..15),
      ],
    ),
    (
      "^bar^",
      nodes![node!(Superscript(nodes![node!("bar"; 1..4)]), 0..5)],
    ),
    (
      "^bar^",
      nodes![node!(Superscript(nodes![node!("bar"; 1..4)]), 0..5)],
    ),
    ("foo ^bar", nodes![node!("foo ^bar"; 0..8)]),
    ("foo bar^", nodes![node!("foo bar^"; 0..8)]),
    (
      "foo ^bar^ foo",
      nodes![
        node!("foo "; 0..4),
        node!(Superscript(nodes![node!("bar"; 5..8)]), 4..9),
        node!(" foo"; 9..13),
      ],
    ),
    (
      "doublefootnote:[ymmv _i_]bar",
      nodes![
        node!("double"; 0..6),
        node!(
          Macro(Footnote {
            number: 1,
            id: None,
            text: nodes![
              node!("ymmv "; 16..21),
              node!(Italic(nodes![node!("i"; 22..23)]), 21..24),
            ],
          }),
          6..25,
        ),
        node!("bar"; 25..28),
      ],
    ),
    (
      "[.role]#bar#",
      nodes![node!(
        TextSpan(
          AttrList {
            roles: vecb![src!("role", 2..6)],
            ..attr_list!(0..7)
          },
          just!("bar", 8..11),
        ),
        0..12
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
