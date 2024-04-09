use asciidork_parser::Parser;
use test_utils::{assert_eq, *};

mod helpers;

test_eval!(
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

test_eval!(
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

test_eval!(
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

test_eval!(
  xskipping_paragraph_comment,
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

test_eval!(
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

test_error!(
  unclosed_comment_block,
  adoc! {r#"
    foobar

    ////
    unclosed comment
  "#},
  error! {"
    3: ////
       ^^^^ This delimiter was never closed
  "}
);
