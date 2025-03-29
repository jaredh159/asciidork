use test_utils::*;

assert_html!(
  simple_book,
  adoc! {r#"
    = Book Title
    :doctype: book

    = Part 1

    == Chapter A

    content
  "#},
  html! {r#"
    <h1 id="_part_1" class="sect0">Part 1</h1>
    <div class="sect1">
      <h2 id="_chapter_a">Chapter A</h2>
      <div class="sectionbody">
        <div class="paragraph"><p>content</p></div>
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
  book_partintro_title,
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
