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
    <div id="header">
      <h1>Doc Title</h1>
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
    <div id="header">
      <h1>Doc Title</h1>
    </div>
    <div id="toc" class="toc">
      <div id="toctitle">Ham Sandwich</div>
      <ul class="sectlevel1">
        <li><a href="#_sect_1">sect 1</a>
          <ul class="sectlevel2">
            <li><a href="#_sect_1_1">sect 1.1</a></li>
          </ul>
        </li>
        <li><a href="#_sect_2">sect 2</a>
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
