use asciidork_ast::prelude::*;
use asciidork_ast::short::block::*;
use asciidork_ast::variants::inline::*;
use asciidork_parser::Parser;
use test_utils::{assert_eq, *};

fn callout<'bmp>(num: u8, list_idx: u8, idx: u8) -> asciidork_ast::Inline<'bmp> {
  CalloutNum(Callout { list_idx, callout_idx: idx, num })
}

#[test]
fn test_parse_callout() {
  let input = adoc! {r#"
    ....
    System.out.println("Hello, world!"); <1>
    ....
  "#};
  assert_block_core!(
    input,
    Context::Literal,
    Content::Simple(nodes![
      node!("System.out.println(\"Hello, world!\");"; 5..41),
      node!(callout(1, 0, 0), 41..45),
    ])
  );
}

#[test]
fn test_parse_callout_nums() {
  let input = adoc! {r#"
    ....
    System.out.println("Hello, world!"); <1>
    ....
  "#};
  assert_block_core!(
    input,
    Context::Literal,
    Content::Simple(nodes![
      node!("System.out.println(\"Hello, world!\");"; 5..41),
      node!(callout(1, 0, 0), 41..45),
    ]),
  );
}

#[test]
fn test_parse_callout_behind_comment() {
  let input = adoc! {"
    ----
    foo // <1>
    bar # <2>
    baz ;; <3>
    ----
  "};
  assert_block_core!(
    input,
    Context::Listing,
    Content::Simple(nodes![
      node!("foo "; 5..9),
      node!(CalloutTuck(bstr!("// ")), 9..12),
      node!(callout(1, 0, 0), 12..15),
      node!(JoiningNewline, 15..16),
      node!("bar "; 16..20),
      node!(CalloutTuck(bstr!("# ")), 20..22),
      node!(callout(2, 0, 1), 22..25),
      node!(JoiningNewline, 25..26),
      node!("baz "; 26..30),
      node!(CalloutTuck(bstr!(";; ")), 30..33),
      node!(callout(3, 0, 2), 33..36),
    ]),
  );
}

#[test]
fn test_parse_callout_behind_custom_comment() {
  let input = adoc! {"
    [line-comment=%]
    ----
    foo % <1>
    ----
  "};
  assert_block_core!(
    input,
    Context::Listing,
    Content::Simple(nodes![
      node!("foo "; 22..26),
      node!(CalloutTuck(bstr!("% ")), 26..28),
      node!(callout(1, 0, 0), 28..31),
    ]),
  );
}

#[test]
fn test_parse_multiple_callout_nums() {
  let input = adoc! {"
    ....
    foo <1>
    bar baz <2> <3>
    ....
  "};
  assert_block_core!(
    input,
    Context::Literal,
    Content::Simple(nodes![
      node!("foo"; 5..8),
      node!(callout(1, 0, 0), 8..12),
      node!(JoiningNewline, 12..13),
      node!("bar baz"; 13..20),
      node!(callout(2, 0, 1), 20..24),
      node!(callout(3, 0, 2), 24..28),
    ]),
  );
}

#[test]
fn test_handling_special_chars_and_misplaced_callouts() {
  let input = adoc! {"
    ....
    <foo> <1>
    a <1> b <2> <3>
    ....
  "};
  assert_block_core!(
    input,
    Context::Literal,
    Content::Simple(nodes![
      node!(SpecialChar(SpecialCharKind::LessThan), 5..6),
      node!("foo"; 6..9),
      node!(SpecialChar(SpecialCharKind::GreaterThan), 9..10),
      node!(callout(1, 0, 0), 10..14),
      node!(JoiningNewline, 14..15),
      node!("a "; 15..17),
      node!(SpecialChar(SpecialCharKind::LessThan), 17..18),
      node!("1"; 18..19),
      node!(SpecialChar(SpecialCharKind::GreaterThan), 19..20),
      node!(" b"; 20..22),
      node!(callout(2, 0, 1), 22..26),
      node!(callout(3, 0, 2), 26..30),
    ]),
  );
}

#[test]
fn test_subs_minus_specialchars_misplaced_callout() {
  let input = adoc! {"
    [subs=-specialchars]
    ....
    a <3> b <1>
    ....
  "};
  assert_block_core!(
    input,
    Context::Literal,
    Content::Simple(nodes![
      node!("a <3> b"; 26..33),
      node!(callout(1, 0, 0), 33..37)
    ]),
  );
}

#[test]
fn test_subs_none_callout() {
  let input = adoc! {"
    [subs=none]
    ....
    a <3> b <1>
    ....
  "};
  assert_block_core!(
    input,
    Context::Literal,
    Content::Simple(nodes![node!("a <3> b <1>"; 17..28)]),
  );
}

#[test]
fn test_parse_non_delim_callout() {
  let input = adoc! {"
    [, ruby]
    puts 'hello world' <1>
  "};
  assert_block_core!(
    input,
    Context::Paragraph,
    Content::Simple(nodes![
      node!("puts 'hello world'"; 9..27),
      node!(callout(1, 0, 0), 27..31),
    ]),
  );
}

#[test]
fn test_parse_xml_callout_num() {
  let input = adoc! {r#"
    ....
    Hello world! <!--1-->
    ....
  "#};
  assert_block_core!(
    input,
    Context::Literal,
    Content::Simple(nodes![
      node!("Hello world!"; 5..17),
      node!(callout(1, 0, 0), 17..26),
    ]),
  );
}
