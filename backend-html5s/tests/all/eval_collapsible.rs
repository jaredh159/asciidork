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
    <details><div class="content"><p>inner content</p></div></details>
  "#}
);

assert_html!(
  collapsible_paragraph,
  adoc! {r#"
    [example%collapsible]
    inner content
  "#},
  html! {r#"
    <details><div class="content">inner content</div></details>
  "#}
);

assert_html!(
  collapsible_custom_title_and_open,
  adoc! {r#"
    .Custom Title
    [%collapsible%open]
    ====
    inner content
    ====
  "#},
  html! {r#"
    <details open><summary>Custom Title</summary><div class="content"><p>inner content</p></div></details>
  "#}
);
