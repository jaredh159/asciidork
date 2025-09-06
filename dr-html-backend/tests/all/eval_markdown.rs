use test_utils::*;

assert_html!(
  markdown_headings,
  adoc! {r#"
    # Document Title (Level 0)

    ## Section Level 1

    ### Section Level 2

    #### Section Level 3

    ##### Section Level 4

    ###### Section Level 5
  "#},
  html! {r#"
    <div class="sect1">
      <h2 id="_section_level_1">Section Level 1</h2>
      <div class="sectionbody">
        <div class="sect2">
          <h3 id="_section_level_2">Section Level 2</h3>
          <div class="sect3">
            <h4 id="_section_level_3">Section Level 3</h4>
            <div class="sect4">
              <h5 id="_section_level_4">Section Level 4</h5>
              <div class="sect5">
                <h6 id="_section_level_5">Section Level 5</h6>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
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
