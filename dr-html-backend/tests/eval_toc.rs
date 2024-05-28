use test_utils::{adoc, html};

mod helpers;

test_eval!(
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
    <h1>Doc Title</h1>
    <div id="toc" class="toc">
      <div id="toctitle">Table of Contents</div>
      <ul class="sectlevel1">
        <li><a href="#_section_1">Section 1</a></li>
        <li><a href="#_section_2">Section 2</a></li>
      </ul>
    </div>
    <div class="sect1">
      <h2 id="_section_1">Section 1</h2>
      <div class="sectionbody">
        <div class="paragraph"><p>foo</p></div>
      </div>
    </div>
    <div class="sect1">
      <h2 id="_section_2">Section 2</h2>
      <div class="sectionbody">
        <div class="paragraph"><p>bar</p></div>
      </div>
    </div>
  "##}
);

test_eval!(
  toc_preamble,
  adoc! {"
    = Doc Title
    :toc: preamble

    preamble

    == Section 1

    == Section 2
  "},
  html! {r##"
    <div id="preamble">
      <div class="sectionbody">
        <div class="paragraph"><p>preamble</p></div>
      </div>
    </div>
    <div id="toc" class="toc">
      <div id="toctitle">Table of Contents</div>
      <ul class="sectlevel1">
        <li><a href="#_section_1">Section 1</a></li>
        <li><a href="#_section_2">Section 2</a></li>
      </ul>
    </div>
    <div class="sect1">
      <h2 id="_section_1">Section 1</h2>
      <div class="sectionbody"></div>
    </div>
    <div class="sect1">
      <h2 id="_section_2">Section 2</h2>
      <div class="sectionbody"></div>
    </div>
  "##}
);

test_eval!(
  toc_macro,
  adoc! {"
    = Doc Title
    :toc: macro

    == Section 1

    == Section 2

    toc::[]
  "},
  html! {r##"
    <div class="sect1">
      <h2 id="_section_1">Section 1</h2>
      <div class="sectionbody"></div>
    </div>
    <div class="sect1">
      <h2 id="_section_2">Section 2</h2>
      <div class="sectionbody">
        <div id="toc" class="toc">
          <div id="toctitle">Table of Contents</div>
          <ul class="sectlevel1">
            <li><a href="#_section_1">Section 1</a></li>
            <li><a href="#_section_2">Section 2</a></li>
          </ul>
        </div>
      </div>
    </div>
  "##}
);

test_eval!(
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
    <h1>Doc Title</h1>
    <div id="toc" class="toc">
      <div id="toctitle">Ham Sandwich</div>
      <ul class="sectlevel1">
        <li>
          <a href="#_sect_1">sect 1</a>
          <ul class="sectlevel2">
            <li><a href="#_sect_1_1">sect 1.1</a></li>
          </ul>
        </li>
        <li>
          <a href="#_sect_2">sect 2</a>
          <ul class="sectlevel2">
            <li><a href="#_sect_2_1">sect 2.1</a></li>
          </ul>
        </li>
      </ul>
    </div>
    <div class="sect1">
      <h2 id="_sect_1">sect 1</h2>
      <div class="sectionbody">
        <div class="sect2">
          <h3 id="_sect_1_1">sect 1.1</h3>
        </div>
      </div>
    </div>
    <div class="sect1">
      <h2 id="_sect_2">sect 2</h2>
      <div class="sectionbody">
        <div class="sect2">
          <h3 id="_sect_2_1">sect 2.1</h3>
        </div>
      </div>
    </div>
  "##}
);

test_eval!(
  dont_render_empty_toc,
  adoc! {"
    = Doc Title
    :toc:

    not sectioned
  "},
  html! {r#"
    <div class="paragraph">
      <p>not sectioned</p>
    </div>
  "#}
);

test_non_embedded_contains!(
  toc_special_classes,
  adoc! {"
    = Doc Title
    :toc: left

    == Section 1
  "},
  [
    r#"<body class="article toc2 toc-left">"#,
    r#"<div id="toc" class="toc2">"#
  ],
);
