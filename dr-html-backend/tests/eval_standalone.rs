use test_utils::*;

mod helpers;

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
