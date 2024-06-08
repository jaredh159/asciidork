// use asciidork_ast::prelude::*;
// use asciidork_parser::Parser;
// use test_utils::{assert_eq, *};

// #[test]
// fn basic_include() {
//   assert_doc_content!(
//     adoc! {"
//       Para 1

//       include::some_file.adoc[]
//     "},
//     resolving: b"Para 2",
//     DocContent::Blocks(vecb![
//       simple_text_block!("Para 1", 0..6),
//       simple_text_block!("Para 2", 8..14),
//     ])
//   )
// }
