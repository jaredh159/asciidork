use test_utils::*;

assert_html!(
  simple_include_no_newline,
  resolving: b"Line-2",
  adoc! {r#"
    Line-1
    include::some_file.adoc[]
  "#},
  html! {r#"
    <div class="paragraph">
      <p>Line-1 Line-2</p>
    </div>
  "#}
);

assert_html!(
  inline_include_no_newline,
  resolving: b"Line-2",
  adoc! {r#"
    Line-1
    include::some_file.adoc[]
    Line-3
  "#},
  html! {r#"
    <div class="paragraph">
      <p>Line-1 Line-2 Line-3</p>
    </div>
  "#}
);

assert_html!(
  inline_include_w_newline,
  resolving: b"Line-2\n",
  adoc! {r#"
    Line-1
    include::some_file.adoc[]
    Line-3
  "#},
  html! {r#"
    <div class="paragraph">
      <p>Line-1 Line-2 Line-3</p>
    </div>
  "#}
);

assert_html!(
  inline_include_w_2_newlines,
  resolving: b"Line-2\n\n", // <-- 2 newlines
  adoc! {r#"
    Line-1
    include::some_file.adoc[]
    Line-3
  "#},
  html! {r#"
    <div class="paragraph">
      <p>Line-1 Line-2</p>
    </div>
    <div class="paragraph">
      <p>Line-3</p>
    </div>
  "#}
);

assert_html!(
  include_inner_para_break,
  resolving: b"Line-2\n\nLine-3",
  adoc! {r#"
    Line-1
    include::some_file.adoc[]
    Line-4
  "#},
  html! {r#"
    <div class="paragraph">
      <p>Line-1 Line-2</p>
    </div>
    <div class="paragraph">
      <p>Line-3 Line-4</p>
    </div>
  "#}
);
