use asciidork_core::{JobAttr, JobSettings};
use asciidork_parser::prelude::*;
use test_utils::*;

assert_standalone_body!(
  normal_doc_structure,
  adoc! {r#"
    = Document Title
    Bob Smith
  "#},
  html! {r#"
    <body class="article">
      <div id="header">
        <h1>Document Title</h1>
        <div class="details">
          <span id="author" class="author">Bob Smith</span><br>
        </div>
      </div>
      <div id="content"></div>
      <div id="footer"></div>
    </body>
  "#}
);

assert_standalone_body!(
  normal_doc_structure_win_crlf,
  adoc_win_crlf! {r#"
    = Document Title
    Bob Smith
  "#},
  html! {r#"
    <body class="article">
      <div id="header">
        <h1>Document Title</h1>
        <div class="details">
          <span id="author" class="author">Bob Smith</span><br>
        </div>
      </div>
      <div id="content"></div>
      <div id="footer"></div>
    </body>
  "#}
);

assert_standalone_body!(
  disable_doc_sections,
  adoc! {r#"
    = Document Title
    :noheader:
    :nofooter:
  "#},
  html! {r#"
    <body class="article">
      <div id="content"></div>
    </body>
  "#}
);

assert_standalone_body!(
  doc_attrs_after_comment,
  adoc! {r#"
    = Document Title
    :noheader:
    // here is a comment
    :nofooter:
  "#},
  html! {r#"
    <body class="article">
      <div id="content"></div>
    </body>
  "#}
);

assert_standalone_body!(
  notitle,
  adoc! {r#"
    = Document Title
    Bob Smith
    :notitle:
  "#},
  html! {r#"
    <body class="article">
      <div id="header">
        <div class="details">
          <span id="author" class="author">Bob Smith</span><br>
        </div>
      </div>
      <div id="content"></div>
      <div id="footer"></div>
    </body>
  "#}
);

assert_standalone_body!(
  doctitle_from_leveloffset,
  adoc! {r#"
    :leveloffset: -1
    == Document Title
  "#},
  html! {r#"
    <body class="article">
      <div id="header">
        <h1>Document Title</h1>
      </div>
      <div id="content"></div>
      <div id="footer"></div>
    </body>
  "#}
);

assert_standalone_body!(
  doctitle_from_leveloffset_api,
  |job_settings: &mut JobSettings| {
    job_settings
      .job_attrs
      .insert_unchecked("leveloffset", JobAttr::readonly("-1"));
  },
  adoc! {r#"
    == Document Title
  "#},
  html! {r#"
    <body class="article">
      <div id="header">
        <h1>Document Title</h1>
      </div>
      <div id="content"></div>
      <div id="footer"></div>
    </body>
  "#}
);

assert_standalone_body!(
  css_signature_honored,
  adoc! {r#"
    = Document Title
    :css-signature: custom-id
    :nofooter:
  "#},
  html! {r#"
    <body id="custom-id" class="article">
      <div id="header">
        <h1>Document Title</h1>
      </div>
      <div id="content"></div>
    </body>
  "#}
);

assert_standalone_body!(
  max_width_honored,
  adoc! {r#"
    = Document Title
    :max-width: 3 furlongs

    hifootnote:[bye]
  "#},
  html! {r##"
    <body class="article">
      <div id="header" style="max-width: 3 furlongs;">
        <h1>Document Title</h1>
      </div>
      <div id="content" style="max-width: 3 furlongs;">
        <div class="paragraph">
          <p>hi<sup class="footnote">[<a id="_footnoteref_1" class="footnote" href="#_footnotedef_1" title="View footnote.">1</a>]</sup></p>
        </div>
      </div>
      <div id="footnotes" style="max-width: 3 furlongs;">
        <hr>
        <div class="footnote" id="_footnotedef_1">
          <a href="#_footnoteref_1">1</a>. bye
        </div>
      </div>
      <div id="footer" style="max-width: 3 furlongs;"></div>
    </body>
  "##}
);

assert_standalone_body!(
  multiple_complex_authors,
  adoc! {r#"
    = The Intrepid Chronicles
    Kismet R. Lee <kismet@asciidoctor.org>; B. Steppenwolf; Pax Draeke <pax@asciidoctor.org>
  "#},
  html! {r#"
    <body class="article">
      <div id="header">
        <h1>The Intrepid Chronicles</h1>
        <div class="details">
          <span id="author" class="author">Kismet R. Lee</span><br>
          <span id="email" class="email"><a href="mailto:kismet@asciidoctor.org">kismet@asciidoctor.org</a></span><br>
          <span id="author2" class="author">B. Steppenwolf</span><br>
          <span id="author3" class="author">Pax Draeke</span><br>
          <span id="email3" class="email"><a href="mailto:pax@asciidoctor.org">pax@asciidoctor.org</a></span><br>
        </div>
      </div>
      <div id="content"></div>
      <div id="footer"></div>
    </body>
  "#}
);

assert_standalone_body!(
  author_attr,
  adoc! {r#"
    = The Intrepid Chronicles
    :author: Kismet R. Lee
    :email: kismet@asciidoctor.org
  "#},
  html! {r#"
    <body class="article">
      <div id="header">
        <h1>The Intrepid Chronicles</h1>
        <div class="details">
          <span id="author" class="author">Kismet R. Lee</span><br>
          <span id="email" class="email"><a href="mailto:kismet@asciidoctor.org">kismet@asciidoctor.org</a></span><br>
        </div>
      </div>
      <div id="content"></div>
      <div id="footer"></div>
    </body>
  "#}
);

test_non_embedded_contains!(
  webfonts_css_default,
  adoc! {"
    hello world
  "},
  [
    r#"<link rel="stylesheet" href="https://fonts.googleapis.com/css?family=Open+Sans:300,300italic,400,400italic,600,600italic%7CNoto+Serif:400,400italic,700,700italic%7CDroid+Sans+Mono:400,700" />"#
  ]
);

test_non_embedded_contains!(
  webfonts_css_override,
  adoc! {"
    :webfonts: custom

    hello world
  "},
  [r#"<link rel="stylesheet" href="https://fonts.googleapis.com/css?family=custom" />"#]
);

test_non_embedded_contains!(
  exceptions_before_doc_title,
  adoc! {"
    :toc: left

    :foo: bar

    // foobar

    // comment w/ attr below
    :thing: cool

    :wow: baz
    // attr w/ comment below

    ////
    a comment block
    ////

    ////
    a comment block

    with an empty line
    ////

    :a: b
    ////
    comment block after attr
    ////

    ////
    comment block before attr
    ////
    :c: d

    ////
    comment block before title
    ////
    = Doc Title
  "},
  ["<h1>Doc Title</h1>"],
);

assert_html!(
  level_0_heading_best_effort,
  strict: false,
  adoc! {r#"
    foo bar

    = Doc title
  "#},
  contains:
    r#"<div class="sect0"><h1 id="_doc_title">Doc title</h1></div>"#
);

assert_error!(
  level_0_heading_from_malformed_header,
  adoc! {"
    paragraph

    = Title
  "},
  error! {"
     --> test.adoc:3:1
      |
    3 | = Title
      | ^^^^^^^ Level 0 section allowed only in doctype=book
  "}
);
