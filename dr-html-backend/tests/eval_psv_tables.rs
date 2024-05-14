use test_utils::*;

mod helpers;

test_eval!(
  basic_table,
  adoc! {r#"
    |===
    |a | b
    |c | d
    |===
  "#},
  html! {r#"
    <table class="tableblock frame-all grid-all stretch">
      <colgroup>
        <col style="width: 50%;">
        <col style="width: 50%;">
      </colgroup>
      <tbody>
        <tr>
          <td class="tableblock halign-left valign-top">
            <p class="tableblock">a</p>
          </td>
          <td class="tableblock halign-left valign-top">
            <p class="tableblock">b</p>
          </td>
        </tr>
        <tr>
          <td class="tableblock halign-left valign-top">
            <p class="tableblock">c</p>
          </td>
          <td class="tableblock halign-left valign-top">
            <p class="tableblock">d</p>
          </td>
        </tr>
      </tbody>
    </table>
  "#}
);

test_eval!(
  complex_table,
  adoc! {r#"
    .Table Title
    [cols="25%,~",width=50%,%footer,frame=ends]
    |===
    |h1 | *h2*

    >e|a ^h| b \| b2
    .>l|c .^m| d\|
    2*s| ef
    |foot1 | foot2
    |===
  "#},
  html! {r#"
    <table class="tableblock frame-ends grid-all" style="width: 50%;">
      <caption class="title">Table 1. Table Title</caption>
      <colgroup>
        <col style="width: 25%;">
        <col>
      </colgroup>
      <thead>
        <tr>
          <th class="tableblock halign-left valign-top">h1</th>
          <th class="tableblock halign-left valign-top"><strong>h2</strong></th>
        </tr>
      </thead>
      <tbody>
        <tr>
          <td class="tableblock halign-right valign-top">
            <p class="tableblock"><em>a</em></p>
          </td>
          <th class="tableblock halign-center valign-top">
            <p class="tableblock">b | b2</p>
          </th>
        </tr>
        <tr>
          <td class="tableblock halign-left valign-bottom">
            <div class="literal"><pre>c</pre></div>
          </td>
          <td class="tableblock halign-left valign-middle">
            <p class="tableblock"><code>d|</code></p>
          </td>
        </tr>
        <tr>
          <td class="tableblock halign-left valign-top">
            <p class="tableblock"><strong>ef</strong></p>
          </td>
          <td class="tableblock halign-left valign-top">
            <p class="tableblock"><strong>ef</strong></p>
          </td>
        </tr>
      </tbody>
      <tfoot>
        <tr>
          <td class="tableblock halign-left valign-top">
            <p class="tableblock">foot1</p>
          </td>
          <td class="tableblock halign-left valign-top">
            <p class="tableblock">foot2</p>
          </td>
        </tr>
      </tfoot>
    </table>
  "#}
);

test_eval!(
  cell_content_paragraphs,
  adoc! {r#"
    |===
    |para
    wraps

    then after newlines
    |===
  "#},
  html! {r#"
    <table class="tableblock frame-all grid-all stretch">
      <colgroup><col style="width: 100%;"></colgroup>
      <tbody>
        <tr>
          <td class="tableblock halign-left valign-top">
            <p class="tableblock">para wraps</p>
            <p class="tableblock">then after newlines</p>
          </td>
        </tr>
      </tbody>
    </table>
  "#}
);

test_eval!(
  formatting_in_header_row,
  adoc! {r#"
    [cols="2*m"]
    |===
    | _foo_ | *bar*

    | a | b
    |===
  "#},
  html! {r#"
    <table class="tableblock frame-all grid-all stretch">
      <colgroup><col style="width: 50%;"><col style="width: 50%;"></colgroup>
      <thead>
        <tr>
          <th class="tableblock halign-left valign-top"><em>foo</em></th>
          <th class="tableblock halign-left valign-top"><strong>bar</strong></th>
        </tr>
      </thead>
      <tbody>
        <tr>
          <td class="tableblock halign-left valign-top">
            <p class="tableblock"><code>a</code></p>
          </td>
          <td class="tableblock halign-left valign-top">
            <p class="tableblock"><code>b</code></p>
          </td>
        </tr>
      </tbody>
    </table>
  "#}
);

test_eval!(
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
    <table class="tableblock frame-all grid-all stretch">
      <colgroup><col style="width: 50%;"><col style="width: 50%;"></colgroup>
      <tbody>
        <tr>
          <td class="tableblock halign-left valign-top">
            <p class="tableblock"><strong><em>strong</em></strong></p>
          </td>
          <td class="tableblock halign-left valign-top">
            <p class="tableblock"><em><strong>emphasis</strong></em></p>
          </td>
        </tr>
        <tr>
          <td class="tableblock halign-left valign-top">
            <p class="tableblock"><strong>strong</strong></p>
          </td>
          <td class="tableblock halign-left valign-top">
            <p class="tableblock"><em>emphasis</em></p>
          </td>
        </tr>
      </tbody>
    </table>
  "#}
);

test_eval!(
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
    <table class="tableblock frame-all grid-all" style="width: 25%;">
      <colgroup>
        <col style="width: 25%;">
        <col style="width: 25%;">
        <col style="width: 25%;">
        <col style="width: 25%;">
      </colgroup>
      <tbody>
        <tr>
          <td class="tableblock halign-left valign-top">
            <p class="tableblock"><em>1</em></p>
          </td>
          <td class="tableblock halign-right valign-top">
            <p class="tableblock"><strong>2</strong></p>
          </td>
          <td class="tableblock halign-center valign-top">
            <p class="tableblock">3</p>
          </td>
          <td class="tableblock halign-right valign-top">
            <p class="tableblock"><strong>4</strong></p>
          </td>
        </tr>
        <tr>
          <td class="tableblock halign-center valign-top">
            <p class="tableblock"><em>5</em></p>
          </td>
          <td class="tableblock halign-center valign-middle" colspan="2" rowspan="2">
            <p class="tableblock"><code>6</code></p>
          </td>
          <td class="tableblock halign-left valign-bottom" rowspan="3">
            <p class="tableblock"><code>7</code></p>
          </td>
        </tr>
        <tr>
          <td class="tableblock halign-center valign-top">
            <p class="tableblock"><em>8</em></p>
          </td>
        </tr>
        <tr>
          <td class="tableblock halign-left valign-top">
            <p class="tableblock">9</p>
          </td>
          <td class="tableblock halign-right valign-top" colspan="2">
            <p class="tableblock"><code>10</code></p>
          </td>
        </tr>
      </tbody>
    </table>
  "#}
);

test_eval!(
  table_float_class,
  adoc! {r#"
    [float=left]
    |===
    |a | b
    |===
  "#},
  html_contains: r#"<table class="tableblock frame-all grid-all stretch left">"#
);

test_eval!(
  width_100_to_stretch_class,
  adoc! {r#"
    [width=100%]
    |===
    |a | b
    |===
  "#},
  html_contains: r#"<table class="tableblock frame-all grid-all stretch">"#
);

test_eval!(
  table_stripes_class,
  adoc! {r#"
    [stripes=odd]
    |===
    |a | b
    |===
  "#},
  html_contains: r#"<table class="tableblock frame-all grid-all stretch stripes-odd">"#
);

test_eval!(
  custom_table_class,
  adoc! {r#"
    [.so-custom]
    |===
    |a | b
    |===
  "#},
  html_contains: r#"<table class="tableblock frame-all grid-all stretch so-custom">"#
);

test_eval!(
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
  html_contains: r#"<caption class="title">Table 2. Second</caption>"#
);

test_eval!(
  custom_table_caption,
  adoc! {r#"
    [caption="So wow: "]
    .My Title
    |===
    |a | b
    |===
  "#},
  html_contains: r#"<caption class="title">So wow: My Title</caption>"#
);

test_eval!(
  empty_captions_disables_numbered,
  adoc! {r#"
    [caption=""]
    .No caption
    |===
    |a | b
    |===
  "#},
  html_contains: r#"<caption class="title">No caption</caption>"#
);

test_eval!(
  doc_attr_disables_table_captions,
  adoc! {r#"
    :!table-caption:

    .No caption
    |===
    |a | b
    |===
  "#},
  html_contains: r#"<caption class="title">No caption</caption>"#
);

test_eval!(
  subs_table_content,
  adoc! {r#"
    :show_title: Cool new show
    |===
    |{show_title} |Coming soon...
    |===
  "#},
  html_contains: r#"<p class="tableblock">Cool new show</p></td>"#
);

test_eval!(
  autowidth_class,
  adoc! {r#"
    [%autowidth]
    |===
    |a | b
    |===
  "#},
  html_contains: r#"<table class="tableblock frame-all grid-all fit-content">"#
);
