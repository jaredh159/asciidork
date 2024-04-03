use asciidork_ast::prelude::*;
use asciidork_ast::short::block::*;
use asciidork_ast::variants::inline::*;
use asciidork_parser::Parser;
use test_utils::{assert_eq, *};

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
      node!(CalloutNum(1), 41..45),
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
      node!(CalloutNum(1), 41..45),
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
      node!(CalloutNum(1), 8..12),
      node!(JoiningNewline, 12..13),
      node!("bar baz"; 13..20),
      node!(CalloutNum(2), 20..24),
      node!(CalloutNum(3), 24..28),
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
      node!(CalloutNum(1), 10..14),
      node!(JoiningNewline, 14..15),
      node!("a "; 15..17),
      node!(SpecialChar(SpecialCharKind::LessThan), 17..18),
      node!("1"; 18..19),
      node!(SpecialChar(SpecialCharKind::GreaterThan), 19..20),
      node!(" b"; 20..22),
      node!(CalloutNum(2), 22..26),
      node!(CalloutNum(3), 26..30),
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
      node!(CalloutNum(1), 33..37)
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
      node!(CalloutNum(1), 27..31),
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
      node!(CalloutNum(1), 17..26),
    ]),
  );
}
