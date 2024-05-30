use asciidork_meta::{JobAttr, JobSettings};
use asciidork_parser::Parser;
use test_utils::{assert_eq, *};

mod helpers;

test_eval!(
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

test_eval!(
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

test_eval!(
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

test_eval!(
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
  html_contains: r#"<div class="content"><div class="paragraph"><p>content"#
);

test_eval!(
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

test_eval!(
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

test_eval!(
  xref_from_adoc_cell_to_parent,
  adoc! {r#"
    == Some

    |===
    a|See <<_more>>
    |===

    == More

    content
  "#},
  html_contains: r##"<p>See <a href="#_more">More</a></p>"##
);

test_eval!(
  xref_from_parent_to_adoc_cell,
  adoc! {r#"
    And a <<tigers>> link.

    |===
    a|Here is [#tigers]#a text span#.
    |===
  "#},
  html! { r##"
    <div class="paragraph">
      <p>And a <a href="#tigers">a text span</a> link.</p>
    </div>
    <table class="tableblock frame-all grid-all stretch">
      <colgroup><col style="width: 100%;"></colgroup>
      <tbody>
        <tr>
          <td class="tableblock halign-left valign-top">
            <div class="content">
              <div class="paragraph">
                <p>Here is <span id="tigers">a text span</span>.</p>
              </div>
            </div>
          </td>
        </tr>
      </tbody>
    </table>
  "##}
);

test_error!(
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

test_eval!(
  adoc_cell_can_turn_on_new_attr,
  adoc! {r#"
    |===
    a|
    :icons: font

    NOTE: This admonition does not have a font-based icon.
    |===
  "#},
  html_contains: r#"<i class="fa icon-note" title="Note"></i>"#
);

test_eval!(
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
  html_contains: r#"<td class="icon"><div class="title">Note</div></td>"#
);
