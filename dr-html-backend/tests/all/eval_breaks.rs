use test_utils::*;

assert_html!(
  basic_thematic_break,
  adoc! {r#"
    foo

    '''

    bar
  "#},
  html! {r#"
    <div class="paragraph"><p>foo</p></div>
    <hr>
    <div class="paragraph"><p>bar</p></div>
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
    <div class="paragraph"><p>foo</p></div>
    <hr class="fancy">
    <div class="paragraph"><p>bar</p></div>
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
    <div class="paragraph"><p>foo</p></div>
    <div style="page-break-after: always;"></div>
    <div class="paragraph"><p>bar</p></div>
  "#}
);

assert_html!(
  markdown_thematic_break,
  adoc! {r#"
    foo

    ---

    bar

    ***

    baz

    - - -

    jim

    * * *
    
    jam
  "#},
  html! {r#"
    <div class="paragraph"><p>foo</p></div>
    <hr>
    <div class="paragraph"><p>bar</p></div>
    <hr>
    <div class="paragraph"><p>baz</p></div>
    <hr>
    <div class="paragraph"><p>jim</p></div>
    <hr>
    <div class="paragraph"><p>jam</p></div>
  "#}
);
