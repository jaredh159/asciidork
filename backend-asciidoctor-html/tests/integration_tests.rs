use asciidork_backend_asciidoctor_html::AsciidoctorHtml;
use asciidork_eval::{eval, Flags};
use asciidork_parser::prelude::*;

use indoc::indoc;
use pretty_assertions::assert_eq;
use regex::Regex;

#[test]
fn test_eval() {
  let cases = vec![
    (
      "_foo_\nbar\n\n",
      r#"<div class="paragraph"><p><em>foo</em> bar</p></div>"#,
    ),
    (
      "`*_foo_*`",
      r#"<div class="paragraph"><p><code><strong><em>foo</em></strong></code></p></div>"#,
    ),
    (
      "+_<foo>&_+",
      r#"<div class="paragraph"><p>_&lt;foo&gt;&amp;_</p></div>"#,
    ),
    (
      "foo #bar#",
      r#"<div class="paragraph"><p>foo <mark>bar</mark></p></div>"#,
    ),
    (
      "foo `bar`",
      r#"<div class="paragraph"><p>foo <code>bar</code></p></div>"#,
    ),
    (
      "rofl +_foo_+ lol",
      r#"<div class="paragraph"><p>rofl _foo_ lol</p></div>"#,
    ),
    (
      "+++_<foo>&_+++ bar",
      r#"<div class="paragraph"><p>_<foo>&_ bar</p></div>"#,
    ),
    (
      "foo ~bar~ baz",
      r#"<div class="paragraph"><p>foo <sub>bar</sub> baz</p></div>"#,
    ),
    (
      "foo ^bar^ baz",
      r#"<div class="paragraph"><p>foo <sup>bar</sup> baz</p></div>"#,
    ),
    (
      "foo `'bar'`",
      r#"<div class="paragraph"><p>foo <code>'bar'</code></p></div>"#,
    ),
    (
      "foo \"`bar`\"",
      r#"<div class="paragraph"><p>foo &#8220;bar&#8221;</p></div>"#,
    ),
    (
      "Olaf's wrench",
      r#"<div class="paragraph"><p>Olaf&#8217;s wrench</p></div>"#,
    ),
    (
      "foo   bar",
      r#"<div class="paragraph"><p>foo bar</p></div>"#,
    ),
    (
      "`+{name}+`",
      r#"<div class="paragraph"><p><code>{name}</code></p></div>"#,
    ),
    (
      "foo <bar> & lol",
      r#"<div class="paragraph"><p>foo &lt;bar&gt; &amp; lol</p></div>"#,
    ),
    (
      "press the btn:[OK] button",
      r#"<div class="paragraph"><p>press the <b class="button">OK</b> button</p></div>"#,
    ),
    (
      "select menu:File[Save].",
      indoc! {r#"
        <div class="paragraph">
          <p>select <span class="menuseq"><span class="menu">File</span>&#160;&#9656;<span class="menuitem">Save</span></span>.</p>
        </div>
      "#},
    ),
    (
      "select menu:File[Save > Reset].",
      indoc! {r#"
        <div class="paragraph">
          <p>
            select <span class="menuseq"
              ><span class="menu">File</span>&#160;&#9656;
              <span class="submenu">Save</span>&#160;&#9656;
              <span class="menuitem">Reset</span></span
            >.
          </p>
        </div>
      "#},
    ),
    (
      "[sidebar]\nfoo bar",
      indoc! {r#"
        <div class="sidebarblock">
          <div class="content">
            foo bar
          </div>
        </div>
      "#},
    ),
    (
      ".Title\nfoo",
      indoc! {r#"
        <div class="paragraph">
          <div class="title">Title</div>
          <p>foo</p>
        </div>
      "#},
    ),
    (
      "--\nfoo\n--",
      indoc! {r#"
        <div class="openblock">
          <div class="content">
            <div class="paragraph">
              <p>foo</p>
            </div>
          </div>
        </div>
      "#},
    ),
    (
      "====\nfoo\n====",
      indoc! {r#"
        <div class="exampleblock">
          <div class="content">
            <div class="paragraph">
              <p>foo</p>
            </div>
          </div>
        </div>
      "#},
    ),
    (
      "[#my-id.some-class]\nTIP: never start a land war in Asia",
      indoc! {r#"
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
      "#},
    ),
    (
      ".Title\nNOTE: foo",
      indoc! {r#"
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
      "#},
    ),
    (
      "image::name.png[]",
      indoc! {r#"
        <div class="imageblock">
          <div class="content">
            <img src="name.png" alt="name">
          </div>
        </div>
      "#},
    ),
    (
      ".Title\n[#lol.rofl]\nimage::cat.jpg[]",
      indoc! {r#"
        <div id="lol" class="imageblock rofl">
          <div class="content">
            <img src="cat.jpg" alt="cat">
          </div>
          <div class="title">Figure 1. Title</div>
        </div>
      "#},
    ),
    (
      indoc! {r#"
        ****
        --
        foo
        --
        ****
      "#},
      indoc! {r#"
        <div class="sidebarblock">
          <div class="content">
            <div class="openblock">
              <div class="content">
                <div class="paragraph">
                  <p>foo</p>
                </div>
              </div>
            </div>
          </div>
        </div>
      "#},
    ),
    (
      indoc! {r#"
        .Cat
        image::cat.png[]

        .Dog
        image::dog.png[]
      "#},
      indoc! {r#"
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
      "#},
    ),
    (
      indoc! {r#"
        = Doc Header
        :!figure-caption:

        .Cat
        image::cat.png[]

        .Dog
        image::dog.png[]
      "#},
      indoc! {r#"
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
      "#},
    ),
    (
      indoc! {r#"
        .A mountain sunset
        [#img-sunset,link=https://www.flickr.com/photos/javh/5448336655]
        image::sunset.jpg[Sunset,200,100]
      "#},
      indoc! {r#"
        <div id="img-sunset" class="imageblock">
          <div class="content">
            <a class="image" href="https://www.flickr.com/photos/javh/5448336655">
              <img src="sunset.jpg" alt="Sunset" width="200" height="100">
            </a>
          </div>
          <div class="title">Figure 1. A mountain sunset</div>
        </div>
      "#},
    ),
    (
      indoc! {r#"
        .Title
        image::foo.png[]

        :!figure-caption:

        .Next
        image::bar.png[]
      "#},
      indoc! {r#"
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
      "#},
    ),
    (
      "foo.footnote:[bar _baz_]",
      indoc! {r##"
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
      "##},
    ),
    (
      indoc! {r#"
        ****
        This is content in a sidebar block.

        image::name.png[]

        This is more content in the sidebar block.
        ****
      "#},
      indoc! {r#"
        <div class="sidebarblock">
          <div class="content">
            <div class="paragraph">
              <p>This is content in a sidebar block.</p>
            </div>
            <div class="imageblock">
              <div class="content">
                <img src="name.png" alt="name">
              </div>
            </div>
            <div class="paragraph">
              <p>This is more content in the sidebar block.</p>
            </div>
          </div>
        </div>
      "#},
    ),
    (
      indoc! {r#"
      foo.footnote:[bar _baz_]

      lol.footnote:cust[baz]
    "#},
      indoc! {r##"
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
    "##},
    ),
  ];
  let bump = &Bump::new();
  let re = Regex::new(r"(?m)\n\s*").unwrap();
  for (input, expected) in cases {
    let expected = re.replace_all(expected, "");
    let parser = Parser::new(bump, input);
    let doc = parser.parse().unwrap().document;
    assert_eq!(
      eval(doc, Flags::embedded(), AsciidoctorHtml::new()).unwrap(),
      expected,
      "input was\n\n{}",
      input
    );
  }
}

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
    let html = eval(document, Flags::default(), AsciidoctorHtml::new()).unwrap();
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
  let html = eval(document, Flags::default(), AsciidoctorHtml::new()).unwrap();
  assert!(html.contains("<title>Untitled</title>"));
}

#[test]
fn test_non_embedded() {
  let input = indoc! {r#"
    = *Document* _title_

    foo
  "#};
  let expected = indoc! {r##"
    <!DOCTYPE html>
    <html lang="en">
      <head>
        <meta charset="UTF-8">
        <meta http-equiv="X-UA-Compatible" content="IE=edge">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <meta name="generator" content="Asciidork">
        <title>Document title</title>
      </head>
      <body>
        <div class="paragraph">
          <p>foo</p>
        </div>
      </body>
    </html>
  "##};
  let bump = &Bump::new();
  let re = Regex::new(r"(?m)\n\s*").unwrap();
  let expected = re.replace_all(expected, "");
  let parser = Parser::new(bump, input);
  let doc = parser.parse().unwrap().document;
  assert_eq!(
    eval(doc, Flags::default(), AsciidoctorHtml::new()).unwrap(),
    expected,
    "input was {}",
    input
  );
}

#[test]
fn test_isolate() {
  let input = indoc! {r#"
    foo
  "#};
  let expected = indoc! {r##"
    <!DOCTYPE html>
    <html lang="en">
      <head>
        <meta charset="UTF-8">
        <meta http-equiv="X-UA-Compatible" content="IE=edge">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <meta name="generator" content="Asciidork">
        <title>Untitled</title>
      </head>
      <body>
        <div class="paragraph">
          <p>foo</p>
        </div>
      </body>
    </html>
  "##};
  let bump = &Bump::new();
  let re = Regex::new(r"(?m)\n\s*").unwrap();
  let expected = re.replace_all(expected, "");
  let parser = Parser::new(bump, input);
  let doc = parser.parse().unwrap().document;
  assert_eq!(
    eval(doc, Flags::default(), AsciidoctorHtml::new()).unwrap(),
    expected,
    "input was {}",
    input
  );
}
