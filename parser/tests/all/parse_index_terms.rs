use asciidork_ast::{InlineNodes, prelude::*};
use asciidork_parser::prelude::*;
use test_utils::*;

#[test]
fn test_parse_index_macros_visible() {
  run(vec![
    (
      "indexterm2:[foo]",
      nodes![node!(
        Inline::IndexTerm(asciidork_ast::IndexTerm {
          term_type: IndexTermType::Visible { term: just!("foo", 12..15) },
          term_ref: IndexTermReference::None
        }),
        0..16
      )],
    ),
    (
      "indexterm2:[foo bar]",
      nodes![node!(
        Inline::IndexTerm(asciidork_ast::IndexTerm {
          term_type: IndexTermType::Visible { term: just!("foo bar", 12..19) },
          term_ref: IndexTermReference::None
        }),
        0..20
      )],
    ),
    (
      "indexterm2:[Flash,see=HTML 5]",
      nodes![node!(
        Inline::IndexTerm(asciidork_ast::IndexTerm {
          term_type: IndexTermType::Visible { term: just!("Flash", 12..17) },
          term_ref: IndexTermReference::See(bstr!("HTML 5"))
        }),
        0..29
      )],
    ),
    (
      r#"indexterm2:[HTML 5,see-also="CSS 3, SVG"]"#,
      nodes![node!(
        Inline::IndexTerm(asciidork_ast::IndexTerm {
          term_type: IndexTermType::Visible { term: just!("HTML 5", 12..18) },
          term_ref: IndexTermReference::SeeAlso(vec![bstr!("CSS 3"), bstr!("SVG")])
        }),
        0..41
      )],
    ),
  ]);
}

#[test]
fn test_parse_index_macros_concealed() {
  run(vec![
    (
      "indexterm:[foo]",
      nodes![node!(
        Inline::IndexTerm(asciidork_ast::IndexTerm {
          term_type: IndexTermType::Concealed {
            primary: just!("foo", 11..14),
            secondary: None,
            tertiary: None,
          },
          term_ref: IndexTermReference::None
        }),
        0..15
      )],
    ),
    (
      "indexterm:[foo, bar baz]",
      nodes![node!(
        Inline::IndexTerm(asciidork_ast::IndexTerm {
          term_type: IndexTermType::Concealed {
            primary: just!("foo", 11..14),
            secondary: Some(just!("bar baz", 16..23)),
            tertiary: None,
          },
          term_ref: IndexTermReference::None
        }),
        0..24
      )],
    ),
    (
      r#"indexterm:[foo, bar, baz, see-also="thing, other thing"]"#,
      nodes![node!(
        Inline::IndexTerm(asciidork_ast::IndexTerm {
          term_type: IndexTermType::Concealed {
            primary: just!("foo", 11..14),
            secondary: Some(just!("bar", 16..19)),
            tertiary: Some(just!("baz", 21..24)),
          },
          term_ref: IndexTermReference::SeeAlso(vec![bstr!("thing"), bstr!("other thing")])
        }),
        0..56
      )],
    ),
    (
      r#"indexterm:[foo, "bar, baz",see=qux]"#,
      nodes![node!(
        Inline::IndexTerm(asciidork_ast::IndexTerm {
          term_type: IndexTermType::Concealed {
            primary: just!("foo", 11..14),
            secondary: Some(just!("bar, baz", 17..25)),
            tertiary: None,
          },
          term_ref: IndexTermReference::See(bstr!("qux"))
        }),
        0..35
      )],
    ),
  ]);
}

#[test]
fn test_parse_index_parens_concealed() {
  run(vec![
    (
      "(((foo, bar , \"baz, bar\")))",
      nodes![node!(
        Inline::IndexTerm(asciidork_ast::IndexTerm {
          term_type: IndexTermType::Concealed {
            primary: just!("foo", 3..6),
            secondary: Some(just!("bar", 8..11)),
            tertiary: Some(just!("baz, bar", 15..23)),
          },
          term_ref: IndexTermReference::None
        }),
        0..27
      )],
    ),
    (
      "(((foo)))",
      nodes![node!(
        Inline::IndexTerm(asciidork_ast::IndexTerm {
          term_type: IndexTermType::Concealed {
            primary: just!("foo", 3..6),
            secondary: None,
            tertiary: None,
          },
          term_ref: IndexTermReference::None
        }),
        0..9
      )],
    ),
    (
      "(((foo, _bar_)))",
      nodes![node!(
        Inline::IndexTerm(asciidork_ast::IndexTerm {
          term_type: IndexTermType::Concealed {
            primary: just!("foo", 3..6),
            secondary: Some(nodes![node!(
              Inline::Span(SpanKind::Italic, None, just!("bar", 9..12)),
              8..13
            )]),
            tertiary: None,
          },
          term_ref: IndexTermReference::None
        }),
        0..16
      )],
    ),
  ]);
}

#[test]
fn test_parse_index_parens_edge_cases() {
  run(vec![
    (
      "\\(((NIST)))",
      nodes![
        node!(Inline::Discarded, 0..1),
        node!("("; 1..2),
        node!(
          Inline::IndexTerm(asciidork_ast::IndexTerm {
            term_type: IndexTermType::Visible { term: just!("NIST)", 4..9) },
            term_ref: IndexTermReference::None
          }),
          2..11
        ),
      ],
    ),
    (
      "(((foo (bar))))",
      nodes![node!(
        Inline::IndexTerm(asciidork_ast::IndexTerm {
          term_type: IndexTermType::Concealed {
            primary: just!("foo (bar)", 3..12),
            secondary: None,
            tertiary: None
          },
          term_ref: IndexTermReference::None
        }),
        0..15
      )],
    ),
    (
      "((((4-3)))",
      nodes![node!(
        Inline::IndexTerm(asciidork_ast::IndexTerm {
          term_type: IndexTermType::Concealed {
            primary: just!("(4-3", 3..7),
            secondary: None,
            tertiary: None,
          },
          term_ref: IndexTermReference::None
        }),
        0..10
      )],
    ),
    (
      "((((4-4))))",
      nodes![node!(
        Inline::IndexTerm(asciidork_ast::IndexTerm {
          term_type: IndexTermType::Concealed {
            primary: just!("(4-4)", 3..8),
            secondary: None,
            tertiary: None,
          },
          term_ref: IndexTermReference::None
        }),
        0..11
      )],
    ),
    (
      "((2-4))))",
      nodes![node!(
        Inline::IndexTerm(asciidork_ast::IndexTerm {
          term_type: IndexTermType::Visible { term: just!("2-4))", 2..7) },
          term_ref: IndexTermReference::None
        }),
        0..9
      )],
    ),
    (
      "((2-5)))))",
      nodes![node!(
        Inline::IndexTerm(asciidork_ast::IndexTerm {
          term_type: IndexTermType::Visible { term: just!("2-5)))", 2..8) },
          term_ref: IndexTermReference::None
        }),
        0..10
      )],
    ),
    (
      "((((4-4))))",
      nodes![node!(
        Inline::IndexTerm(asciidork_ast::IndexTerm {
          term_type: IndexTermType::Concealed {
            primary: just!("(4-4)", 3..8),
            secondary: None,
            tertiary: None,
          },
          term_ref: IndexTermReference::None
        }),
        0..11
      )],
    ),
    (
      "((((4-3)))",
      nodes![node!(
        Inline::IndexTerm(asciidork_ast::IndexTerm {
          term_type: IndexTermType::Concealed {
            primary: just!("(4-3", 3..7),
            secondary: None,
            tertiary: None,
          },
          term_ref: IndexTermReference::None
        }),
        0..10
      )],
    ),
    (
      "((((4-2))",
      nodes![
        node!("(("; 0..2),
        node!(
          Inline::IndexTerm(asciidork_ast::IndexTerm {
            term_type: IndexTermType::Visible { term: just!("4-2", 4..7) },
            term_ref: IndexTermReference::None
          }),
          0..9
        ),
      ],
    ),
    (
      "(((((5-2))",
      nodes![
        node!("((("; 0..3),
        node!(
          Inline::IndexTerm(asciidork_ast::IndexTerm {
            term_type: IndexTermType::Visible { term: just!("5-2", 5..8) },
            term_ref: IndexTermReference::None
          }),
          0..10
        ),
      ],
    ),
    (
      "(((foo)) bar)",
      nodes![
        node!("("; 0..1),
        node!(
          Inline::IndexTerm(asciidork_ast::IndexTerm {
            term_type: IndexTermType::Visible { term: just!("foo", 3..6) },
            term_ref: IndexTermReference::None
          }),
          0..8
        ),
        node!(" bar)"; 8..13)
      ],
    ),
    (
      "\\((foo))",
      nodes![node!(Inline::Discarded, 0..1), node!("((foo))"; 1..8)],
    ),
    ("((((foo", just!("((((foo", 0..7)),
    ("(foo))", just!("(foo))", 0..6)),
  ]);
}

#[test]
fn test_parse_index_parens_visible() {
  run(vec![
    (
      "(( panthera\ntigris ))",
      nodes![node!(
        Inline::IndexTerm(asciidork_ast::IndexTerm {
          term_type: IndexTermType::Visible {
            term: nodes![
              node!("panthera"; 3..11),
              node!(Inline::Newline, 11..12),
              node!("tigris"; 12..18),
            ]
          },
          term_ref: IndexTermReference::None
        }),
        0..20
      )],
    ),
    (
      "((foo))",
      nodes![node!(
        Inline::IndexTerm(asciidork_ast::IndexTerm {
          term_type: IndexTermType::Visible { term: just!("foo", 2..5) },
          term_ref: IndexTermReference::None
        }),
        0..7
      )],
    ),
    (
      "((*foo*))",
      nodes![node!(
        Inline::IndexTerm(asciidork_ast::IndexTerm {
          term_type: IndexTermType::Visible {
            term: nodes![node!(
              Inline::Span(SpanKind::Bold, None, nodes![node!("foo"; 3..6)]),
              2..7
            )]
          },
          term_ref: IndexTermReference::None
        }),
        0..9
      )],
    ),
    (
      "((foo bar))",
      nodes![node!(
        Inline::IndexTerm(asciidork_ast::IndexTerm {
          term_type: IndexTermType::Visible { term: just!("foo bar", 2..9) },
          term_ref: IndexTermReference::None
        }),
        0..11
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
