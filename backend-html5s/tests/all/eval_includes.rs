use asciidork_core::{JobSettings, SafeMode};
// use asciidork_parser::includes::*;
use test_utils::*;

assert_html!(
  simple_include_no_newline,
  resolving: b"Line-2",
  adoc! {r#"
    Line-1
    include::some_file.adoc[]
  "#},
  raw_html! {r#"
    <p>Line-1
    Line-2</p>"#}
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
    <p>para1</p>
    <p>included</p>
    <p>para2</p>
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
  raw_html! {r#"
    <p>Line-1
    <a href="file.adoc" class="bare include">file.adoc</a>
    Line-3</p><p><a href="with spaces.adoc" class="bare include">with spaces.adoc</a></p><p><a href="http://a.us/b.adoc" class="bare include">http://a.us/b.adoc</a></p>"#}
);
