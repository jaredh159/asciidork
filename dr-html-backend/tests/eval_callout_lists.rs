use test_utils::{adoc, html, raw_html};

mod helpers;

test_eval!(
  basic_callout_list,
  adoc! {r#"
    [source,ruby]
    ----
    require 'asciidoctor' # <2>
    doc = Asciidoctor::Document.new('Hello, World!') # <3>
    puts doc.convert # <1>
    ----
    <1> Describe the first line
    <2> Describe the second line
    <3> Describe the third line
  "#},
  wrap_source_appending(
    "ruby",
    raw_html! {r#"
      require 'asciidoctor' # <b class="conum">(2)</b>
      doc = Asciidoctor::Document.new('Hello, World!') # <b class="conum">(3)</b>
      puts doc.convert # <b class="conum">(1)</b>
    "#},
    html! {r#"
      <div class="colist arabic">
        <ol>
          <li><p>Describe the first line</p></li>
          <li><p>Describe the second line</p></li>
          <li><p>Describe the third line</p></li>
        </ol>
      </div>
    "#}
  )
);

test_eval!(
  basic_callout_list_w_icons_font,
  adoc! {r#"
    :icons: font

    [source,ruby]
    ----
    require 'asciidoctor' # <2>
    puts doc.convert # <3>
    puts doc.convert # <1>
    ----
    <1> Describe the first line
    <2> Describe the second line
    <3> Describe the third line
  "#},
  wrap_source_appending(
    "ruby",
    raw_html! {r#"
      require 'asciidoctor' <i class="conum" data-value="2"></i><b>(2)</b>
      puts doc.convert <i class="conum" data-value="3"></i><b>(3)</b>
      puts doc.convert <i class="conum" data-value="1"></i><b>(1)</b>
    "#},
    html! {r#"
      <div class="colist arabic">
        <table>
          <tr>
            <td><i class="conum" data-value="1"></i><b>(1)</b></td>
            <td>Describe the first line</td>
          </td>
          <tr>
            <td><i class="conum" data-value="2"></i><b>(2)</b></td>
            <td>Describe the second line</td>
          </td>
          <tr>
            <td><i class="conum" data-value="3"></i><b>(3)</b></td>
            <td>Describe the third line</td>
          </td>
        </table>
      </div>
    "#}
  )
);

test_eval!(
  basic_callout_list_w_icons_not_font,
  adoc! {r#"
    :icons:

    [source,ruby]
    ----
    require 'asciidoctor' # <2>
    puts doc.convert # <3>
    puts doc.convert # <1>
    ----
    <1> Describe the first line
    <2> Describe the second line
    <3> Describe the third line
  "#},
  wrap_source_appending(
    "ruby",
    raw_html! {r#"
      require 'asciidoctor' # <img src="./images/icons/callouts/2.png" alt="2">
      puts doc.convert # <img src="./images/icons/callouts/3.png" alt="3">
      puts doc.convert # <img src="./images/icons/callouts/1.png" alt="1">
    "#},
    html! {r#"
      <div class="colist arabic">
        <table>
          <tr>
            <td><img src="./images/icons/callouts/1.png" alt="1"></td>
            <td>Describe the first line</td>
          </td>
          <tr>
            <td><img src="./images/icons/callouts/2.png" alt="2"></td>
            <td>Describe the second line</td>
          </td>
          <tr>
            <td><img src="./images/icons/callouts/3.png" alt="3"></td>
            <td>Describe the third line</td>
          </td>
        </table>
      </div>
    "#}
  )
);

// helpers

fn wrap_listing(inner: &str) -> String {
  format!(
    r#"<div class="listingblock"><div class="content">{}</div></div>"#,
    inner.trim(),
  )
}

fn wrap_source_appending(lang: &str, inner: &str, rest: String) -> String {
  let listing = wrap_listing(&format!(
    r#"<pre class="highlight"><code class="language-{lang}" data-lang="{lang}">{}</code></pre>"#,
    inner.trim(),
  ));
  format!("{}{}", listing, rest)
}
