// use asciidork_core::{JobAttr, JobSettings};
// use asciidork_eval::eval;
// use asciidork_parser::prelude::*;
use test_utils::*;

// use regex::Regex;

assert_inline_html!(
  simple_inline_w_newline,
  adoc! {r#"
    _foo_
    bar
  "#},
  "<em>foo</em>\nbar"
);

assert_inline_html!(
  nested_inlines,
  "`*_foo_*`",
  r#"<code><strong><em>foo</em></strong></code>"#
);

assert_inline_html!(
  biblio_anchor_out_of_place,
  "a [[[foo]]] bar",
  r#"a [<a id="foo" aria-hidden="true"></a>] bar"#
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
assert_inline_html!(curly_quotes, "foo \"`bar`\"", r#"foo &#x201c;bar&#x201d;"#);
assert_inline_html!(implicit_apos, "Olaf's wrench", r#"Olaf&#8217;s wrench"#);
assert_inline_html!(multichar_whitespace, "foo   bar", r#"foo   bar"#);
assert_inline_html!(litmono_attr_ref, "`+{name}+`", r#"<code>{name}</code>"#);
assert_inline_html!(not_implicit_apostrophe, "('foo')", r#"('foo')"#);

assert_inline_html!(
  emdash_start_line_swallows_newline,
  "foo\n-- bar", // rx removes the newline in this case
  r#"foo&#8201;&#8211;&#8201;bar"#
);

assert_inline_html!(
  emdash_end_line_swallows_newline,
  "foo --\nbar", // rx removes the newline in this case
  r#"foo&#8201;&#8211;&#8201;bar"#
);

assert_inline_html!(
  char_replacements_symbols,
  "(C)(TM)(R)...->=><-<=",
  r#"&#169;&#8482;&#174;&#8230;&#8203;&#8594;&#8658;&#8592;&#8656;"#
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
  attr_before_para,
  adoc! {r#"
    para 1

    :foo: bar
    para 2 {foo}

    :baz: foo \
    bar
    para 3 {baz}
  "#},
  html! {r#"
    <p>para 1</p>
    <p>para 2 bar</p>
    <p>para 3 foo bar</p>
  "#}
);

// // COMMENTED OUT: menu macro not supported in jirutka backend
// // assert_html!(
// //   menu_macro,
// //   "select menu:File[Save].",
// //   html! {r#"
// //     <div class="paragraph">
// //       <p>select <span class="menuseq"><span class="menu">File</span>&#160;&#9656;<span class="menuitem">Save</span></span>.</p>
// //     </div>
// //   "#}
// // );
//
// // COMMENTED OUT: menu macro not supported in jirutka backend
// // assert_html!(
// //   menu_macro_2,
// //   "select menu:File[Save > Reset].",
// //   html! {r#"
// //     <div class="paragraph">
// //       <p>
// //         select <span class="menuseq"
// //           ><span class="menu">File</span>&#160;&#9656;
// //           <span class="submenu">Save</span>&#160;&#9656;
// //           <span class="menuitem">Reset</span></span
// //         >.
// //       </p>
// //     </div>
// //   "#}
// // );
//
assert_html!(
  para_w_attrs,
  adoc! {r#"
    [#custom-id.custom-class]
    foo bar
  "#},
  html! {r#"
    <p id="custom-id" class="custom-class">foo bar</p>
  "#}
);

assert_html!(
  sidebar,
  "[sidebar]\nfoo bar",
  html! {r#"
    <aside class="sidebar">foo bar</aside>
  "#}
);

assert_html!(
  title,
  ".Title\nfoo",
  html! {r#"
    <section class="paragraph"><h6 class="block-title">Title</h6><p>foo</p></section>
  "#}
);

assert_html!(
  admonition_w_custom_attrs,
  adoc! {r#"
    [#my-id.some-class]
    TIP: never start a land war in Asia
  "#},
  html! {r#"
    <aside id="my-id" class="admonition-block tip some-class" role="doc-tip"><h6 class="block-title label-only"><span class="title-label">Tip: </span></h6><p>never start a land war in Asia</p></aside>
  "#}
);

assert_html!(
  note_w_title,
  adoc! {r#"
    .Title
    NOTE: foo
  "#},
  html! {r#"
    <aside class="admonition-block note" role="note">
      <h6 class="block-title"><span class="title-label">Note: </span>Title</h6>
      <p>foo</p>
    </aside>
  "#}
);

assert_html!(
  image_macro,
  "image::name.png[]",
  html! {r#"
    <div class="image-block">
      <img src="name.png" alt="name">
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
    <figure id="lol" class="image-block rofl">
      <img src="cat.jpg" alt="cat">
      <figcaption>Figure 1. Title</figcaption>
    </figure>
  "#}
);

assert_html!(
  quote_cite,
  adoc! {r#"
    [quote,,cite]
    foo bar
  "#},
  html! {r#"
    <div class="quote-block">
      <blockquote>
        <p>foo bar</p>
        <footer>&#8212; <cite>cite</cite></footer>
      </blockquote>
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
    <div class="quote-block">
      <blockquote>
        <p>foo bar</p>
        <footer>&#8212; <cite>source</cite></footer>
      </blockquote>
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
    <div class="quote-block">
      <blockquote>
        <p>foo bar</p><footer>&#8212; <cite>source, location</cite></footer>
      </blockquote>
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
    <section class="quote-block">
      <h6 class="block-title">After landing the cloaked Klingon bird of prey in Golden Gate park:</h6>
      <blockquote>
        <p>Everybody remember where we parked.</p>
        <footer>&#8212; <cite>Captain James T. Kirk, Star Trek IV: The Voyage Home</cite></footer>
      </blockquote>
    </section>
  "#}
);

assert_html!(
  quoted_paragraph_w_attr,
  adoc! {r#"
    "I hold it blah blah..."
    -- Thomas Jefferson https://site.com[Source]
  "#},
  html! {r#"
    <div class="quote-block">
      <blockquote>
        <p>I hold it blah blah&#8230;&#8203;</p>
        <footer>&#8212; <cite>Thomas Jefferson <a href="https://site.com">Source</a></cite></footer>
      </blockquote>
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
    <figure class="image-block">
      <img src="cat.png" alt="cat">
      <figcaption>Figure 1. Cat</figcaption>
    </figure>
    <figure class="image-block">
      <img src="dog.png" alt="dog">
      <figcaption>Figure 2. Dog</figcaption>
    </figure>
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
    <figure class="image-block">
      <img src="cat.png" alt="cat">
      <figcaption>Cat</figcaption>
    </figure>
    <figure class="image-block">
      <img src="dog.png" alt="dog">
      <figcaption>Dog</figcaption>
    </figure>
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
    <figure id="img-sunset" class="image-block">
      <a class="image" href="https://www.flickr.com/photos/javh/5448336655">
        <img src="sunset.jpg" alt="Sunset" width="200" height="100">
      </a>
      <figcaption>Figure 1. A mountain sunset</figcaption>
    </figure>
  "#}
);

assert_html!(
  quote_newlines,
  adoc! {r#"
    "`foo
    bar`"
    baz
  "#},
  html_e! {r#"
    <p>&#x201c;foo
    bar&#x201d;
    baz</p>"#}
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
  html_e! {r#"
    <p>foo<br>
    bar</p><p>Ruby is red.<br>
    Java is beige.</p><p>normal
    breaks</p><p>foo<br>
    bar</p><p>bar
    baz</p>"#}
);

assert_html!(
  simple_listing_block,
  adoc! {r#"
    [listing]
    foo `bar`
  "#},
  html! {r#"
    <div class="listing-block"><pre>foo `bar`</pre></div>
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
   <p>foobar</p>
   <div class="example-block">
     <div class="example"><p>baz</p></div>
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
    <aside class="admonition-block note" role="note">
      <h6 class="block-title label-only"><span class="title-label">Note: </span></h6>
      <p>Tip #1</p>
    </aside>
    <aside class="admonition-block note" role="note">
      <h6 class="block-title label-only"><span class="title-label">Note: </span></h6>
      <p>Tip #2</p>
    </aside>
    <aside class="admonition-block note" role="note">
      <h6 class="block-title label-only"><span class="title-label">Note: </span></h6>
      <p>Tip #3</p>
    </aside>
  "#}
);

assert_html!(
  admonition_blocks,
  adoc! {r#"
    [NOTE]
    ====
    This is a note!
    ====

    [TIP]
    ====
    This is a tip!
    ====

    [IMPORTANT]
    ====
    This is a important!
    ====

    [WARNING]
    ====
    This is a warning!
    ====

    [CAUTION]
    ====
    This is a caution!
    ====
  "#},
  html! {r#"
    <aside class="admonition-block note" role="note"><h6 class="block-title label-only"><span class="title-label">Note: </span></h6><p>This is a note!</p></aside>
    <aside class="admonition-block tip" role="doc-tip"><h6 class="block-title label-only"><span class="title-label">Tip: </span></h6><p>This is a tip!</p></aside>
    <section class="admonition-block important" role="doc-notice"><h6 class="block-title label-only"><span class="title-label">Important: </span></h6><p>This is a important!</p></section>
    <section class="admonition-block warning" role="doc-notice"><h6 class="block-title label-only"><span class="title-label">Warning: </span></h6><p>This is a warning!</p></section>
    <section class="admonition-block caution" role="doc-notice"><h6 class="block-title label-only"><span class="title-label">Caution: </span></h6><p>This is a caution!</p></section>
  "#}
);

// #[test]
// fn test_full_doc() {
//   let input = adoc! {r#"
//     = *Document* _title_
//     Beyonce Smith; J Z <jz@you.com>
//
//     foo
//   "#};
//   let expected = adoc! {r#"
//     <!DOCTYPE html>
//     <html lang="en">
//       <head>
//         <meta charset="UTF-8">
//         <meta http-equiv="X-UA-Compatible" content="IE=edge">
//         <meta name="viewport" content="width=device-width, initial-scale=1.0">
//         <meta name="generator" content="Asciidork">
//         <meta name="author" content="Beyonce Smith, J Z">
//         <title>Document title</title>
//         <link rel="stylesheet" href="https://fonts.googleapis.com/css?family=Open+Sans:300,300italic,400,400italic,600,600italic%7CNoto+Serif:400,400italic,700,700italic%7CDroid+Sans+Mono:400,700" />
//         <style>{CSS}</style>
//       </head>
//       <body class="article">
//         <div id="header">
//           <h1><strong>Document</strong> <em>title</em></h1>
//           <div class="details">
//             <span id="author" class="author">Beyonce Smith</span><br>
//             <span id="author2" class="author">J Z</span><br>
//             <span id="email2" class="email">
//               <a href="mailto:jz@you.com">jz@you.com</a>
//             </span><br>
//           </div>
//         </div>
//         <div id="content">
//           <div class="paragraph"><p>foo</p></div>
//         </div>
//         <div id="footer"></div>
//       </body>
//     </html>
//   "#};
//   let re = Regex::new(r"(?m)\n\s*").unwrap();
//   let expected = re.replace_all(expected, "");
//   // let expected = expected.replace("{CSS}", css::DEFAULT);
//   let parser = test_parser!(input);
//   let doc = parser.parse().unwrap().document;
//   expect_eq!(
//     eval(&doc, AsciidoctorHtml::new()).unwrap(),
//     expected,
//     from: input
//   );
// }
