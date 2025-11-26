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
    <section class="doc-section level-0"><h1 id="_part_1">Part 1</h1><section class="doc-section level-1"><h2 id="_chapter_a">Chapter A</h2><p>content book</p></section></section>
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
    <section class="doc-section level-1"><h2 id="_dedication">Dedication</h2><p>For S.S.T.--</p>
    <p>thank you for the plague of archetypes.</p></section>
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
    <section class="doc-section level-0 custom-class"><h1 id="_part_1">Part 1</h1><div class="open-block partintro"><div class="content"><p>It was a dark and stormy night&#8230;&#8203;</p></div></div>
    <section class="doc-section level-1"><h2 id="_chapter_a">Chapter A</h2><p>content</p></section></section>
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
    <section class="doc-section level-0"><h1 id="_chapter_one">Chapter One</h1><div class="open-block partintro"><div class="content"><p>It was a dark and stormy night&#8230;&#8203;</p></div></div>
    <section class="doc-section level-1"><h2 id="_scene_one">Scene One</h2><p>Someone&#8217;s gonna get axed.</p></section></section>
    <section class="doc-section level-0"><h1 id="_chapter_two">Chapter Two</h1><div class="open-block partintro"><div class="content"><p>They couldn&#8217;t believe their eyes when&#8230;&#8203;</p></div></div>
    <section class="doc-section level-1"><h2 id="_interlude">Interlude</h2><p>While they were waiting&#8230;&#8203;</p></section></section>
    <section class="doc-section level-0"><h1 id="_chapter_three">Chapter Three</h1><section class="doc-section level-1"><h2 id="_scene_one_2">Scene One</h2><p>That&#8217;s all she wrote!</p></section></section>
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
    <section class="doc-section level-1"><h2 id="_preface">Preface</h2><p>Preface content</p></section>
    <section class="doc-section level-0"><h1 id="_part_1">Part 1</h1><div class="open-block partintro"><div class="content"><p>Part intro content</p></div></div>
    <section class="doc-section level-1"><h2 id="_chapter_1">Chapter 1</h2><p>Chapter content 1.1</p></section></section>
    <section class="doc-section level-0"><h1 id="_part_2">Part 2</h1><section class="doc-section level-1"><h2 id="_chapter_1_2">Chapter 1</h2><p>Chapter content 2.1</p></section></section>
    <section class="doc-section level-1"><h2 id="_appendix_title">Appendix A: Appendix Title</h2><p>Appendix content</p></section>
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
  html! {r#"
    <section class="doc-section level-0"><h1 id="_part_1">Part 1</h1><section class="open-block partintro"><h6 class="block-title">Intro</h6>
    <div class="content"><p>Read this first.</p></div></section>
    <section class="doc-section level-1"><h2 id="_chapter_1">Chapter 1</h2></section></section>
  "#}
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
  html! {r##"
  <section class="doc-section level-0"><h1 id="_part_1">Part 1</h1><div class="open-block partintro"><div class="content"><section class="paragraph"><h6 class="block-title">Intro</h6><p>Read this first.</p></section></div></div>
  <section class="doc-section level-1"><h2 id="_chapter_1">Chapter 1</h2></section></section>
  "##}
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
  html! {r#"
    <section class="doc-section level-0"><h1 id="_part_1">Part 1</h1><div class="open-block partintro"><div class="content"><p>part intro</p></div></div>
    <section class="doc-section level-1"><h2 id="_chapter_1">Chapter 1</h2></section></section>
  "#}
);

assert_html!(
  multipart_book_toc,
  adoc! {r#"
    = Book Title
    :doctype: book
    :sectnums:
    :toc:

    = First Part

    == Chapter

    === Subsection
  "#},
  html! {r##"
    <nav id="toc" class="toc" role="doc-toc"><h2 id="toc-title">Table of Contents</h2><ol class="toc-list level-0"><li><a href="#_first_part">First Part</a><ol class="toc-list level-1"><li><a href="#_chapter">1. Chapter</a><ol class="toc-list level-2"><li><a href="#_subsection">1.1. Subsection</a></li></ol></li></ol></li></ol></nav><section class="doc-section level-0"><h1 id="_first_part">First Part</h1><section class="doc-section level-1"><h2 id="_chapter">1. Chapter</h2><section class="doc-section level-2"><h3 id="_subsection">1.1. Subsection</h3></section></section></section>
  "##}
);

assert_html!(
  book_part_chapter_signifiers_toc,
  strict: false,
  adoc! {r#"
    = The Secret Manual
    :doctype: book
    :sectnums:
    :partnums:
    :toc: macro
    :part-signifier: Part
    :chapter-signifier: Chapter

    toc::[]

    = Defensive Operations

    == An Introduction to DefenseOps

    = Managing Werewolves

    == Section
  "#},
  html! {r##"
    <section id="preamble" aria-label="Preamble"><nav id="toc" class="toc" role="doc-toc"><h2 id="toc-title">Table of Contents</h2><ol class="toc-list level-0"><li><a href="#_defensive_operations">Part I: Defensive Operations</a><ol class="toc-list level-1"><li><a href="#_an_introduction_to_defenseops">Chapter 1. An Introduction to DefenseOps</a></li></ol></li><li><a href="#_managing_werewolves">Part II: Managing Werewolves</a><ol class="toc-list level-1"><li><a href="#_section">Chapter 2. Section</a></li></ol></li></ol></nav></section>
    <section class="doc-section level-0"><h1 id="_defensive_operations">Part I: Defensive Operations</h1><section class="doc-section level-1"><h2 id="_an_introduction_to_defenseops">Chapter 1. An Introduction to DefenseOps</h2></section></section>
    <section class="doc-section level-0"><h1 id="_managing_werewolves">Part II: Managing Werewolves</h1><section class="doc-section level-1"><h2 id="_section">Chapter 2. Section</h2></section></section>
  "##}
);

assert_html!(
  article_gnarly_toc,
  strict: false,
  adoc! {r#"
    = Article Title
    :appendix-caption: Exhibit
    :sectnums:
    :toc:
    :toclevels: 6

    == Section

    === Subsection

    [appendix]
    == First Appendix

    === First Subsection

    ==== First Subsubsection

    ===== First Subsubsubsection

    === Second Subsection

    [appendix]
    == Second Appendix
  "#},
  html! {r##"
    <nav id="toc" class="toc" role="doc-toc"><h2 id="toc-title">Table of Contents</h2><ol class="toc-list level-1"><li><a href="#_section">1. Section</a><ol class="toc-list level-2"><li><a href="#_subsection">1.1. Subsection</a></li></ol></li><li><a href="#_first_appendix">Exhibit A: First Appendix</a><ol class="toc-list level-2"><li><a href="#_first_subsection">A.1. First Subsection</a><ol class="toc-list level-3"><li><a href="#_first_subsubsection">A.1.1. First Subsubsection</a><ol class="toc-list level-4"><li><a href="#_first_subsubsubsection">First Subsubsubsection</a></li></ol></li></ol></li><li><a href="#_second_subsection">A.2. Second Subsection</a></li></ol></li><li><a href="#_second_appendix">Exhibit B: Second Appendix</a></li></ol></nav><section class="doc-section level-1"><h2 id="_section">1. Section</h2><section class="doc-section level-2"><h3 id="_subsection">1.1. Subsection</h3></section></section>
    <section class="doc-section level-1"><h2 id="_first_appendix">Exhibit A: First Appendix</h2><section class="doc-section level-2"><h3 id="_first_subsection">A.1. First Subsection</h3><section class="doc-section level-3"><h4 id="_first_subsubsection">A.1.1. First Subsubsection</h4><section class="doc-section level-4"><h5 id="_first_subsubsubsection">First Subsubsubsection</h5></section></section></section>
    <section class="doc-section level-2"><h3 id="_second_subsection">A.2. Second Subsection</h3></section></section>
    <section class="doc-section level-1"><h2 id="_second_appendix">Exhibit B: Second Appendix</h2></section>
  "##}
);

assert_html!(
  book_gnarly_toc,
  adoc! {r#"
    = Book Title
    :doctype: book
    :sectnums:
    :toc:

    = First Part

    == Chapter

    === Subsection

    == Second Part

    == Chapter

    [appendix]
    = First Appendix

    === First Subsection

    === Second Subsection

    [appendix]
    = Second Appendix
  "#},
  html! {r##"
    <nav id="toc" class="toc" role="doc-toc"><h2 id="toc-title">Table of Contents</h2><ol class="toc-list level-0"><li><a href="#_first_part">First Part</a><ol class="toc-list level-1"><li><a href="#_chapter">1. Chapter</a><ol class="toc-list level-2"><li><a href="#_subsection">1.1. Subsection</a></li></ol></li><li><a href="#_second_part">2. Second Part</a></li><li><a href="#_chapter_2">3. Chapter</a></li></ol></li><li><a href="#_first_appendix">Appendix A: First Appendix</a><ol class="toc-list level-2"><li><a href="#_first_subsection">A.1. First Subsection</a></li><li><a href="#_second_subsection">A.2. Second Subsection</a></li></ol></li><li><a href="#_second_appendix">Appendix B: Second Appendix</a></li></ol></nav><section class="doc-section level-0"><h1 id="_first_part">First Part</h1><section class="doc-section level-1"><h2 id="_chapter">1. Chapter</h2><section class="doc-section level-2"><h3 id="_subsection">1.1. Subsection</h3></section></section>
    <section class="doc-section level-1"><h2 id="_second_part">2. Second Part</h2></section>
    <section class="doc-section level-1"><h2 id="_chapter_2">3. Chapter</h2></section></section>
    <section class="doc-section level-1"><h2 id="_first_appendix">Appendix A: First Appendix</h2><section class="doc-section level-2"><h3 id="_first_subsection">A.1. First Subsection</h3></section>
    <section class="doc-section level-2"><h3 id="_second_subsection">A.2. Second Subsection</h3></section></section>
    <section class="doc-section level-1"><h2 id="_second_appendix">Appendix B: Second Appendix</h2></section>
  "##}
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

    == Section
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
