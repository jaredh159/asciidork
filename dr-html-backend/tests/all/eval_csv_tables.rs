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
    <table class="tableblock frame-all grid-all stretch">
      <colgroup><col style="width: 50%;"><col style="width: 50%;"></colgroup>
      <tbody>
        <tr>
          <td class="tableblock halign-left valign-top">
            <p class="tableblock">A1</p>
          </td>
          <td class="tableblock halign-left valign-top"></td>
        </tr>
        <tr>
          <td class="tableblock halign-left valign-top">
            <p class="tableblock">B1</p>
          </td>
          <td class="tableblock halign-left valign-top">
            <p class="tableblock">B2</p>
          </td>
        </tr>
      </tbody>
    </table>
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
    <table class="tableblock frame-all grid-all stretch">
      <colgroup>
        <col style="width: 33.3333%;">
        <col style="width: 33.3333%;">
        <col style="width: 33.3333%;">
      </colgroup>
      <thead>
        <tr>
          <th class="tableblock halign-left valign-top">element</th>
          <th class="tableblock halign-left valign-top">description</th>
          <th class="tableblock halign-left valign-top">example</th>
        </tr>
      </thead>
      <tbody>
        <tr>
          <td class="tableblock halign-left valign-top">
            <p class="tableblock">foo</p>
          </td>
          <td class="tableblock halign-left valign-top">
            <p class="tableblock">bar</p>
          </td>
          <td class="tableblock halign-left valign-top">
            <div class="content">
              <div class="paragraph">
                <p><em>baz</em></p>
              </div>
            </div>
          </td>
        </tr>
      </tbody>
    </table>
  "#}
);

assert_html!(
  csv_strips_adoc_cell_whitespace,
  adoc! {r#"
    [cols="1,1,1a",separator=;]
    ,===
    element;description;example

    paragraph;contiguous lines of words and phrases;"
      one sentence, one line
      "
    ,===
  "#},
  contains:
    r#"<p class="tableblock">paragraph</p>"#,
    r#"<p class="tableblock">contiguous lines of words and phrases</p>"#,
    r#"<div class="paragraph"><p>one sentence, one line</p>"#,
);

assert_html!(
  csv_preserving_newlines,
  adoc! {r#"
    [cols="1,1,1l"]
    ,===
    "A
    B
    C","one

    two

    three","do

    re

    me"
    ,===
  "#},
  contains:
    r#"<col style="width: 33.3333%;"><col style="width: 33.3333%;">"#,
    r#"<p class="tableblock">A B C</p>"#,
    r#"<p class="tableblock">one</p>"#,
    r#"<p class="tableblock">two</p>"#,
    r#"<p class="tableblock">three</p>"#,
    "<div class=\"literal\"><pre>do\n\nre\n\nme</pre></div>",
);

assert_html!(
  csv_tab_separated,
  adoc! {"
    [separator=\"\t\"]
    ,===
    a\tb
    1\t2
    ,===
  "},
  contains:
    r#"<p class="tableblock">a</p>"#,
    r#"<p class="tableblock">2</p>"#,
);

assert_html!(
  tsv_tab_separated,
  adoc! {"
    [format=tsv]
    ,===
    a\tb
    1\t2
    ,===
  "},
  contains:
    r#"<p class="tableblock">a</p>"#,
    r#"<p class="tableblock">2</p>"#,
);

assert_html!(
  csv_custom_separator,
  adoc! {"
    [format=csv,separator=;]
    |===
    a;b
    1;2
    |===
  "},
  contains:
    r#"<p class="tableblock">a</p>"#,
    r#"<p class="tableblock">2</p>"#,
);

assert_html!(
  csv_multibyte_separator,
  adoc! {"
    [format=csv,separator=¦]
    |===
    a¦b¦c
    1¦2¦3
    |===
  "},
  contains:
    r#"<p class="tableblock">a</p>"#,
    r#"<p class="tableblock">c</p>"#,
    r#"<p class="tableblock">2</p>"#,
);

assert_html!(
  complex_csv,
  adoc! {r#"
    [%header,format="csv"]
    |===
    Year,Make,Model,Description,Price
    1997," Ford ",E350,"ac, abs, moon",3000.00
    1999,Chevy,"Venture ""Extended Edition""","",4900.00
    1999,Chevy,"Venture ""Extended Edition, Very Large""",,5000.00
    1996,Jeep,Grand Cherokee,"MUST SELL!
    air, moon roof, loaded",4799.00
    2000,Toyota,Tundra,"""This one's gonna to blow you're socks off,"" per the sticker",10000.00
    2000,Toyota,Tundra,"Check it, ""this one's gonna to blow you're socks off"", per the sticker",10000.00
    |===
  "#},
  html! { r#"
    <table class="tableblock frame-all grid-all stretch">
      <colgroup>
        <col style="width: 20%;">
        <col style="width: 20%;">
        <col style="width: 20%;">
        <col style="width: 20%;">
        <col style="width: 20%;">
      </colgroup>
      <thead>
        <tr>
          <th class="tableblock halign-left valign-top">Year</th>
          <th class="tableblock halign-left valign-top">Make</th>
          <th class="tableblock halign-left valign-top">Model</th>
          <th class="tableblock halign-left valign-top">Description</th>
          <th class="tableblock halign-left valign-top">Price</th>
        </tr>
      </thead>
      <tbody>
        <tr>
          <td class="tableblock halign-left valign-top">
            <p class="tableblock">1997</p>
          </td>
          <td class="tableblock halign-left valign-top">
            <p class="tableblock">Ford</p>
          </td>
          <td class="tableblock halign-left valign-top">
            <p class="tableblock">E350</p>
          </td>
          <td class="tableblock halign-left valign-top">
            <p class="tableblock">ac, abs, moon</p>
          </td>
          <td class="tableblock halign-left valign-top">
            <p class="tableblock">3000.00</p>
          </td>
        </tr>
        <tr>
          <td class="tableblock halign-left valign-top">
            <p class="tableblock">1999</p>
          </td>
          <td class="tableblock halign-left valign-top">
            <p class="tableblock">Chevy</p>
          </td>
          <td class="tableblock halign-left valign-top">
            <p class="tableblock">Venture "Extended Edition"</p>
          </td>
          <td class="tableblock halign-left valign-top">
          </td>
          <td class="tableblock halign-left valign-top">
            <p class="tableblock">4900.00</p>
          </td>
        </tr>
        <tr>
          <td class="tableblock halign-left valign-top">
            <p class="tableblock">1999</p>
          </td>
          <td class="tableblock halign-left valign-top">
            <p class="tableblock">Chevy</p>
          </td>
          <td class="tableblock halign-left valign-top">
            <p class="tableblock">Venture "Extended Edition, Very Large"</p>
          </td>
          <td class="tableblock halign-left valign-top"></td>
          <td class="tableblock halign-left valign-top">
            <p class="tableblock">5000.00</p>
          </td>
        </tr>
        <tr>
          <td class="tableblock halign-left valign-top">
            <p class="tableblock">1996</p>
          </td>
          <td class="tableblock halign-left valign-top">
            <p class="tableblock">Jeep</p>
          </td>
          <td class="tableblock halign-left valign-top">
            <p class="tableblock">Grand Cherokee</p>
          </td>
          <td class="tableblock halign-left valign-top">
           <p class="tableblock">MUST SELL! air, moon roof, loaded
          </p></td>
          <td class="tableblock halign-left valign-top">
            <p class="tableblock">4799.00</p>
          </td>
        </tr>
        <tr>
          <td class="tableblock halign-left valign-top">
            <p class="tableblock">2000</p>
          </td>
          <td class="tableblock halign-left valign-top">
            <p class="tableblock">Toyota</p>
          </td>
          <td class="tableblock halign-left valign-top">
            <p class="tableblock">Tundra</p>
          </td>
          <td class="tableblock halign-left valign-top">
            <p class="tableblock">"This one&#8217;s gonna to blow you&#8217;re socks off," per the sticker</p>
          </td>
          <td class="tableblock halign-left valign-top">
            <p class="tableblock">10000.00</p>
          </td>
        </tr>
        <tr>
          <td class="tableblock halign-left valign-top">
            <p class="tableblock">2000</p>
          </td>
          <td class="tableblock halign-left valign-top">
            <p class="tableblock">Toyota</p>
          </td>
          <td class="tableblock halign-left valign-top">
            <p class="tableblock">Tundra</p>
          </td>
          <td class="tableblock halign-left valign-top">
            <p class="tableblock">Check it, "this one&#8217;s gonna to blow you&#8217;re socks off", per the sticker</p>
          </td>
          <td class="tableblock halign-left valign-top">
            <p class="tableblock">10000.00</p>
          </td>
        </tr>
      </tbody>
    </table>
  "#}
);
