use asciidork_meta::{JobAttr, JobSettings};
use test_utils::*;

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
