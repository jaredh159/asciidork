use test_utils::*;

assert_html!(
  collapsible_delimited,
  adoc! {r#"
    [%collapsible]
    ====
    inner content
    ====
  "#},
  html! {r#"
    <details>
      <summary class="title">Details</summary>
      <div class="content">
        <div class="paragraph"><p>inner content</p></div>
      </div>
    </details>
  "#}
);

assert_html!(
  collapsible_paragraph,
  adoc! {r#"
    [example%collapsible]
    inner content
  "#},
  html! {r#"
    <details>
      <summary class="title">Details</summary>
      <div class="content">
        inner content
      </div>
    </details>
  "#}
);

assert_html!(
  collapsible_custom_title_an_open,
  adoc! {r#"
    .Custom Title
    [%collapsible%open]
    ====
    inner content
    ====
  "#},
  html! {r#"
    <details open>
      <summary class="title">Custom Title</summary>
      <div class="content">
        <div class="paragraph"><p>inner content</p></div>
      </div>
    </details>
  "#}
);
