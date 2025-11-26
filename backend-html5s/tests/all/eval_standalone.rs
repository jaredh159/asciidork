// use asciidork_core::{JobAttr, JobSettings};
use asciidork_parser::prelude::*;
use test_utils::*;

// NOTE: Standalone document tests need updates for jirutka's semantic HTML structure

assert_standalone_body!(
  normal_doc_structure,
  adoc! {r#"
    = Document Title
    Bob Smith
  "#},
  html! {r#"
    <body class="article"><header><h1>Document Title</h1><div class="details"><span class="author" id="author">Bob Smith</span><br></div></header><div id="content"></div><footer><div id="footer-text"></div></footer></body>
  "#}
);

assert_standalone_body!(
  normal_doc_structure2,
  adoc! {r#"
    = Document Title
    Bob Smith <bob@smith.com>
  "#},
  html! {r#"
    <body class="article"><header><h1>Document Title</h1><div class="details"><span class="author" id="author">Bob Smith</span><br><span class="email" id="email"><a href="mailto:bob@smith.com">bob@smith.com</a></span><br></div></header><div id="content"></div><footer><div id="footer-text"></div></footer></body>
  "#}
);

assert_standalone_body!(
  normal_doc_structure3,
  adoc! {r#"
    = Document Title
    Bob Smith <bob@smith.com>; Kate Smith; Henry Sue <henry@sue.com>
  "#},
  html! {r#"
    <body class="article"><header><h1>Document Title</h1><div class="details"><span class="author" id="author">Bob Smith</span><br><span class="email" id="email"><a href="mailto:bob@smith.com">bob@smith.com</a></span><br><span class="author" id="author2">Kate Smith</span><br><span class="author" id="author3">Henry Sue</span><br><span class="email" id="email3"><a href="mailto:henry@sue.com">henry@sue.com</a></span></div></header><div id="content"></div><footer><div id="footer-text"></div></footer></body>
  "#}
);

assert_standalone_body!(
  revision_marks,
  adoc! {r#"
    = The Intrepid Chronicles
    Kismet Lee
    2.9, October 31, 2021: Fall incarnation
  "#},
  html! {r#"
    <body class="article"><header><h1>The Intrepid Chronicles</h1><div class="details"><span class="author" id="author">Kismet Lee</span><br><span id="revnumber">version 2.9,</span> <time id="revdate" datetime="2021-10-31">October 31, 2021</time><br><span id="revremark">Fall incarnation</span></div></header><div id="content"></div><footer><div id="footer-text">Version 2.9</div></footer></body>
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
    <body class="article"><div id="content"></div></body>
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
    <body class="article"><header><div class="details"><span class="author" id="author">Bob Smith</span><br></div></header><div id="content"></div><footer><div id="footer-text"></div></footer></body>
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
    <body id="custom-id" class="article"><header><h1>Document Title</h1></header><div id="content"></div></body>
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
    <body class="article" style="max-width: 3 furlongs;"><header><h1>Document Title</h1></header><div id="content"><p>hi<a class="footnote-ref" id="_footnoteref_1" href="#_footnote_1" title="View footnote 1" role="doc-noteref">[1]</a></p></div><section class="footnotes" aria-label="Footnotes" role="doc-endnotes"><hr><ol class="footnotes"><li class="footnote" id="_footnote_1" role="doc-endnote">bye <a class="footnote-backref" href="#_footnoteref_1" role="doc-backlink" title="Jump to the first occurrence in the text">&#8617;</a></li></ol></section><footer><div id="footer-text"></div></footer></body>
  "##}
);

test_non_embedded_contains!(
  head_css,
  adoc! {r#"
    = Document Title

    paragraph
  "#},
  [
    html! {r##"
    <!DOCTYPE html><html lang="en"><head><meta charset="UTF-8"><meta http-equiv="X-UA-Compatible" content="IE=edge"><meta name="viewport" content="width=device-width, initial-scale=1.0"><meta name="generator" content="Asciidork"><title>Document Title</title><style>
  "##},
    "<style>\nhtml{font-family:sans-serif;-webkit-text-size-adjust:100%}".to_string(),
    "@media amzn-kf8{#header,#content,#footnotes,#footer{padding:0}}\n</style>".to_string(),
  ]
);
