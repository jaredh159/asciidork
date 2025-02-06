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
