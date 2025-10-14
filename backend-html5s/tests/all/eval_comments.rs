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
    <div class="example-block"><div class="example"><p>first paragraph</p>
    <p>second paragraph</p></div></div>
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
    <p>first paragraph</p>
    <p>second paragraph</p>
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
    <p>not this text</p>
  "#}
);
