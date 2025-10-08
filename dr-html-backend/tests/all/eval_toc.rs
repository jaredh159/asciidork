use test_utils::{adoc, html};

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

assert_html!(
  multipart_book_toc,
  adoc! {"
    = Multi-Part Book with Special Sections
    :doctype: book
    :toc:

    [colophon]
    = The Colophon

    Colophon content

    = The First Part

    == The First Chapter

    Chapter 1 content

    [appendix]
    = The Appendix

    === Basics

    Basics content

    === Subsections

    Subsection content
  "},
  html! {r##"
    <div id="toc" class="toc">
      <div id="toctitle">Table of Contents</div>
      <ul class="sectlevel0">
        <li><a href="#_the_colophon">The Colophon</a></li>
        <li><a href="#_the_first_part">The First Part</a>
        <ul class="sectlevel1">
          <li><a href="#_the_first_chapter">The First Chapter</a></li>
        </ul>
        </li>
        <li>
          <a href="#_the_appendix">Appendix A: The Appendix</a>
          <ul class="sectlevel2">
            <li><a href="#_basics">Basics</a></li>
            <li><a href="#_subsections">Subsections</a></li>
          </ul>
        </li>
      </ul>
    </div>
    <div class="sect1">
      <h2 id="_the_colophon">The Colophon</h2>
      <div class="sectionbody">
        <div class="paragraph"><p>Colophon content</p></div>
      </div>
    </div>
    <h1 id="_the_first_part" class="sect0">The First Part</h1>
    <div class="sect1">
      <h2 id="_the_first_chapter">The First Chapter</h2>
      <div class="sectionbody">
        <div class="paragraph"><p>Chapter 1 content</p></div>
      </div>
    </div>
    <div class="sect1">
      <h2 id="_the_appendix">Appendix A: The Appendix</h2>
      <div class="sectionbody">
        <div class="sect2">
          <h3 id="_basics">Basics</h3>
          <div class="paragraph"><p>Basics content</p></div>
        </div>
        <div class="sect2">
          <h3 id="_subsections">Subsections</h3>
          <div class="paragraph"><p>Subsection content</p></div>
        </div>
      </div>
    </div>
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
    <div class="sect1">
      <h2 id="_section_1">Section 1</h2>
      <div class="sectionbody"></div>
    </div>
    <div class="sect1">
      <h2 id="_section_2">Section 2</h2>
      <div class="sectionbody">
        <div id="toc" class="toc">
          <div id="toctitle" class="title">Table of Contents</div>
          <ul class="sectlevel1">
            <li><a href="#_section_1">Section 1</a></li>
            <li><a href="#_section_2">Section 2</a></li>
          </ul>
        </div>
      </div>
    </div>
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
  contains:
    r#"<div id="custom-id" class="table-of-contents">"#,
    r#"<div id="custom-idtitle" class="title">Table of Contents</div>"#,
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
  contains: &html! {r##"
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
  "##}
);

assert_html!(
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

assert_html!(
  table_cell_toc,
  adoc! {"
    = Document Title
    :toc:

    == Section A

    |===
    a|
    = Subdocument Title
    :toc: macro

    [#table-cell-toc]
    toc::[]

    == Subdocument Section A
    |===
  "},
  contains:
    r#"<div id="table-cell-toc" class="toc">"#,
    r#"<div id="table-cell-toctitle" class="title">Table of Contents"#,
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
