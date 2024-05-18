use asciidork_dr_html_backend::AsciidoctorHtml;
use asciidork_eval::{eval, Opts};
use asciidork_parser::prelude::*;
use test_utils::{adoc, assert_eq, html};

mod helpers;

use regex::Regex;

test_eval!(
  simple_inline_w_newline,
  adoc! {r#"
    _foo_
    bar
  "#},
  r#"<div class="paragraph"><p><em>foo</em> bar</p></div>"#
);

test_eval!(
  nested_inlines,
  "`*_foo_*`",
  r#"<div class="paragraph"><p><code><strong><em>foo</em></strong></code></p></div>"#
);

test_eval!(
  passthrough,
  "+_<foo>&_+",
  r#"<div class="paragraph"><p>_&lt;foo&gt;&amp;_</p></div>"#
);

test_eval!(
  text_span,
  "[.foo]#bar#",
  r#"<div class="paragraph"><p><span class="foo">bar</span></p></div>"#
);

test_eval!(passthrough_block, "[pass]\n_<foo>&_", "_<foo>&_");

test_eval!(
  minus_subs,
  "[subs=-specialchars]\nfoo & _bar_",
  r#"<div class="paragraph"><p>foo & <em>bar</em></p></div>"#
);

test_eval!(
  highlight,
  "foo #bar#",
  r#"<div class="paragraph"><p>foo <mark>bar</mark></p></div>"#
);

test_eval!(
  mono,
  "foo `bar`",
  r#"<div class="paragraph"><p>foo <code>bar</code></p></div>"#
);

test_eval!(
  passthrough_2,
  "rofl +_foo_+ lol",
  r#"<div class="paragraph"><p>rofl _foo_ lol</p></div>"#
);

test_eval!(
  inline_passthrough,
  "+++_<foo>&_+++ bar",
  r#"<div class="paragraph"><p>_<foo>&_ bar</p></div>"#
);

test_eval!(
  subscript,
  "foo ~bar~ baz",
  r#"<div class="paragraph"><p>foo <sub>bar</sub> baz</p></div>"#
);

test_eval!(
  superscript,
  "foo ^bar^ baz",
  r#"<div class="paragraph"><p>foo <sup>bar</sup> baz</p></div>"#
);

test_eval!(
  not_quotes,
  "foo `'bar'`",
  r#"<div class="paragraph"><p>foo <code>'bar'</code></p></div>"#
);

test_eval!(
  curly_quotes,
  "foo \"`bar`\"",
  r#"<div class="paragraph"><p>foo &#8220;bar&#8221;</p></div>"#
);

test_eval!(
  implicit_apos,
  "Olaf's wrench",
  r#"<div class="paragraph"><p>Olaf&#8217;s wrench</p></div>"#
);

test_eval!(
  multichar_whitespace,
  "foo   bar",
  r#"<div class="paragraph"><p>foo bar</p></div>"#
);

test_eval!(
  litmono_attr_ref,
  "`+{name}+`",
  r#"<div class="paragraph"><p><code>{name}</code></p></div>"#
);

test_eval!(
  special_chars,
  "foo <bar> & lol",
  r#"<div class="paragraph"><p>foo &lt;bar&gt; &amp; lol</p></div>"#
);

test_eval!(
  btn_macro,
  "press the btn:[OK] button",
  r#"<div class="paragraph"><p>press the <b class="button">OK</b> button</p></div>"#
);

test_eval!(
  comment_lines,
  adoc! {r#"
    // leading
    foo
    bar

    foo
    // middle
    bar

    foo
    bar
    // trailing

    foo // not a comment
    bar

    ----
    // retained in verbatim
    ----
  "#},
  html! {r#"
    <div class="paragraph">
      <p>foo bar</p>
    </div>
    <div class="paragraph">
      <p>foo bar</p>
    </div>
    <div class="paragraph">
      <p>foo bar</p>
    </div>
    <div class="paragraph">
      <p>foo // not a comment bar</p>
    </div>
    <div class="listingblock">
      <div class="content">
        <pre>// retained in verbatim</pre>
      </div>
    </div>
  "#}
);

test_eval!(
  menu_macro,
  "select menu:File[Save].",
  html! {r#"
    <div class="paragraph">
      <p>select <span class="menuseq"><span class="menu">File</span>&#160;&#9656;<span class="menuitem">Save</span></span>.</p>
    </div>
  "#}
);

test_eval!(
  menu_macro_2,
  "select menu:File[Save > Reset].",
  html! {r#"
    <div class="paragraph">
      <p>
        select <span class="menuseq"
          ><span class="menu">File</span>&#160;&#9656;
          <span class="submenu">Save</span>&#160;&#9656;
          <span class="menuitem">Reset</span></span
        >.
      </p>
    </div>
  "#}
);

test_eval!(
  para_w_attrs,
  adoc! {r#"
    [#custom-id.custom-class]
    foo bar
  "#},
  html! {r#"
    <div id="custom-id" class="paragraph custom-class">
      <p>foo bar</p>
    </div>
  "#}
);

test_eval!(
  sidebar,
  "[sidebar]\nfoo bar",
  html! {r#"
    <div class="sidebarblock">
      <div class="content">
        foo bar
      </div>
    </div>
  "#}
);

test_eval!(
  title,
  ".Title\nfoo",
  html! {r#"
    <div class="paragraph">
      <div class="title">Title</div>
      <p>foo</p>
    </div>
  "#}
);

test_eval!(
  admonition_w_custom_attrs,
  adoc! {r#"
    [#my-id.some-class]
    TIP: never start a land war in Asia
  "#},
  html! {r#"
    <div id="my-id" class="admonitionblock tip some-class">
      <table>
        <tr>
          <td class="icon">
            <div class="title">Tip</div>
          </td>
          <td class="content">
            never start a land war in Asia
          </td>
        </tr>
      </table>
    </div>
  "#}
);

test_eval!(
  inferred_doc_title_attr,
  adoc! {r#"
    = Doc _Title_

    foo {doctitle}
  "#},
   html_contains: "foo Doc _Title_"
);

test_eval!(
  explicit_doc_title_attr,
  adoc! {r#"
    = Doc _Title_
    :doctitle: Custom Title

    foo {doctitle}
  "#},
   html_contains: "foo Custom Title"
);

test_eval!(
  note_w_title,
  adoc! {r#"
    .Title
    NOTE: foo
  "#},
  html! {r#"
    <div class="admonitionblock note">
      <table>
        <tr>
          <td class="icon">
            <div class="title">Note</div>
          </td>
          <td class="content">
            <div class="title">Title</div>
            foo
          </td>
        </tr>
      </table>
    </div>
  "#}
);

test_eval!(
  image_macro,
  "image::name.png[]",
  html! {r#"
    <div class="imageblock">
      <div class="content">
        <img src="name.png" alt="name">
      </div>
    </div>
  "#}
);

test_eval!(
  image_w_title_and_attrs,
  adoc! {r#"
    .Title
    [#lol.rofl]
    image::cat.jpg[]
  "#},
  html! {r#"
    <div id="lol" class="imageblock rofl">
      <div class="content">
        <img src="cat.jpg" alt="cat">
      </div>
      <div class="title">Figure 1. Title</div>
    </div>
  "#}
);

test_eval!(
  quote_cite,
  adoc! {r#"
    [quote,,cite]
    foo bar
  "#},
  html! {r#"
    <div class="quoteblock">
      <blockquote>foo bar</blockquote>
      <div class="attribution">&#8212; cite</div>
    </div>
  "#}
);

test_eval!(
  quote_source,
  adoc! {r#"
    [quote,source]
    foo bar
  "#},
  html! {r#"
    <div class="quoteblock">
      <blockquote>foo bar</blockquote>
      <div class="attribution">&#8212; source</div>
    </div>
  "#}
);

test_eval!(
  quote_source_location,
  adoc! {r#"
    [quote,source,location]
    foo bar
  "#},
  html! {r#"
    <div class="quoteblock">
      <blockquote>foo bar</blockquote>
      <div class="attribution">
        &#8212; source<br>
        <cite>location</cite>
      </div>
    </div>
  "#}
);

test_eval!(
  complex_quote_example,
  adoc! {r#"
    .After landing the cloaked Klingon bird of prey in Golden Gate park:
    [quote,Captain James T. Kirk,Star Trek IV: The Voyage Home]
    Everybody remember where we parked.
  "#},
  html! {r#"
    <div class="quoteblock">
      <div class="title">After landing the cloaked Klingon bird of prey in Golden Gate park:</div>
      <blockquote>
        Everybody remember where we parked.
      </blockquote>
      <div class="attribution">
        &#8212; Captain James T. Kirk<br>
        <cite>Star Trek IV: The Voyage Home</cite>
      </div>
    </div>
  "#}
);

test_eval!(
  quoted_paragraph,
  adoc! {r#"
    "I hold it that a little rebellion now and then is a good thing,
    and as necessary in the political world as storms in the physical."
    -- Thomas Jefferson, Papers of Thomas Jefferson: Volume 11
  "#},
  html! {r#"
    <div class="quoteblock">
      <blockquote>
        I hold it that a little rebellion now and then is a good thing, and as necessary in the political world as storms in the physical.
      </blockquote>
      <div class="attribution">
        &#8212; Thomas Jefferson<br>
        <cite>Papers of Thomas Jefferson: Volume 11</cite>
      </div>
    </div>
  "#}
);

test_eval!(
  multiple_image_blocks_w_title,
  adoc! {r#"
    .Cat
    image::cat.png[]

    .Dog
    image::dog.png[]
  "#},
  html! {r#"
    <div class="imageblock">
      <div class="content">
        <img src="cat.png" alt="cat">
      </div>
      <div class="title">Figure 1. Cat</div>
    </div>
    <div class="imageblock">
      <div class="content">
        <img src="dog.png" alt="dog">
      </div>
      <div class="title">Figure 2. Dog</div>
    </div>
  "#}
);

test_eval!(
  fig_caption,
  adoc! {r#"
    = Doc Header
    :!figure-caption:

    .Cat
    image::cat.png[]

    .Dog
    image::dog.png[]
  "#},
  html! {r#"
    <div class="imageblock">
      <div class="content">
        <img src="cat.png" alt="cat">
      </div>
      <div class="title">Cat</div>
    </div>
    <div class="imageblock">
      <div class="content">
        <img src="dog.png" alt="dog">
      </div>
      <div class="title">Dog</div>
    </div>
  "#}
);

test_eval!(
  complex_image_block,
  adoc! {r#"
    .A mountain sunset
    [#img-sunset,link=https://www.flickr.com/photos/javh/5448336655]
    image::sunset.jpg[Sunset,200,100]
  "#},
  html! {r#"
    <div id="img-sunset" class="imageblock">
      <div class="content">
        <a class="image" href="https://www.flickr.com/photos/javh/5448336655">
          <img src="sunset.jpg" alt="Sunset" width="200" height="100">
        </a>
      </div>
      <div class="title">Figure 1. A mountain sunset</div>
    </div>
  "#}
);

test_eval!(
  change_fig_cap,
  adoc! {r#"
    .Title
    image::foo.png[]

    :!figure-caption:

    .Next
    image::bar.png[]
  "#},
  html! {r#"
    <div class="imageblock">
      <div class="content">
        <img src="foo.png" alt="foo">
      </div>
      <div class="title">Figure 1. Title</div>
    </div>
    <div class="imageblock">
      <div class="content">
        <img src="bar.png" alt="bar">
      </div>
      <div class="title">Next</div>
    </div>
  "#}
);

test_eval!(
  footnote,
  "foo.footnote:[bar _baz_]",
  html! {r##"
    <div class="paragraph">
      <p>foo.
        <sup class="footnote">
          [<a id="_footnoteref_1" class="footnote" href="#_footnotedef_1" title="View footnote.">1</a>]
        </sup>
      </p>
    </div>
    <div id="footnotes">
      <hr>
      <div class="footnote" id="_footnotedef_1">
        <a href="#_footnoteref_1">1</a>. bar <em>baz</em>
      </div>
    </div>
  "##}
);

test_eval!(
  two_footnotes_w_cust,
  adoc! {r#"
    foo.footnote:[bar _baz_]

    lol.footnote:cust[baz]
  "#},
  html! {r##"
    <div class="paragraph">
      <p>foo.
        <sup class="footnote">
          [<a id="_footnoteref_1" class="footnote" href="#_footnotedef_1" title="View footnote.">1</a>]
        </sup>
      </p>
    </div>
    <div class="paragraph">
      <p>lol.
        <sup class="footnote" id="_footnote_cust">
          [<a id="_footnoteref_2" class="footnote" href="#_footnotedef_2" title="View footnote.">2</a>]
        </sup>
      </p>
    </div>
    <div id="footnotes">
      <hr>
      <div class="footnote" id="_footnotedef_1">
        <a href="#_footnoteref_1">1</a>. bar <em>baz</em>
      </div>
      <div class="footnote" id="_footnotedef_2">
        <a href="#_footnoteref_2">2</a>. baz
      </div>
    </div>
  "##}
);

test_eval!(
  quote_newlines,
  adoc! {r#"
    "`foo
    bar`"
    baz
  "#},
  html! {r#"
    <div class="paragraph">
      <p>&#8220;foo bar&#8221; baz</p>
    </div>
  "#}
);

test_eval!(
  line_breaks,
  adoc! {r#"
    foo +
    bar

    [%hardbreaks]
    Ruby is red.
    Java is beige.

    normal
    breaks

    :hardbreaks-option:

    foo
    bar

    :!hardbreaks-option:

    bar
    baz
  "#},
  html! {r#"
    <div class="paragraph">
      <p>foo<br> bar</p>
    </div>
    <div class="paragraph">
      <p>Ruby is red.<br> Java is beige.</p>
    </div>
    <div class="paragraph">
      <p>normal breaks</p>
    </div>
    <div class="paragraph">
      <p>foo<br> bar</p>
    </div>
    <div class="paragraph">
      <p>bar baz</p>
    </div>
  "#}
);

test_eval!(
  simple_listing_block,
  adoc! {r#"
    [listing]
    foo `bar`
  "#},
  html! {r#"
    <div class="listingblock">
      <div class="content">
        <pre>foo `bar`</pre>
      </div>
    </div>
  "#}
);

test_eval!(
  delimited_unspaced_from_paragraph,
  adoc! {r#"
    foobar
    ====
    baz
    ====
  "#},
  html! {r#"
    <div class="paragraph">
      <p>foobar</p>
    </div>
    <div class="exampleblock">
      <div class="content">
        <div class="paragraph">
          <p>baz</p>
        </div>
      </div>
    </div>
  "#}
);

enum SubstrTest {
  Contains(&'static str),
  DoesNotContain(&'static str),
}

#[test]
fn test_head_opts() {
  use SubstrTest::*;
  let cases = vec![
    (":nolang:", DoesNotContain("lang=")),
    (":nolang:", Contains("<title>Doc Header</title>")),
    (
      ":title: Such Custom Title",
      Contains("<title>Such Custom Title</title>"),
    ),
    (":lang: es", Contains("lang=\"es\"")),
    (":encoding: latin1", Contains("charset=\"latin1\"")),
    (":reproducible:", DoesNotContain("generator")),
    (
      ":app-name: x",
      Contains(r#"<meta name="application-name" content="x">"#),
    ),
    (
      ":description: x",
      Contains(r#"<meta name="description" content="x">"#),
    ),
    (
      ":keywords: x, y",
      Contains(r#"<meta name="keywords" content="x, y">"#),
    ),
    (
      "Kismet R. Lee <kismet@asciidoctor.org>",
      Contains(r#"<meta name="author" content="Kismet R. Lee">"#),
    ),
    (
      "Kismet R. Lee <kismet@asciidoctor.org>; Bob Smith",
      Contains(r#"<meta name="author" content="Kismet R. Lee, Bob Smith">"#),
    ),
    (
      ":copyright: x",
      Contains(r#"<meta name="copyright" content="x">"#),
    ),
    (
      ":favicon:",
      Contains(r#"<link rel="icon" type="image/x-icon" href="favicon.ico">"#),
    ),
    (
      ":favicon: ./images/favicon/favicon.png",
      Contains(r#"<link rel="icon" type="image/png" href="./images/favicon/favicon.png">"#),
    ),
    (
      ":iconsdir: custom\n:favicon: {iconsdir}/my/icon.png",
      Contains(r#"<link rel="icon" type="image/png" href="custom/my/icon.png">"#),
    ),
  ];
  let bump = &Bump::new();

  for (opts, expectation) in cases {
    let input = format!("= Doc Header\n{}\n\nignore me\n\n", opts);
    let parser = Parser::new(bump, &input);
    let document = parser.parse().unwrap().document;
    let html = eval(&document, Opts::default(), AsciidoctorHtml::new()).unwrap();
    match expectation {
      Contains(s) => assert!(
        html.contains(s),
        "\n`{}` was NOT found when expected\n\n```adoc\n{}\n```\n\n```html\n{}\n```",
        s,
        input.trim(),
        html.replace('>', ">\n").trim()
      ),
      DoesNotContain(s) => assert!(
        !html.contains(s),
        "\n`{}` WAS found when not expected\n\n```adoc\n{}\n```\n\n```html\n{}\n```",
        s,
        input.trim(),
        html.replace('>', ">\n").trim()
      ),
    }
  }
  // one test with no doc header
  let parser = Parser::new(bump, "without doc header");
  let document = parser.parse().unwrap().document;
  let html = eval(&document, Opts::default(), AsciidoctorHtml::new()).unwrap();
  assert!(html.contains("<title>Untitled</title>"));
}

#[test]
fn test_non_embedded() {
  let input = adoc! {r#"
    = *Document* _title_
    Beyonce Smith; J Z <jz@you.com>

    foo
  "#};
  let expected = adoc! {r#"
    <!DOCTYPE html>
    <html lang="en">
      <head>
        <meta charset="UTF-8">
        <meta http-equiv="X-UA-Compatible" content="IE=edge">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <meta name="generator" content="Asciidork">
        <meta name="author" content="Beyonce Smith, J Z">
        <title>Document title</title>
      </head>
      <body class="article">
        <div id="header">
          <h1><strong>Document</strong> <em>title</em></h1>
          <div class="details">
            <span id="author" class="author">Beyonce Smith</span><br>
            <span id="author2" class="author">J Z</span><br>
            <span id="email2" class="email">
              <a href="mailto:jz@you.com">jz@you.com</a>
            </span><br>
          </div>
        </div>
        <div class="paragraph">
          <p>foo</p>
        </div>
      </body>
    </html>
  "#};
  let bump = &Bump::new();
  let re = Regex::new(r"(?m)\n\s*").unwrap();
  let expected = re.replace_all(expected, "");
  let parser = Parser::new(bump, input);
  let doc = parser.parse().unwrap().document;
  assert_eq!(
    eval(&doc, Opts::default(), AsciidoctorHtml::new()).unwrap(),
    expected,
    from: input
  );
}
