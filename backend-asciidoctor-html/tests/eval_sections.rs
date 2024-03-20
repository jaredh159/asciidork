use test_utils::{adoc, html};
mod helpers;

test_eval!(
  single_simple_section,
  adoc! {r#"
    == Section 1

    Section Content.
  "#},
  html! {r#"
    <div class="sect1">
      <h2 id="_section_1">Section 1</h2>
      <div class="sectionbody">
        <div class="paragraph">
          <p>Section Content.</p>
        </div>
      </div>
    </div>
  "#}
);

test_eval!(
  customized_id_and_prefix,
  adoc! {r#"
    :idprefix: foo_
    :idseparator: -

    == Section 1
  "#},
  html! {r#"
    <div class="sect1">
      <h2 id="foo_section-1">Section 1</h2>
      <div class="sectionbody"></div>
    </div>
  "#}
);

test_eval!(
  section_w_preamble,
  adoc! {r#"
    Preamble

    == Section 1

    Section Content.
  "#},
  html! {r#"
    <div class="paragraph">
      <p>Preamble</p>
    </div>
    <div class="sect1">
      <h2 id="_section_1">Section 1</h2>
      <div class="sectionbody">
        <div class="paragraph">
          <p>Section Content.</p>
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

test_eval!(
  sect_ids_disabled,
  adoc! {r#"
    :sectids!:

    == Section 1

    Content.
  "#},
  html! {r#"
    <div class="sect1">
      <h2>Section 1</h2>
      <div class="sectionbody">
        <div class="paragraph">
          <p>Content.</p>
        </div>
      </div>
    </div>
  "#}
);

test_eval!(
  flip_flop_sectids,
  adoc! {r#"
    == ID generation on

    :!sectids:
    == ID generation off
    :sectids:

    == ID generation on again
  "#},
  html! {r#"
    <div class="sect1">
      <h2 id="_id_generation_on">ID generation on</h2>
      <div class="sectionbody"></div>
    </div>
    <div class="sect1">
      <h2>ID generation off</h2>
      <div class="sectionbody"></div>
    </div>
    <div class="sect1">
      <h2 id="_id_generation_on_again">ID generation on again</h2>
      <div class="sectionbody"></div>
    </div>
  "#}
);

test_eval!(
  explicit_ids,
  adoc! {r#"
    [#tigers-subspecies]
    == Subspecies of Tiger

    [id=longhand]
    == Chapter 2

    [[legacy]]
    == Chapter 3
  "#},
  html! {r#"
    <div class="sect1">
      <h2 id="tigers-subspecies">Subspecies of Tiger</h2>
      <div class="sectionbody"></div>
    </div>
    <div class="sect1">
      <h2 id="longhand">Chapter 2</h2>
      <div class="sectionbody"></div>
    </div>
    <div class="sect1">
      <h2 id="legacy">Chapter 3</h2>
      <div class="sectionbody"></div>
    </div>
  "#}
);

test_eval!(
  explicit_id_sequenced,
  adoc! {r#"
    :idseparator: -
    :idprefix:

    [#tigers-subspecies]
    == Subspecies of Tiger

    == Tigers Subspecies
  "#},
  html! {r#"
    <div class="sect1">
      <h2 id="tigers-subspecies">Subspecies of Tiger</h2>
      <div class="sectionbody"></div>
    </div>
    <div class="sect1">
      <h2 id="tigers-subspecies-2">Tigers Subspecies</h2>
      <div class="sectionbody"></div>
    </div>
  "#}
);
