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
    <div id="preamble">
      <div class="sectionbody">
        <div class="quoteblock abstract">
          <div class="title">Abstract</div>
          <blockquote>Pithy quote</blockquote>
        </div>
      </div>
    </div>
    <div class="sect1">
      <h2 id="_first_section">First Section</h2>
      <div class="sectionbody"></div>
    </div>
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
  contains: &html! {r#"
    <div class="quoteblock abstract">
      <div class="title">My Abstract</div>
      <blockquote>
        <div class="paragraph">
          <p>This article is about stuff.</p>
        </div>
        <div class="paragraph">
          <p>And other stuff.</p>
        </div>
      </blockquote>
    </div>
  "#}
);
