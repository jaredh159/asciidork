use crate::assert_html;
use asciidork_meta::{JobAttr, JobSettings};
use asciidork_parser::prelude::*;
use test_utils::*;

assert_html!(
  basic_asciidoc_content,
  adoc! {r#"
    |===
    a|AsciiDoc table cell
    a|--
    NOTE: content

    content
    --
    |===
  "#},
  html! {r#"
    <table class="tableblock frame-all grid-all stretch">
      <colgroup><col style="width: 100%;"></colgroup>
      <tbody>
        <tr>
          <td class="tableblock halign-left valign-top">
            <div class="content">
              <div class="paragraph">
                <p>AsciiDoc table cell</p>
              </div>
            </div>
          </td>
        </tr>
        <tr>
          <td class="tableblock halign-left valign-top">
            <div class="content">
              <div class="openblock">
                <div class="content">
                  <div class="admonitionblock note">
                    <table>
                      <tr>
                        <td class="icon"><div class="title">Note</div></td>
                        <td class="content">content</td>
                      </tr>
                    </table>
                  </div>
                  <div class="paragraph">
                    <p>content</p>
                  </div>
                </div>
              </div>
            </div>
          </td>
        </tr>
      </tbody>
    </table>
  "#}
);

assert_html!(
  adoc_cell_can_set_attr_when_parent_has_unset,
  adoc! {r#"
    :!sectids:

    == No ID

    |===
    a|

    == No ID

    :sectids:

    == Has ID
    |===
  "#},
  html! {r#"
    <div class="sect1">
      <h2>No ID</h2>
      <div class="sectionbody">
        <table class="tableblock frame-all grid-all stretch">
          <colgroup><col style="width: 100%;"></colgroup>
          <tbody>
            <tr>
              <td class="tableblock halign-left valign-top">
                <div class="content">
                  <div class="sect1">
                    <h2>No ID</h2>
                    <div class="sectionbody"></div>
                  </div>
                  <div class="sect1">
                    <h2 id="_has_id">Has ID</h2>
                    <div class="sectionbody"></div>
                  </div>
                </div>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>
  "#}
);

assert_html!(
  override_unset_showtitle_from_parent,
  adoc! {r#"
    = Document Title
    :!showtitle:

    |===
    a|
    = Nested Document Title
    :showtitle:

    content
    |===
  "#},
  html! {r#"
    <table class="tableblock frame-all grid-all stretch">
      <colgroup><col style="width: 100%;"></colgroup>
      <tbody>
        <tr>
          <td class="tableblock halign-left valign-top">
            <div class="content">
              <h1>Nested Document Title</h1>
              <div class="paragraph"><p>content</p></div>
            </div>
          </td>
        </tr>
      </tbody>
    </table>
  "#}
);

assert_html!(
  override_set_showtitle_from_parent,
  adoc! {r#"
    = Document Title
    :showtitle:

    |===
    a|
    = Nested Document Title
    :!showtitle:

    content
    |===
  "#},
  contains: r#"<div class="content"><div class="paragraph"><p>content"#
);

assert_html!(
  preserves_newlines_if_cell_starts_newline,
  adoc! {r#"
    |===
    a|
     $ command
    a| paragraph
    |===
  "#},
  html! {r#"
    <table class="tableblock frame-all grid-all stretch">
      <colgroup><col style="width: 100%;"></colgroup>
      <tbody>
        <tr>
          <td class="tableblock halign-left valign-top">
            <div class="content">
              <div class="literalblock">
                <div class="content"><pre>$ command</pre></div>
              </div>
            </div>
          </td>
        </tr>
        <tr>
          <td class="tableblock halign-left valign-top">
            <div class="content">
              <div class="paragraph"><p>paragraph</p></div>
            </div>
          </td>
        </tr>
      </tbody>
    </table>
  "#}
);

assert_html!(
  basic_table_nesting,
  adoc! {r#"
    |===
    a|!===
    !1 !2
    !===
    |===
  "#},
  html! {r#"
    <table class="tableblock frame-all grid-all stretch">
      <colgroup><col style="width: 100%;"></colgroup>
      <tbody>
        <tr>
          <td class="tableblock halign-left valign-top">
            <div class="content">
              <table class="tableblock frame-all grid-all stretch">
                <colgroup><col style="width: 50%;"><col style="width: 50%;"></colgroup>
                <tbody>
                  <tr>
                    <td class="tableblock halign-left valign-top">
                      <p class="tableblock">1</p>
                    </td>
                    <td class="tableblock halign-left valign-top">
                      <p class="tableblock">2</p>
                    </td>
                  </tr>
                </tbody>
              </table>
            </div>
          </td>
        </tr>
      </tbody>
    </table>
  "#}
);

assert_html!(
  nested_table_with_custom_separator,
  adoc! {r#"
    |===
    a|
    [separator=;]
    !===
    ;1 ;2
    !===
    |===
  "#},
  contains:
   r#"<p class="tableblock">1</p></td>"#,
   r#"<p class="tableblock">2</p></td>"#,
);

assert_html!(
  toc_in_adoc_cell,
  adoc! {r#"
    = Document Title

    == Section A

    |===
    a|
    = Subdocument Title
    :toc:

    == Subdocument Section A

    content
    |===
  "#},
  contains:
    r#"<td class="tableblock halign-left valign-top"><div class="content"><div id="toc" class="toc">"#
);

assert_html!(
  // https://github.com/asciidoctor/asciidoctor/issues/4017#issuecomment-821915135
  toc_in_adoc_cell_even_if_parent_hard_unsets,
  |s: &mut JobSettings| {
    s.job_attrs.insert_unchecked("toc", JobAttr::readonly(false));
  },
  adoc! {r#"
    = Document Title

    == Section A

    |===
    a|
    = Subdocument Title
    :toc:

    == Subdocument Section A

    content
    |===
  "#},
  contains: r#"<div id="toctitle">Table of Contents</div>"#
);

assert_html!(
  anchor_starting_explicit_header_cell,
  adoc! {r#"
    [%header,cols=1a]
    |===
    |[[foo,Foo]]* not AsciiDoc
    | AsciiDoc
    |===

    See <<foo>>.
  "#},
  html! { r##"
    <table class="tableblock frame-all grid-all stretch">
      <colgroup><col style="width: 100%;"></colgroup>
      <thead>
        <tr>
          <th class="tableblock halign-left valign-top">
            <a id="foo"></a>* not AsciiDoc
          </th>
        </tr>
      </thead>
      <tbody>
        <tr>
          <td class="tableblock halign-left valign-top">
            <div class="content">
              <div class="paragraph"><p>AsciiDoc</p></div>
            </div>
          </td>
        </tr>
      </tbody>
    </table>
    <div class="paragraph">
      <p>See <a href="#foo">Foo</a>.</p>
    </div>
  "##}
);

assert_html!(
  anchor_starting_implicit_reparsed_header_cell,
  adoc! {r#"
    [cols=1a]
    |===
    |[[foo,Foo]]* not AsciiDoc

    | AsciiDoc
    |===

    See <<foo>>.
  "#},
  html! { r##"
    <table class="tableblock frame-all grid-all stretch">
      <colgroup><col style="width: 100%;"></colgroup>
      <thead>
        <tr>
          <th class="tableblock halign-left valign-top">
            <a id="foo"></a>* not AsciiDoc
          </th>
        </tr>
      </thead>
      <tbody>
        <tr>
          <td class="tableblock halign-left valign-top">
            <div class="content">
              <div class="paragraph"><p>AsciiDoc</p></div>
            </div>
          </td>
        </tr>
      </tbody>
    </table>
    <div class="paragraph">
      <p>See <a href="#foo">Foo</a>.</p>
    </div>
  "##}
);

assert_html!(
  xref_from_adoc_cell_to_parent,
  adoc! {r#"
    == Some

    |===
    a|See <<_more>>
    |===

    == More

    content
  "#},
  contains:
    r##"<p>See <a href="#_more">More</a></p>"##,
    r##"<h2 id="_more">More</h2>"##,
);

assert_html!(
  xref_from_parent_to_adoc_cell,
  adoc! {r#"
    And a <<tigers>> link.

    |===
    a|Here is [#tigers]#a text span#.
    |===
  "#},
  contains:
    r##"<p>And a <a href="#tigers">a text span</a> link.</p>"##,
    r##"<p>Here is <span id="tigers">a text span</span>.</p>"##,
);

assert_error!(
  xref_unknown_anchor_in_adoc_cell,
  adoc! {r#"
    |===
    a|<<foo>>
    |===
  "#},
  error! {r"
    2: a|<<foo>>
           ^^^ Invalid cross reference, no anchor found for `foo`
  "}
);

assert_html!(
  adoc_cell_global_footnote_numbering,
  adoc! {r#"
    main footnote:[main note 1]

    |===
    a|AsciiDoc footnote:[cell note]
    |===

    main footnote:[main note 2]
  "#},
  contains:
    r##"<a href="#_footnoteref_1">1</a>. main note 1"##,
    r##"<a href="#_footnoteref_2">2</a>. cell note"##,
    r##"<a href="#_footnoteref_3">3</a>. main note 2"##,
);

assert_html!(
  adoc_cell_global_section_ids,
  adoc! {r#"
    == sect

    |===
    a|

    == sect
    |===

    == sect
  "#},
  contains:
    r##"<h2 id="_sect">sect</h2>"##,
    r##"<h2 id="_sect_2">sect</h2>"##,
    r##"<h2 id="_sect_3">sect</h2>"##,
);

assert_html!(
  adoc_cell_can_turn_on_new_attr,
  adoc! {r#"
    |===
    a|
    :icons: font

    NOTE: This admonition does not have a font-based icon.
    |===
  "#},
  contains: r#"<i class="fa icon-note" title="Note"></i>"#
);

assert_html!(
  adoc_cell_cant_unset_readonly_jobattr,
  |s: &mut JobSettings| {
    s.job_attrs.insert_unchecked("icons", JobAttr::readonly(false));
  },
  adoc! {r#"
    |===
    a|
    :icons: font

    NOTE: This admonition does not have a font-based icon.
    |===
  "#},
  contains: r#"<td class="icon"><div class="title">Note</div></td>"#
);
