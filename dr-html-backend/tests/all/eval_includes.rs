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
