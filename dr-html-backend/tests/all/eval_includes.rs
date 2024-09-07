use asciidork_meta::{JobSettings, SafeMode};
use asciidork_parser::includes::*;
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
  include_separated_paras,
  resolving: b"included\n",
  adoc! {r#"
    para1

    include::some_file.adoc[]

    para2
  "#},
  html! {r#"
    <div class="paragraph"><p>para1</p></div>
    <div class="paragraph"><p>included</p></div>
    <div class="paragraph"><p>para2</p></div>
  "#}
);

assert_html!(
  secure_include_to_link,
  |settings: &mut JobSettings| {
    settings.safe_mode = SafeMode::Secure;
  },
  adoc! {r#"
    Line-1
    include::file.adoc[]
    Line-3

    include::with spaces.adoc[]

    include::http://a.us/b.adoc[]
  "#},
  html! {r#"
    <div class="paragraph">
      <p>Line-1 <a href="file.adoc" class="bare include">file.adoc</a> Line-3</p>
    </div>
    <div class="paragraph">
      <p><a href="with spaces.adoc" class="bare include">with spaces.adoc</a></p>
    </div>
    <div class="paragraph">
      <p><a href="http://a.us/b.adoc" class="bare include">http://a.us/b.adoc</a></p>
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
  include_strips_bom,
  resolving: [0xEF, 0xBB, 0xBF, 0xE4, 0xBA, 0xBA, 0x0A],
  adoc! {r#"
    Line-1
    include::some_file.adoc[]
    Line-3
  "#},
  html! {r#"
    <div class="paragraph">
      <p>Line-1 人 Line-3</p>
    </div>
  "#}
);

assert_html!(
  include_honors_encoding,
  resolving: [0x68, 0x00, 0x69, 0x00], // <-- "hi" in UTF-16 LE
  adoc! {r#"
    Line-1
    include::some_file.adoc[encoding=utf-16]
    Line-3
  "#},
  html! {r#"
    <div class="paragraph">
      <p>Line-1 hi Line-3</p>
    </div>
  "#}
);

assert_html!(
  include_err_on_missing_file,
  resolving_err: ResolveError::NotFound,
  "include::404.adoc[]",
  html! {r#"
    <div class="paragraph">
      <p>Unresolved directive in test.adoc - include::404.adoc[]</p>
    </div>
  "#}
);

assert_html!(
  include_err_on_io,
  resolving_err: ResolveError::Io("permission denied".into()),
  "include::404.adoc[]",
  html! {r#"
    <div class="paragraph">
      <p>Unresolved directive in test.adoc - include::404.adoc[]</p>
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
