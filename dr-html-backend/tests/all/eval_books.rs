// use test_utils::*;

// assert_html!(
//   block_comment_inside_example,
//   adoc! {r#"
//     = Book Title
//     :doctype: book

//     = Part 1

//     == Chapter A

//     content
//   "#},
//   html! {r#"
//     <h1 id="_part_1" class="sect0">Part 1</h1>
//     <div class="sect1">
//       <h2 id="_chapter_a">Chapter A</h2>
//       <div class="sectionbody">
//         <div class="paragraph"><p>content</p></div>
//       </div>
//     </div>
//   "#}
// );
