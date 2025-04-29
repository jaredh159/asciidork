use test_utils::*;

assert_html!(
  simple_book,
  adoc! {r#"
    = Book Title
    :doctype: book

    = Part 1

    == Chapter A

    content {doctype}
  "#},
  html! {r#"
    <h1 id="_part_1" class="sect0">Part 1</h1>
    <div class="sect1">
      <h2 id="_chapter_a">Chapter A</h2>
      <div class="sectionbody">
        <div class="paragraph"><p>content book</p></div>
      </div>
    </div>
  "#}
);

assert_html!(
  book_only_dedication,
  adoc! {r#"
    = Book Title
    :doctype: book

    [dedication]
    == Dedication

    For S.S.T.--

    thank you for the plague of archetypes.
  "#},
  html! {r#"
    <div class="sect1">
      <h2 id="_dedication">Dedication</h2>
      <div class="sectionbody">
        <div class="paragraph">
          <p>For S.S.T.--</p>
        </div>
        <div class="paragraph">
          <p>thank you for the plague of archetypes.</p>
        </div>
      </div>
    </div>
  "#}
);

assert_html!(
  simple_book_w_part_intro,
  adoc! {r#"
    = Book Title
    :doctype: book

    [.custom-class]
    = Part 1

    [partintro]
    It was a dark and stormy night...

    == Chapter A

    content
  "#},
  html! {r#"
    <h1 id="_part_1" class="sect0 custom-class">Part 1</h1>
    <div class="openblock partintro">
      <div class="content">
        <div class="paragraph">
          <p>It was a dark and stormy night&#8230;&#8203;</p>
        </div>
      </div>
    </div>
    <div class="sect1">
      <h2 id="_chapter_a">Chapter A</h2>
      <div class="sectionbody">
        <div class="paragraph">
          <p>content</p>
        </div>
      </div>
    </div>
  "#}
);

assert_html!(
  book_multiple_parts,
  adoc! {r#"
    = Book
    Doc Writer
    :doctype: book

    = Chapter One

    [partintro]
    It was a dark and stormy night...

    == Scene One

    Someone's gonna get axed.

    = Chapter Two

    [partintro]
    They couldn't believe their eyes when...

    == Interlude

    While they were waiting...

    = Chapter Three

    == Scene One

    That's all she wrote!
  "#},
  html! {r#"
    <h1 id="_chapter_one" class="sect0">Chapter One</h1>
    <div class="openblock partintro">
      <div class="content">
        <div class="paragraph">
          <p>It was a dark and stormy night&#8230;&#8203;</p>
        </div>
      </div>
    </div>
    <div class="sect1">
      <h2 id="_scene_one">Scene One</h2>
      <div class="sectionbody">
        <div class="paragraph">
          <p>Someone&#8217;s gonna get axed.</p>
        </div>
      </div>
    </div>
    <h1 id="_chapter_two" class="sect0">Chapter Two</h1>
    <div class="openblock partintro">
      <div class="content">
        <div class="paragraph">
          <p>They couldn&#8217;t believe their eyes when&#8230;&#8203;</p>
        </div>
      </div>
    </div>
    <div class="sect1">
      <h2 id="_interlude">Interlude</h2>
      <div class="sectionbody">
        <div class="paragraph">
        <p>While they were waiting&#8230;&#8203;</p>
        </div>
      </div>
    </div>
    <h1 id="_chapter_three" class="sect0">Chapter Three</h1>
    <div class="sect1">
      <h2 id="_scene_one_2">Scene One</h2>
      <div class="sectionbody">
        <div class="paragraph">
          <p>That&#8217;s all she wrote!</p>
        </div>
      </div>
    </div>
  "#}
);

assert_html!(
  multipart_book_w_special_sects,
  adoc! {r#"
    = Book With Preface
    :doctype: book

    [preface]
    = Preface

    Preface content

    = Part 1

    [partintro]
    Part intro content

    == Chapter 1

    Chapter content 1.1

    = Part 2

    == Chapter 1

    Chapter content 2.1

    [appendix]
    = Appendix Title

    Appendix content
  "#},
  html! {r#"
    <div class="sect1">
      <h2 id="_preface">Preface</h2>
      <div class="sectionbody">
        <div class="paragraph">
          <p>Preface content</p>
        </div>
      </div>
    </div>
    <h1 id="_part_1" class="sect0">Part 1</h1>
    <div class="openblock partintro">
      <div class="content">
        <div class="paragraph">
          <p>Part intro content</p>
        </div>
      </div>
    </div>
    <div class="sect1">
      <h2 id="_chapter_1">Chapter 1</h2>
      <div class="sectionbody">
        <div class="paragraph">
          <p>Chapter content 1.1</p>
        </div>
      </div>
    </div>
    <h1 id="_part_2" class="sect0">Part 2</h1>
    <div class="sect1">
      <h2 id="_chapter_1_2">Chapter 1</h2>
      <div class="sectionbody">
        <div class="paragraph">
          <p>Chapter content 2.1</p>
        </div>
      </div>
    </div>
    <div class="sect1">
      <h2 id="_appendix_title">Appendix A: Appendix Title</h2>
      <div class="sectionbody">
        <div class="paragraph">
          <p>Appendix content</p>
        </div>
      </div>
    </div>
  "#}
);

assert_html!(
  book_partintro_role_title_promoted,
  adoc! {r#"
    = Book
    :doctype: book

    = Part 1

    .Intro
    [partintro]
    Read this first.

    == Chapter 1
  "#},
  contains:
    r#"<div class="openblock partintro"><div class="title">Intro</div>"#
);

assert_html!(
  book_para_intro_title_not_promoted,
  adoc! {r#"
    = Book
    :doctype: book

    = Part 1

    .Intro
    Read this first.

    == Chapter 1
  "#},
  contains:
    r#"<div class="paragraph"><div class="title">Intro</div><p>Read this first.</p>"#
);

assert_html!(
  book_partintro_open_block_not_doubled,
  adoc! {r#"
    = Book
    :doctype: book

    = Part 1

    --
    part intro
    --

    == Chapter 1
  "#},
  contains: &html! {r#"
    <div class="openblock partintro">
      <div class="content">
        <div class="paragraph">
          <p>part intro</p>
        </div>
      </div>
    </div>
  "#}
);

assert_html!(
  book_partnums,
  strict: false,
  adoc! {r#"
    = The Secret Manual
    :doctype: book
    :sectnums:
    :partnums:

    = Defensive Operations

    == An Introduction to DefenseOps

    = Managing Werewolves
  "#},
  contains:
    "I: Defensive Operations",
    "1. An Introduction to DefenseOps",
    "II: Managing Werewolves",
);

assert_html!(
  appendix_prefix,
  adoc! {r#"
    = Multi-Part Book with Special Sections and TOC
    :doctype: book
    :toc:

    = The First Part

    == The First Chapter

    Chapter content

    [appendix]
    = The Appendix

    Appendix content
  "#},
  contains:
    r##"<li><a href="#_the_appendix">Appendix A: The Appendix</a>"##,
    r#"<h2 id="_the_appendix">Appendix A: The Appendix</h2>"#,
);

assert_html!(
  renders_invalid_book_best_effort,
  strict: false,
  adoc! {r#"
    = Invalid book
    :doctype: book

    Preamble

    = Invalid part

    No section
  "#},
  html! {r#"
    <div id="preamble">
      <div class="sectionbody">
        <div class="paragraph"><p>Preamble</p></div>
      </div>
    </div>
    <h1 id="_invalid_part" class="sect0">Invalid part</h1>
    <div class="openblock partintro">
      <div class="content">
        <div class="paragraph"><p>No section</p></div>
      </div>
    </div>
  "#}
);
