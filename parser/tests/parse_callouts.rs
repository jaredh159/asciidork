use asciidork_ast::prelude::*;
use asciidork_ast::short::block::*;
use asciidork_ast::variants::inline::*;
use asciidork_parser::Parser;
use test_utils::{assert_eq, *};

#[test]
fn test_parse_callout_nums() {
  let input = adoc! {r#"
    ....
    System.out.println("Hello, world!"); <1>
    ....
  "#};
  parse_block!(input, block, b);
  assert_eq!(
    block,
    Block {
      context: Context::Literal,
      content: Content::Simple(b.inodes([
        n_text(r#"System.out.println("Hello, world!");"#, 5, 41, b),
        n(CalloutNum(1), l(41, 45))
      ])),
      ..b.empty_block(0, 50)
    }
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
  parse_block!(input, block, b);
  assert_eq!(
    block,
    Block {
      context: Context::Literal,
      content: Content::Simple(b.inodes([
        n_text("foo", 5, 8, b),
        n(CalloutNum(1), l(8, 12)),
        n(JoiningNewline, l(12, 13)),
        n_text("bar baz", 13, 20, b),
        n(CalloutNum(2), l(20, 24)),
        n(CalloutNum(3), l(24, 28)),
      ])),
      ..b.empty_block(0, 33)
    }
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
  parse_block!(input, block, b);
  assert_eq!(
    block,
    Block {
      context: Context::Literal,
      content: Content::Simple(b.inodes([
        n(SpecialChar(SpecialCharKind::LessThan), l(5, 6)),
        n_text("foo", 6, 9, b),
        n(SpecialChar(SpecialCharKind::GreaterThan), l(9, 10)),
        n(CalloutNum(1), l(10, 14)),
        n(JoiningNewline, l(14, 15)),
        n_text("a ", 15, 17, b),
        n(SpecialChar(SpecialCharKind::LessThan), l(17, 18)),
        n_text("1", 18, 19, b),
        n(SpecialChar(SpecialCharKind::GreaterThan), l(19, 20)),
        n_text(" b", 20, 22, b),
        n(CalloutNum(2), l(22, 26)),
        n(CalloutNum(3), l(26, 30)),
      ])),
      ..b.empty_block(0, 35)
    }
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
  parse_block!(input, block, b);
  assert_eq!(
    block,
    Block {
      meta: ChunkMeta::new(
        Some(AttrList {
          named: Named::from(b.vec([(b.src("subs", l(1, 5)), b.src("-specialchars", l(6, 19)))])),
          ..AttrList::new(l(0, 20), b)
        }),
        None,
        0,
      ),
      context: Context::Literal,
      content: Content::Simple(
        b.inodes([n_text("a <3> b", 26, 33, b), n(CalloutNum(1), l(33, 37))])
      ),
      ..b.empty_block(0, 42)
    }
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
  parse_block!(input, block, b);
  assert_eq!(
    block,
    Block {
      meta: ChunkMeta::new(
        Some(AttrList {
          named: Named::from(b.vec([(b.src("subs", l(1, 5)), b.src("none", l(6, 10)))])),
          ..AttrList::new(l(0, 11), b)
        }),
        None,
        0,
      ),
      context: Context::Literal,
      content: Content::Simple(b.inodes([n_text("a <3> b <1>", 17, 28, b)])),
      ..b.empty_block(0, 33)
    }
  );
}

#[test]
fn test_parse_non_delim_callout() {
  let input = adoc! {"
    [, ruby]
    puts 'hello world' <1>
  "};
  parse_block!(input, block, b);
  assert_eq!(
    block,
    Block {
      meta: ChunkMeta::new(
        Some(AttrList {
          positional: b.vec([None, Some(b.inodes([n_text("ruby", 3, 7, b)])),]),
          ..AttrList::new(l(0, 8), b)
        }),
        None,
        0,
      ),
      context: Context::Paragraph,
      content: Content::Simple(b.inodes([
        n_text("puts 'hello world'", 9, 27, b),
        n(CalloutNum(1), l(27, 31)),
      ])),
      ..b.empty_block(0, 31)
    }
  );
}

#[test]
fn test_parse_xml_callout_num() {
  let input = adoc! {r#"
    ....
    Hello world! <!--1-->
    ....
  "#};
  parse_block!(input, block, b);
  assert_eq!(
    block,
    Block {
      context: Context::Literal,
      content: Content::Simple(b.inodes([
        n_text("Hello world!", 5, 17, b),
        n(CalloutNum(1), l(17, 26))
      ])),
      ..b.empty_block(0, 31)
    }
  );
}
