use test_utils::{adoc, html};
mod helpers;

test_eval!(
  single_simple_section,
  adoc! {r#"
    == Section 1

    Content.
  "#},
  html! {r#"
    <div class="sect1">
      <h2 id="_section_1">Section 1</h2>
      <div class="sectionbody">
        <div class="paragraph">
          <p>Content.</p>
        </div>
      </div>
    </div>
  "#}
);

test_eval!(
  single_2_simple_sections,
  adoc! {r#"
    == Section 1

    Content.

    == Section 2

    Content.
  "#},
  html! {r#"
    <div class="sect1">
      <h2 id="_section_1">Section 1</h2>
      <div class="sectionbody">
        <div class="paragraph">
          <p>Content.</p>
        </div>
      </div>
    </div>
    <div class="sect1">
      <h2 id="_section_2">Section 2</h2>
      <div class="sectionbody">
        <div class="paragraph">
          <p>Content.</p>
        </div>
      </div>
    </div>
  "#}
);
