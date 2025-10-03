use test_utils::*;

// NOTE: DSV table tests updated for jirutka backend's simplified table structure:
// - div.table-block wrapper instead of direct table
// - Simplified class names and no paragraph wrappers for simple cells

assert_html!(
  basic_dsv_table,
  adoc! {r#"
    :===
    a : b
    c : d\:e
    :===
  "#},
  html! {r#"
    <div class="table-block"><table class="frame-all grid-all stretch"><colgroup><col style="width: 50%;"><col style="width: 50%;"></colgroup><tbody><tr><td class="halign-left valign-top">a</td><td class="halign-left valign-top">b</td></tr><tr><td class="halign-left valign-top">c</td><td class="halign-left valign-top">d:e</td></tr></tbody></table></div>
  "#}
);

assert_html!(
  complex_dsv_table,
  adoc! {r#"
    [%header%footer,format=dsv,separator=;,cols="1a,2e,1m"]
    |===
    one;two
    three

    four;five;
    |===
  "#},
  html! {r#"
    <div class="table-block"><table class="frame-all grid-all stretch"><colgroup><col style="width: 25%;"><col style="width: 50%;"><col style="width: 25%;"></colgroup><thead><tr><th class="halign-left valign-top">one</th><th class="halign-left valign-top">two</th><th class="halign-left valign-top">three</th></tr></thead><tfoot><tr><td class="halign-left valign-top"><p>four</p></td><td class="halign-left valign-top"><em>five</em></td><td class="halign-left valign-top"></td></tr></tfoot></table></div>
  "#}
);
