use test_utils::*;

assert_html!(
  basic_thematic_break,
  adoc! {r#"
    foo

    '''

    bar
  "#},
  html! {r#"
    <p>foo</p>
    <hr>
    <p>bar</p>
  "#}
);

assert_html!(
  thematic_break_w_attrs,
  adoc! {r#"
    foo

    [.fancy]
    '''
    bar
  "#},
  html! {r#"
    <p>foo</p>
    <hr class="fancy">
    <p>bar</p>
  "#}
);

assert_html!(
  basic_page_break,
  adoc! {r#"
    foo

    <<<

    bar
  "#},
  html! {r#"
    <p>foo</p>
    <div role="doc-pagebreak" style="page-break-after: always;"></div>
    <p>bar</p>
  "#}
);
