use asciidork_ast::prelude::*;
use asciidork_parser::prelude::*;
use test_utils::*;

#[test]
fn ifdefs_simple() {
  assert_blocks!(
    adoc! {"
      :defined_true:
      :defined_false!:
      :defined_str: foo

      ifdef::defined_true[]
      line 1
      endif::[]

      ifndef::defined_true[]
      line 2
      endif::[]

      ifdef::defined_false[]
      line 3
      endif::defined_false[]

      ifndef::defined_false[]
      line 4
      endif::defined_false[]

      ifdef::defined_str[]
      line 5
      endif::[]

      ifndef::defined_str[]
      line 6
      endif::[]

      ifdef::not_defined[]
      line 7
      endif::[]

      ifndef::not_defined[]
      line 8
      endif::[]
    "},
    &[
      simple_text_block!("line 1", 73..79),
      simple_text_block!("line 4", 210..216),
      simple_text_block!("line 5", 262..268),
      simple_text_block!("line 8", 381..387),
    ],
  );
}

#[test]
fn ifdefs_join() {
  assert_blocks!(
    adoc! {"
      :defined_true:

      line 1
      ifdef::defined_true[]
      line 2
      endif::[]
    "},
    &[Block {
      content: BlockContent::Simple(nodes![
        node!("line 1"; 16..22),
        node!(Inline::Newline, 22..23),
        node!("line 2"; 45..51),
      ]),
      ..empty_block!(16, 51)
    }],
  );
}

#[test]
fn ifdefs_nested() {
  assert_blocks!(
    adoc! {"
      :defined_true:
      :defined_false!:

      ifdef::defined_true[]
      line 1
      ifdef::defined_false[]
      line 2
      endif::[]
      line 3
      endif::[]

      ifdef::defined_false[]
      ifdef::defined_false[]
      line 4
      endif::[]
      endif::[]

      ifdef::defined_true[]
      line 5
      ifndef::defined_true[]
      line 6
      endif::[]
      line 7
      endif::[]

      line 8
      ifdef::not_defined[]
      line 9
      ifdef::not_defined[]
      line 10
      endif::not_defined[]
      line 11
      endif::not_defined[]
      line 12
    "},
    &[
      Block {
        content: BlockContent::Simple(nodes![
          node!("line 1"; 55..61),
          node!(Inline::Newline, 61..62),
          node!("line 3"; 102..108),
        ]),
        ..empty_block!(55, 108)
      },
      Block {
        content: BlockContent::Simple(nodes![
          node!("line 5"; 216..222),
          node!(Inline::Newline, 222..223),
          node!("line 7"; 263..269),
        ]),
        ..empty_block!(216, 269)
      },
      Block {
        content: BlockContent::Simple(nodes![
          node!("line 8"; 281..287),
          node!(Inline::Newline, 287..288),
          node!("line 12"; 395..402),
        ]),
        ..empty_block!(281, 402)
      },
    ],
  );
}

#[test]
fn ifdefs_all() {
  assert_blocks!(
    adoc! {"
      :defined_true:
      :defined_false!:
      :defined_str: foo
      :foo: bar

      ifdef::defined_true+defined_str[]
      line 1
      endif::[]

      ifdef::defined_true+defined_false[]
      line 2
      endif::[]

      ifdef::defined_true+foo[]
      line 3
      endif::[]

      ifdef::foo+defined_true+defined_str[]
      line 4
      endif::[]

      ifdef::foo+defined_true+defined_str+defined_false[]
      line 5
      endif::[]

      ifndef::defined_true+foo[]
      line 6
      endif::[]

      ifndef::defined_false+not_defined[]
      line 7
      endif::[]

      ifndef::defined_false+defined_true[]
      line 8
      endif::[]
    "},
    &[
      simple_text_block!("line 1", 95..101),
      simple_text_block!("line 3", 193..199),
      simple_text_block!("line 4", 249..255),
      simple_text_block!("line 7", 418..424),
    ],
  );
}

#[test]
fn ifdefs_any() {
  assert_blocks!(
    adoc! {"
      :defined_true:
      :defined_false!:
      :defined_str: foo

      ifdef::defined_true,defined_false[]
      line 1
      endif::[]

      ifdef::defined_false,not_defined[]
      line 2
      endif::[]

      ifndef::defined_true,defined_false[]
      line 3
      endif::defined_true,defined_false[]

      ifndef::defined_false,not_defined[]
      line 4
      endif::[]

      ifndef::defined_true,defined_str[]
      line 5
      endif::[]
    "},
    &[
      simple_text_block!("line 1", 87..93),
      simple_text_block!("line 4", 275..281),
    ],
  );
}

#[test]
fn ifdefs_leading_trailing_repeat() {
  assert_blocks!(
    adoc! {"
      :defined_true:

      ifdef::defined_true,[]
      line 1
      endif::[]

      ifdef::,defined_true[]
      line 2
      endif::[]

      ifdef::defined_true+[]
      line 3
      endif::[]

      ifdef::+defined_true[]
      line 4
      endif::[]

      ifdef::defined_true,,not_defined[]
      line 5
      endif::[]

      ifdef::defined_true++not_defined[]
      line 6
      endif::[]
    "},
    &[
      simple_text_block!("line 1", 39..45),
      simple_text_block!("line 2", 80..86),
      simple_text_block!("line 5", 215..221),
    ],
  );
}

#[test]
fn ifdefs_inline() {
  assert_blocks!(
    adoc! {"
      :defined_true:
      :defined_false!:
      :defined_str: foo

      ifdef::defined_true[Line 1]

      Next line

      ifndef::defined_true[Line 2]
    "},
    &[
      simple_text_block!("Line 1", 71..77),
      simple_text_block!("Next line", 80..89),
    ],
  );
}

assert_error!(
  ifdef_no_endif,
  resolving: b"",
  adoc! {"
    ifdef::nope[]
    line 1
    line 2
  "},
  error! {"
     --> test.adoc:1:1
      |
    1 | ifdef::nope[]
      | ^^^^^^^^^^^^^ This ifdef directive was never closed
  "}
);

assert_error!(
  ifdef_no_ifdef,
  resolving: b"",
  adoc! {"
    line 1
    endif::nope[]
  "},
  error! {"
     --> test.adoc:2:1
      |
    2 | endif::nope[]
      | ^^^^^^^^^^^^^ This endif directive has no previous ifdef/ifndef
  "}
);

assert_error!(
  ifdef_mismatched_end,
  resolving: b"",
  adoc! {"
    ifndef::yup[]
    line 1
    endif::nope[]
  "},
  error! {"
     --> test.adoc:3:8
      |
    3 | endif::nope[]
      |        ^^^^ Mismatched endif directive, expected `yup`
  "}
);
