use asciidork_ast::prelude::*;
use asciidork_parser::prelude::*;
use test_utils::*;

#[test]
fn ifevals_numeric() {
  assert_blocks!(
    adoc! {"
      ifeval::[1 < 2]
      Line 1
      endif::[]

      ifeval::[333 >= 333]
      Line 2
      endif::[]

      ifeval::[1.4 <= 0.2]
      Line 3
      endif::[]
    "},
    &[
      simple_text_block!("Line 1", 16..22),
      simple_text_block!("Line 2", 55..61),
    ],
  );
}

#[test]
fn ifeval_missing_attr_eq_empty_str_includes() {
  assert_blocks!(
    adoc! {"
      ifeval::['{foo}' == '']
      Included
      endif::[]
    "},
    &[simple_text_block!("Included", 24..32)],
  );
}

#[test]
fn ifeval_missing_attr_eq_zero_not_included() {
  assert_blocks!(
    adoc! {"
      ifeval::[{leveloffset} == 0]
      Not included
      endif::[]
    "},
    &[],
  );
}

#[test]
fn ifeval_missing_attr_type_mismatch() {
  assert_blocks!(
    adoc! {"
      ifeval::[{leveloffset} >= 3]
      Not included
      endif::[]

      :asciidork-version: 2.99.4

      ifeval::[{asciidork-version} > true]
      Not included
      endif::[]
    "},
    &[],
  );
}

#[test]
fn ifeval_comparing_strs() {
  assert_blocks!(
    adoc! {r#"
      :some-attr: foo
      :asciidork-version: 2.99.4

      ifeval::["{some-attr}" == "foo"]
      Included 1
      endif::[]

      ifeval::['{some-attr}' == 'foo']
      Included 2
      endif::[]

      ifeval::["{some-attr}" == "not-foo"]
      Not included 3
      endif::[]

      ifeval::["{asciidork-version}" >= "0.1.0"]
      Included 4
      endif::[]

      ifeval::["{some-attr}" == "{some-attr}"]
      Included 5
      endif::[]
    "#},
    &[
      simple_text_block!("Included 1", 77..87),
      simple_text_block!("Included 2", 132..142),
      simple_text_block!("Included 4", 260..270),
      simple_text_block!("Included 5", 323..333),
    ],
  );
}

#[test]
fn ifeval_comparing_nums() {
  assert_blocks!(
    adoc! {r#"
      :number: 1

      ifeval::[{number} == 1]
      Included 1
      endif::[]
    "#},
    &[simple_text_block!("Included 1", 36..46)],
  );
}

assert_error!(
  ifeval_invalid_expr,
  resolving: b"",
  adoc! {"
    ifeval::[1 | 2]
    content
    endif::[]
  "},
  error! {"
     --> test.adoc:1:10
      |
    1 | ifeval::[1 | 2]
      |          ^^^^^ Invalid ifeval directive expression
  "}
);

assert_error!(
  ifeval_invalid_empty_expr,
  resolving: b"",
  adoc! {"
    ifeval::[]
    content
    endif::[]
  "},
  error! {"
     --> test.adoc:1:9
      |
    1 | ifeval::[]
      |         ^^ Invalid ifeval directive expression
  "}
);

assert_error!(
  ifeval_invalid_target,
  resolving: b"",
  adoc! {"
    ifeval::target[1 == 1]
    content
  "},
  error! {"
     --> test.adoc:1:9
      |
    1 | ifeval::target[1 == 1]
      |         ^^^^^^ ifeval directive may not include a target
  "}
);
