use asciidork_core::JobSettings;
use test_utils::*;

assert_html!(
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

assert_html!(
  supports_arbitrary_len_delims,
  adoc! {r#"
    |=====
    |a | b
    |c | d
    |=====
  "#},
  contains: "<p class=\"tableblock\">a</p>"
);

assert_html!(
  table_cell_doctype,
  adoc! {r#"
    |===
    a|
    :doctype: inline

    content
    |===
  "#},
  contains: r#"<div class="content">content</div>"#
);

assert_html!(
  table_cell_resets_doctype_preserves_other_attrs,
  adoc! {r#"
    :doctype: book
    :foo: bar

    |===
    a| doctype={doctype} foo={foo}
    |===
  "#},
  contains: "doctype=article foo=bar"
);

assert_html!(
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

assert_html!(
  table_attrs,
  adoc! {r#"
    :table-frame: sides
    :table-grid: cols
    :table-stripes: hover

    |===
    |a | b
    |===
  "#},
  contains:
    r#"<table class="tableblock frame-sides grid-cols stretch stripes-hover">"#
);

assert_html!(
  table_attrs_override,
  adoc! {r#"
    :table-frame: sides
    :table-grid: cols
    :table-stripes: hover

    [.custom,frame=ends,grid=all,stripes=odd]
    |===
    |a | b
    |===
  "#},
  contains:
    r#"<table class="tableblock frame-ends grid-all stretch stripes-odd custom">"#
);

assert_html!(
  topbot_to_frame_ends,
  adoc! {r#"
    [frame=topbot]
    |===
    |A |B
    |===
  "#},
  contains:
    r#"<table class="tableblock frame-ends grid-all stretch">"#
);

assert_html!(
  topbot_doc_attr_to_frame_ends,
  adoc! {r#"
    :table-frame: topbot

    |===
    |A |B
    |===
  "#},
  contains:
    r#"<table class="tableblock frame-ends grid-all stretch">"#
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
    r#"<p class="tableblock">A<br> B</p>"#,
    r#"<p class="tableblock">C<br> D<br> E</p>"#,
);

assert_html!(
  comments_in_table,
  adoc! {r#"
    |===
    // x
    // y
    |a | b
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
      </tbody>
    </table>
  "#}
);

assert_html!(
  cell_content_paragraphs,
  adoc! {r#"
    |===
    |para
    wraps

    then after newlines
    |joined by blank
    {blank}
    attribute

    strips trailing newlines

    when splitting paragraphs
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
        <tr>
          <td class="tableblock halign-left valign-top">
            <p class="tableblock">joined by blank  attribute</p>
            <p class="tableblock">strips trailing newlines</p>
            <p class="tableblock">when splitting paragraphs</p>
          </td>
        </tr>
      </tbody>
    </table>
  "#}
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

assert_html!(
  table_float_class,
  adoc! {r#"
    [float=left]
    |===
    |a | b
    |===
  "#},
  contains: r#"<table class="tableblock frame-all grid-all stretch left">"#
);

assert_html!(
  width_100_to_stretch_class,
  adoc! {r#"
    [width=100%]
    |===
    |a | b
    |===
  "#},
  contains: r#"<table class="tableblock frame-all grid-all stretch">"#
);

assert_html!(
  table_stripes_class,
  adoc! {r#"
    [stripes=odd]
    |===
    |a | b
    |===
  "#},
  contains: r#"<table class="tableblock frame-all grid-all stretch stripes-odd">"#
);

assert_html!(
  custom_table_class,
  adoc! {r#"
    [.so-custom]
    |===
    |a | b
    |===
  "#},
  contains: r#"<table class="tableblock frame-all grid-all stretch so-custom">"#
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
  contains: r#"<caption class="title">Table 2. Second</caption>"#
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
  contains: r#"<caption class="title">So wow: My Title</caption>"#
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
  r#"<table id="custom-id" class="tableblock"#,
  r#"<caption class="title">So wow: My Title</caption>"#,
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
  contains: r#"<caption class="title">No caption</caption>"#
);

assert_html!(
  doc_attr_disables_table_captions,
  adoc! {r#"
    :!table-caption:

    .No caption
    |===
    |a | b
    |===
  "#},
  contains: r#"<caption class="title">No caption</caption>"#
);

assert_html!(
  subs_table_content,
  adoc! {r#"
    :show_title: Cool new show

    |===
    |{show_title} |Coming soon!
    |===
  "#},
  contains: r#"<p class="tableblock">Cool new show</p></td>"#
);

assert_html!(
  autowidth_class,
  adoc! {r#"
    [%autowidth]
    |===
    |a | b
    |===
  "#},
  contains: r#"<table class="tableblock frame-all grid-all fit-content">"#
);

assert_html!(
  autowidth_class_w_spec,
  adoc! {r#"
    [%autowidth,cols=2*]
    |===
    |a | b
    |===
  "#},
  contains: "<colgroup><col><col></colgroup>" // <-- no width attrs
);

assert_html!(
  multibyte_separator,
  adoc! {r#"
    [separator="¦"]
    |===
    ¦one¦two
    |===
  "#},
  contains:
    r#"<p class="tableblock">one</p>"#,
    r#"<p class="tableblock">two</p>"#,
);

assert_html!(
  psv_tab_separated,
  adoc! {"
    [separator=\"\t\"]
    |===
    \tone\ttwo
    |===
  "},
  contains:
    r#"<p class="tableblock">one</p>"#,
    r#"<p class="tableblock">two</p>"#,
);

assert_html!(
  preserves_slash_not_escaping_delim,
  adoc! {r#"
    |===
    a|
    ----
    slash preserved \
    ----
    |===
  "#},
  contains: r#"<pre>slash preserved \</pre>"#
);

assert_html!(
  header_cell_in_footer,
  adoc! {r#"
    [cols="1h,1s,1e",options="footer"]
    |===
    |a | b | c
    |===
  "#},
  contains: r#"<th class="tableblock halign-left valign-top"><p class="tableblock">a</p></th>"#,
);

assert_html!(
  drops_cell_w_too_many_cols,
  |s: &mut JobSettings| s.strict = false,
  adoc! {r#"
    [cols=2*]
    |===
    3+|A
    |B | C
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
            <p class="tableblock">B</p>
          </td>
          <td class="tableblock halign-left valign-top">
            <p class="tableblock">C</p>
          </td>
        </tr>
      </tbody>
    </table>
  "#}
);

assert_html!(
  no_implicit_header_if_first_cell_multiline,
  adoc! {r#"
    [cols=2*]
    |===
    |A1

    A1 continued|B1

    |A2
    |B2
    |===
  "#},
  contains:
    r#"</colgroup><tbody><tr>"#, // <-- no thead
    r#"<p class="tableblock">A1</p><p class="tableblock">A1 continued</p>"#
);
