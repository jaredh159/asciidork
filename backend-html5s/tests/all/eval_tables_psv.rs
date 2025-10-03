use test_utils::*;

// NOTE: PSV table tests updated for jirutka backend's simplified table structure:
// - div.table-block wrapper instead of direct table
// - Simplified class names (halign-left vs tableblock halign-left)
// - No <p class="tableblock"> wrappers for simple cell content

assert_html!(
  basic_table,
  adoc! {r#"
    |===
    |a | b
    |c | d
    |===
  "#},
  html! {r#"
    <div class="table-block"><table class="frame-all grid-all stretch"><colgroup><col style="width: 50%;"><col style="width: 50%;"></colgroup><tbody><tr><td class="halign-left valign-top">a</td><td class="halign-left valign-top">b</td></tr><tr><td class="halign-left valign-top">c</td><td class="halign-left valign-top">d</td></tr></tbody></table></div>
  "#}
);

assert_html!(
  break_in_table,
  adoc! {r#"
    |===
    |A +
    B
    |C +
    D +
    E
    |===
  "#},
  contains:
    "A<br>\nB",
    "C<br>\nD<br>\nE",
);

assert_html!(
  formatting_in_header_row,
  adoc! {r#"
    [cols="2*m"]
    |===
    | _foo_ | *bar*

    | a | b
    |===
  "#},
  html! {r#"
    <div class="table-block"><table class="frame-all grid-all stretch"><colgroup><col style="width: 50%;"><col style="width: 50%;"></colgroup><thead><tr><th class="halign-left valign-top"><em>foo</em></th><th class="halign-left valign-top"><strong>bar</strong></th></tr></thead><tbody><tr><td class="halign-left valign-top"><code>a</code></td><td class="halign-left valign-top"><code>b</code></td></tr></tbody></table></div>
  "#}
);

assert_html!(
  formatting_in_non_header_row,
  adoc! {r#"
    [cols="s,e"]
    |===
    | _strong_ | *emphasis*
    | strong
    | emphasis
    |===
  "#},
  html! {r#"
    <div class="table-block"><table class="frame-all grid-all stretch"><colgroup><col style="width: 50%;"><col style="width: 50%;"></colgroup><tbody><tr><td class="halign-left valign-top"><strong><em>strong</em></strong></td><td class="halign-left valign-top"><em><strong>emphasis</strong></em></td></tr><tr><td class="halign-left valign-top"><strong>strong</strong></td><td class="halign-left valign-top"><em>emphasis</em></td></tr></tbody></table></div>
  "#}
);

assert_html!(
  spans_alignments_and_styles,
  adoc! {r#"
    [cols="e,m,^,>s",width="25%"]
    |===
    |1 >s|2 |3 |4
    ^|5 2.2+^.^|6 .3+<.>m|7
    ^|8
    d|9 2+>|10
    |===
  "#},
  html! {r#"
    <div class="table-block"><table class="frame-all grid-all" style="width: 25%;"><colgroup><col style="width: 25%;"><col style="width: 25%;"><col style="width: 25%;"><col style="width: 25%;"></colgroup><tbody><tr><td class="halign-left valign-top"><em>1</em></td><td class="halign-right valign-top"><strong>2</strong></td><td class="halign-center valign-top">3</td><td class="halign-right valign-top"><strong>4</strong></td></tr><tr><td class="halign-center valign-top"><em>5</em></td><td class="halign-center valign-middle" colspan="2" rowspan="2"><code>6</code></td><td class="halign-left valign-bottom" rowspan="3"><code>7</code></td></tr><tr><td class="halign-center valign-top"><em>8</em></td></tr><tr><td class="halign-left valign-top">9</td><td class="halign-right valign-top" colspan="2"><code>10</code></td></tr></tbody></table></div>
  "#}
);

assert_html!(
  custom_table_class,
  adoc! {r#"
    [.so-custom]
    |===
    |a | b
    |===
  "#},
  contains:
    r#"<div class="table-block so-custom"><table class="frame-all grid-all stretch">"#,
);

assert_html!(
  only_numbers_titled_tables,
  adoc! {r#"
    .First
    |===
    |1 |2 |3
    |===

    |===
    |4 |5 |6
    |===

    .Second
    |===
    |7 |8 |9
    |===
  "#},
  contains: r#"<figcaption>Table 2. Second</figcaption>"#
);

assert_html!(
  custom_table_caption,
  adoc! {r#"
    [caption="So wow: "]
    .My Title
    |===
    |a | b
    |===
  "#},
  contains: r#"<figcaption>So wow: My Title</figcaption>"#
);

assert_html!(
  table_multiple_attr_lists,
  adoc! {r#"
    [[custom-id]]
    .My Title
    [caption="So wow: "]
    |===
    |a | b
    |===
  "#},
  contains:
  r#"<figure id="custom-id" class="table-block">"#,
  r#"<figcaption>So wow: My Title</figcaption>"#,
);

assert_html!(
  empty_captions_disables_numbered,
  adoc! {r#"
    [caption=""]
    .No caption
    |===
    |a | b
    |===
  "#},
  contains: r#"<figcaption>No caption</figcaption>"#
);

assert_html!(
  header_cell_in_footer,
  adoc! {r#"
    [cols="1h,1s,1e",options="footer"]
    |===
    |a | b | c
    |===
  "#},
  contains: r#"<th class="halign-left valign-top">a</th>"#,
);
