use test_utils::*;

assert_html!(
  basic_dsv_table,
  adoc! {r#"
    :===
    a : b
    c : d\:e
    :===
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
            <p class="tableblock">d:e</p>
          </td>
        </tr>
      </tbody>
    </table>
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
    <table class="tableblock frame-all grid-all stretch">
      <colgroup>
        <col style="width: 25%;">
        <col style="width: 50%;">
        <col style="width: 25%;">
      </colgroup>
      <thead>
        <tr>
          <th class="tableblock halign-left valign-top">one</th>
          <th class="tableblock halign-left valign-top">two</th>
          <th class="tableblock halign-left valign-top">three</th>
        </tr>
      </thead>
      <tfoot>
        <tr>
          <td class="tableblock halign-left valign-top">
            <div class="content">
              <div class="paragraph"><p>four</p></div>
            </div>
          </td>
          <td class="tableblock halign-left valign-top">
            <p class="tableblock"><em>five</em></p>
          </td>
          <td class="tableblock halign-left valign-top"></td>
        </tr>
      </tfoot>
    </table>
  "#}
);

assert_html!(
  dsv_tab_separated,
  adoc! {"
    [separator=\"\t\"]
    :===
    one\ttwo
    :===
  "},
  contains:
    r#"<p class="tableblock">one</p>"#,
    r#"<p class="tableblock">two</p>"#,
);

assert_html!(
  multibyte_separator,
  adoc! {r#"
    [separator="¦"]
    :===
    one¦two
    :===
  "#},
  contains:
    r#"<p class="tableblock">one</p>"#,
    r#"<p class="tableblock">two</p>"#,
);
