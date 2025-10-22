use test_utils::*;

assert_html!(
  abstract_block_style,
  adoc! {r#"
    = Document Title

    [abstract]
    .Abstract
    Pithy quote

    == First Section
  "#},
  html! {r#"
    <section id="preamble" aria-label="Preamble"><section class="quote-block abstract"><h6 class="block-title">Abstract</h6>
    <blockquote>Pithy quote</blockquote></section></section>
    <section class="doc-section level-1"><h2 id="_first_section">First Section</h2></section>
  "#}
);

assert_html!(
  abstract_block_style_open_block,
  adoc! {r#"
    = Article

    .My Abstract
    [abstract]
    --
    This article is about stuff.

    And other stuff.
    --

    == Section One

    content
  "#},
  html! {r#"
    <section id="preamble" aria-label="Preamble"><section class="quote-block abstract"><h6 class="block-title">My Abstract</h6>
    <blockquote><p>This article is about stuff.</p>
    <p>And other stuff.</p></blockquote></section></section>
    <section class="doc-section level-1"><h2 id="_section_one">Section One</h2><p>content</p></section>
  "#}
);
