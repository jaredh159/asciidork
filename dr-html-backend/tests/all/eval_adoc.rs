use asciidork_dr_html_backend::{css, AsciidoctorHtml};
use asciidork_eval::eval;
use asciidork_parser::prelude::*;
use test_utils::*;

use regex::Regex;

assert_inline_html!(
  simple_inline_w_newline,
  adoc! {r#"
    _foo_
    bar
  "#},
  r#"<em>foo</em> bar"#
);

assert_inline_html!(
  nested_inlines,
  "`*_foo_*`",
  r#"<code><strong><em>foo</em></strong></code>"#
);

assert_inline_html!(
  biblio_anchor_out_of_place,
  "a [[[foo]]] bar",
  r#"a [<a id="foo"></a>] bar"#
);

assert_inline_html!(passthrough, "+_<foo>&_+", r#"_&lt;foo&gt;&amp;_"#);
assert_inline_html!(text_span, "[.foo]#bar#", r#"<span class="foo">bar</span>"#);
assert_inline_html!(passthrough_block, "[pass]\n_<foo>&_", "_<foo>&_");
assert_inline_html!(highlight, "foo #bar#", r#"foo <mark>bar</mark>"#);
assert_inline_html!(mono, "foo `bar`", r#"foo <code>bar</code>"#);
assert_inline_html!(passthrough_2, "rofl +_foo_+ lol", r#"rofl _foo_ lol"#);
assert_inline_html!(inline_passthrough, "+++_<foo>&_+++ bar", r#"_<foo>&_ bar"#);
assert_inline_html!(subscript, "foo ~bar~ baz", r#"foo <sub>bar</sub> baz"#);
assert_inline_html!(superscript, "foo ^bar^ baz", r#"foo <sup>bar</sup> baz"#);
assert_inline_html!(not_quotes, "foo `'bar'`", r#"foo <code>'bar'</code>"#);
assert_inline_html!(curly_quotes, "foo \"`bar`\"", r#"foo &#8220;bar&#8221;"#);
assert_inline_html!(implicit_apos, "Olaf's wrench", r#"Olaf&#8217;s wrench"#);
assert_inline_html!(multichar_whitespace, "foo   bar", r#"foo bar"#);
assert_inline_html!(litmono_attr_ref, "`+{name}+`", r#"<code>{name}</code>"#);
assert_inline_html!(not_implicit_apostrophe, "('foo')", r#"('foo')"#);

assert_inline_html!(
  not_two_passthrus_in_one_line,
  "`++`*`++` foo `+*+`",
  r#"<code>`*`</code> foo <code>*</code>"#
);

assert_inline_html!(
  passthru_inside_litmono,
  //     v----v -- inline passthru
  "foo `++a`b`++`",
  r#"foo <code>a`b`</code>"#
);

assert_inline_html!(
  not_passthrough,
  "`\\d+[a]\\d+[b]`",
  "<code>\\d+[a]\\d+[b]</code>"
);

assert_inline_html!(
  emdash_start_line_swallows_newline,
  "foo\n-- bar", // rx removes the newline in this case
  r#"foo&#8201;&#8212;&#8201;bar"#
);

assert_inline_html!(
  emdash_end_line_swallows_newline,
  "foo --\nbar", // rx removes the newline in this case
  r#"foo&#8201;&#8212;&#8201;bar"#
);

assert_inline_html!(
  confusing_combo,
  "`*` foo `*`",
  r#"<code>*</code> foo <code>*</code>"#
);

assert_inline_html!(
  char_replacements_symbols,
  "(C)(TM)(R)...->=><-<=",
  r#"&#169;&#8482;&#174;&#8230;&#8203;&#8594;&#8658;&#8592;&#8656;"#
);

assert_inline_html!(
  minus_subs,
  "[subs=-specialchars]\nfoo & _bar_",
  r#"foo & <em>bar</em>"#
);

assert_inline_html!(
  special_chars,
  "foo <bar> & lol",
  r#"foo &lt;bar&gt; &amp; lol"#
);

assert_inline_html!(
  replaces_punctionation,
  "John's Hideout is the Whites`' place... foo\\'bar",
  r#"John&#8217;s Hideout is the Whites&#8217; place&#8230;&#8203; foo'bar"#
);

assert_inline_html!(
  btn_macro,
  "press the btn:[OK] button",
  r#"press the <b class="button">OK</b> button"#
);

assert_html!(
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

assert_html!(
  menu_macro,
  "select menu:File[Save].",
  html! {r#"
    <div class="paragraph">
      <p>select <span class="menuseq"><span class="menu">File</span>&#160;&#9656;<span class="menuitem">Save</span></span>.</p>
    </div>
  "#}
);

assert_html!(
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

assert_html!(
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

assert_html!(
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

assert_html!(
  title,
  ".Title\nfoo",
  html! {r#"
    <div class="paragraph">
      <div class="title">Title</div>
      <p>foo</p>
    </div>
  "#}
);

assert_html!(
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

assert_html!(
  inferred_doc_title_attr,
  adoc! {r#"
    = Doc _Title_

    foo {doctitle}
  "#},
   // TODO: asciidoctor produces `foo Doc _Title_`
   // here, not sure if it matters though
   // might be a manifestation of ORDER of subs
   contains: "foo Doc <em>Title</em>"
);

assert_html!(
  explicit_doc_title_attr,
  adoc! {r#"
    = Doc _Title_
    :doctitle: Custom Title

    foo {doctitle}
  "#},
   contains: "foo Custom Title"
);

assert_html!(
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

assert_html!(
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

assert_html!(
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

assert_html!(
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

assert_html!(
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

assert_html!(
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

assert_html!(
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

assert_html!(
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

assert_html!(
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

assert_html!(
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

assert_html!(
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

assert_html!(
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

assert_html!(
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

assert_html!(
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

assert_html!(
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

assert_html!(
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

assert_html!(
  admonition_icons,
  adoc! {r#"
    NOTE: Tip #1

    :icons:

    NOTE: Tip #2

    :icons: font

    NOTE: Tip #3
  "#},
  html! {r#"
    <div class="admonitionblock note">
      <table>
        <tr>
          <td class="icon"><div class="title">Note</div></td>
          <td class="content">Tip #1</td>
        </tr>
      </table>
    </div>
    <div class="admonitionblock note">
      <table>
        <tr>
          <td class="icon"><img src="./images/icons/note.png" alt="Note"></td>
          <td class="content">Tip #2</td>
        </tr>
      </table>
    </div>
    <div class="admonitionblock note">
      <table>
        <tr>
          <td class="icon"><i class="fa icon-note" title="Note"></i></td>
          <td class="content">Tip #3</td>
        </tr>
      </table>
    </div>
  "#}
);

assert_html!(
  admonition_blocks,
  adoc! {r#"
    [NOTE]
    ====
    This is a note!
    ====
  "#},
  html! {r#"
    <div class="admonitionblock note">
      <table>
        <tr>
          <td class="icon"><div class="title">Note</div></td>
          <td class="content">
            <div class="paragraph"><p>This is a note!</p></div>
          </td>
        </tr>
      </table>
    </div>
  "#}
);

assert_html!(
  escaped_ifdef,
  adoc! {"
    \\ifdef::yup[]

    Some line

    \\endif::[]
  "},
  html! {r#"
    <div class="paragraph"><p>ifdef::yup[]</p></div>
    <div class="paragraph"><p>Some line</p></div>
    <div class="paragraph"><p>endif::[]</p></div>
  "#}
);

assert_html!(
  attr_ref_behavior,
  adoc! {r#"
    :attribute-missing: drop-line
    :foo: bar
    :Baz: qux

    foo bar
    whoops {missing}
    baz

    :attribute-missing: skip

    foo bar
    whoops {missing}
    baz

    {foo} {Foo} {Baz} {baz}
  "#},
  html! {r#"
    <div class="paragraph">
      <p>foo bar baz</p>
    </div>
    <div class="paragraph">
      <p>foo bar whoops {missing} baz</p>
    </div>
    <div class="paragraph">
      <p>bar bar qux qux</p>
    </div>
  "#}
);

assert_error!(
  missing_attr_ref,
  adoc! {"
    :attribute-missing: warn

    whoops {missing}
  "},
  error! {"
     --> test.adoc:3:8
      |
    3 | whoops {missing}
      |        ^^^^^^^^^ Skipping reference to missing attribute
  "}
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

  for (opts, expectation) in cases {
    let input = format!("= Doc Header\n{opts}\n\nignore me\n\n");
    let parser = test_parser!(&input);
    let document = parser.parse().unwrap().document;
    let html = eval(&document, AsciidoctorHtml::new()).unwrap();
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
  let parser = test_parser!("without doc header");
  let document = parser.parse().unwrap().document;
  let html = eval(&document, AsciidoctorHtml::new()).unwrap();
  assert!(html.contains("<title>Untitled</title>"));
}

#[test]
fn test_full_doc() {
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
        <link rel="stylesheet" href="https://fonts.googleapis.com/css?family=Open+Sans:300,300italic,400,400italic,600,600italic%7CNoto+Serif:400,400italic,700,700italic%7CDroid+Sans+Mono:400,700" />
        <style>{CSS}</style>
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
        <div id="content">
          <div class="paragraph"><p>foo</p></div>
        </div>
        <div id="footer"></div>
      </body>
    </html>
  "#};
  let re = Regex::new(r"(?m)\n\s*").unwrap();
  let expected = re.replace_all(expected, "");
  let expected = expected.replace("{CSS}", css::DEFAULT);
  let parser = test_parser!(input);
  let doc = parser.parse().unwrap().document;
  expect_eq!(
    eval(&doc, AsciidoctorHtml::new()).unwrap(),
    expected,
    from: input
  );
}
