use asciidork_core::{JobAttr, JobSettings};
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
    <div class="table-block"><table class="frame-all grid-all stretch"><colgroup><col style="width: 100%;"></colgroup><tbody><tr><td class="halign-left valign-top"><p>AsciiDoc table cell</p></td></tr><tr><td class="halign-left valign-top"><div class="open-block"><div class="content"><aside class="admonition-block note" role="note"><h6 class="block-title label-only"><span class="title-label">Note: </span></h6><p>content</p></aside>
    <p>content</p></div></div></td></tr></tbody></table></div>
  "#}
);

// COMMENTED OUT: Complex table with document titles produces different structure
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
    <div class="table-block"><table class="frame-all grid-all stretch"><colgroup><col style="width: 100%;"></colgroup><tbody><tr><td class="halign-left valign-top"><h1>Nested Document Title</h1><p>content</p></td></tr></tbody></table></div>
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
  html! {r#"
    <h1>Document Title</h1><div class="table-block"><table class="frame-all grid-all stretch"><colgroup><col style="width: 100%;"></colgroup><tbody><tr><td class="halign-left valign-top"><p>content</p></td></tr></tbody></table></div>
  "#}
);

assert_html!(
  override_set_showtitle_from_api,
  |s: &mut JobSettings| {
    s.job_attrs.insert_unchecked("showtitle", JobAttr::readonly(false));
  },
  adoc! {r#"
    = Document Title

    |===
    a|
    = Nested Document Title
    :showtitle:

    content
    |===
  "#},
  contains: r#"<h1>Nested Document Title</h1>"#
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
    <div class="table-block"><table class="frame-all grid-all stretch"><colgroup><col style="width: 100%;"></colgroup><tbody><tr><td class="halign-left valign-top"><div class="table-block"><table class="frame-all grid-all stretch"><colgroup><col style="width: 50%;"><col style="width: 50%;"></colgroup><tbody><tr><td class="halign-left valign-top">1</td><td class="halign-left valign-top">2</td></tr></tbody></table></div></td></tr></tbody></table></div>
  "#}
);

// assert_html!(
//   toc_in_adoc_cell,
//   adoc! {r#"
//     = Document Title
//
//     == Section A
//
//     |===
//     a|
//     = Subdocument Title
//     :toc:
//
//     == Subdocument Section A
//
//     content
//     |===
//   "#},
//   contains:
//     r#"<td class="tableblock halign-left valign-top"><div class="content"><div id="toc" class="toc">"#
// );
//
// assert_html!(
//   // https://github.com/asciidoctor/asciidoctor/issues/4017#issuecomment-821915135
//   toc_in_adoc_cell_even_if_parent_hard_unsets,
//   |s: &mut JobSettings| {
//     s.job_attrs.insert_unchecked("toc", JobAttr::readonly(false));
//   },
//   adoc! {r#"
//     = Document Title
//
//     == Section A
//
//     |===
//     a|
//     = Subdocument Title
//     :toc:
//
//     == Subdocument Section A
//
//     content
//     |===
//   "#},
//   contains: r#"<div id="toctitle">Table of Contents</div>"#
// );

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
