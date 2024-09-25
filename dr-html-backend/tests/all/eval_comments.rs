use asciidork_parser::prelude::*;
use test_utils::*;

assert_html!(
  block_comment_inside_example,
  adoc! {r#"
    ====
    first paragraph

    ////
    block comment
    ////

    second paragraph
    ====
  "#},
  html! {r#"
    <div class="exampleblock">
      <div class="content">
        <div class="paragraph">
          <p>first paragraph</p>
        </div>
        <div class="paragraph">
          <p>second paragraph</p>
        </div>
      </div>
    </div>
  "#}
);

assert_html!(
  adjacent_comment_block_between_paragraphs,
  adoc! {r#"
    first paragraph
    ////
    block comment
    ////
    second paragraph
  "#},
  html! {r#"
    <div class="paragraph">
      <p>first paragraph</p>
    </div>
    <div class="paragraph">
      <p>second paragraph</p>
    </div>
  "#}
);

assert_html!(
  skipping_paragraph_comment,
  adoc! {r#"
    [comment]
    skip
    this paragraph

    not this text
  "#},
  html! {r#"
    <div class="paragraph">
      <p>not this text</p>
    </div>
  "#}
);

assert_html!(
  skipping_inner_paragraph_comment,
  adoc! {r#"
    ====
    para1

    [comment#idname]
    skip

    para2
    ====
  "#},
  html! {r#"
    <div class="exampleblock">
      <div class="content">
        <div class="paragraph"><p>para1</p></div>
        <div class="paragraph"><p>para2</p></div>
      </div>
    </div>
  "#}
);

assert_html!(
  three_slash_comment_not_a_comment,
  adoc! {r#"
    foo
    /// baz
    bar
  "#},
  html! {r#"
    <div class="paragraph">
      <p>foo /// baz bar</p>
    </div>
  "#}
);

assert_error!(
  unclosed_comment_block,
  adoc! {r#"
    foobar

    ////
    unclosed comment
  "#},
  error! {"
     --> test.adoc:3:1
      |
    3 | ////
      | ^^^^ This delimiter was never closed
  "}
);
