use test_utils::*;

assert_html!(
  basic_toc,
  adoc! {"
    = Doc Title
    :showtitle:
    :toc:

    == Section 1

    foo

    == Section 2

    bar
  "},
  html! {r##"
    <h1>Doc Title</h1><nav id="toc" class="toc" role="doc-toc"><h2 id="toc-title">Table of Contents</h2><ol class="toc-list level-1"><li><a href="#_section_1">Section 1</a></li><li><a href="#_section_2">Section 2</a></li></ol></nav><section class="doc-section level-1"><h2 id="_section_1">Section 1</h2><p>foo</p></section>
    <section class="doc-section level-1"><h2 id="_section_2">Section 2</h2><p>bar</p></section>
  "##}
);

assert_html!(
  toc_preamble,
  adoc! {"
    = Doc Title
    :toc: preamble

    preamble

    == Section 1

    == Section 2
  "},
  html! {r##"
    <section id="preamble" aria-label="Preamble"><p>preamble</p></section><nav id="toc" class="toc" role="doc-toc"><h2 id="toc-title">Table of Contents</h2><ol class="toc-list level-1"><li><a href="#_section_1">Section 1</a></li><li><a href="#_section_2">Section 2</a></li></ol></nav>
    <section class="doc-section level-1"><h2 id="_section_1">Section 1</h2></section>
    <section class="doc-section level-1"><h2 id="_section_2">Section 2</h2></section>
  "##}
);

assert_html!(
  toc_macro,
  adoc! {"
    = Doc Title
    :toc: macro

    == Section 1

    == Section 2

    toc::[]
  "},
  html! {r##"
    <section class="doc-section level-1"><h2 id="_section_1">Section 1</h2></section>
    <section class="doc-section level-1"><h2 id="_section_2">Section 2</h2><nav id="toc" class="toc" role="doc-toc"><h3 id="toc-title">Table of Contents</h3><ol class="toc-list level-1"><li><a href="#_section_1">Section 1</a></li><li><a href="#_section_2">Section 2</a></li></ol></nav></section>
  "##}
);

assert_html!(
  toc_macro_custom_id_class,
  adoc! {"
    = Doc Title
    :toc: macro
    :toc-class: table-of-contents

    == Section 1

    [#custom-id]
    toc::[]
  "},
  html! {r##"
    <section class="doc-section level-1"><h2 id="_section_1">Section 1</h2><nav id="custom-id" class="table-of-contents" role="doc-toc"><h3 id="custom-id-title">Table of Contents</h3><ol class="toc-list level-1"><li><a href="#_section_1">Section 1</a></li></ol></nav></section>
  "##}
);

assert_html!(
  nested_toc,
  adoc! {"
    = Doc Title
    :showtitle:
    :toc:
    :toc-title: Ham Sandwich

    == sect 1

    === sect 1.1

    == sect 2

    === sect 2.1
  "},
  html! {r##"
    <h1>Doc Title</h1><nav id="toc" class="toc" role="doc-toc"><h2 id="toc-title">Ham Sandwich</h2><ol class="toc-list level-1"><li><a href="#_sect_1">sect 1</a><ol class="toc-list level-2"><li><a href="#_sect_1_1">sect 1.1</a></li></ol></li><li><a href="#_sect_2">sect 2</a><ol class="toc-list level-2"><li><a href="#_sect_2_1">sect 2.1</a></li></ol></li></ol></nav><section class="doc-section level-1"><h2 id="_sect_1">sect 1</h2><section class="doc-section level-2"><h3 id="_sect_1_1">sect 1.1</h3></section></section>
    <section class="doc-section level-1"><h2 id="_sect_2">sect 2</h2><section class="doc-section level-2"><h3 id="_sect_2_1">sect 2.1</h3></section></section>
  "##}
);
