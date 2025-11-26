use test_utils::*;

assert_html!(
  basic_csv_table,
  adoc! {r#"
    ,===
    A1,
    B1,B2
    ,===
  "#},
  html! {r#"
    <div class="table-block"><table class="frame-all grid-all stretch"><colgroup><col style="width: 50%;"><col style="width: 50%;"></colgroup><tbody><tr><td class="halign-left valign-top">A1</td><td class="halign-left valign-top"></td></tr><tr><td class="halign-left valign-top">B1</td><td class="halign-left valign-top">B2</td></tr></tbody></table></div>
  "#}
);

assert_html!(
  complex_csv_table,
  adoc! {r#"
    [cols="1,1,1a",separator=;]
    ,===
    element;description;example

    foo;bar;_baz_
    ,===
  "#},
  html! {r#"
    <div class="table-block"><table class="frame-all grid-all stretch"><colgroup><col style="width: 33.3333%;"><col style="width: 33.3333%;"><col style="width: 33.3333%;"></colgroup><thead><tr><th class="halign-left valign-top">element</th><th class="halign-left valign-top">description</th><th class="halign-left valign-top">example</th></tr></thead><tbody><tr><td class="halign-left valign-top">foo</td><td class="halign-left valign-top">bar</td><td class="halign-left valign-top"><p><em>baz</em></p></td></tr></tbody></table></div>
  "#}
);
