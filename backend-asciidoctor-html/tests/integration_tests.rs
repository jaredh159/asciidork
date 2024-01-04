use asciidork_backend_asciidoctor_html::AsciidoctorHtml;
use asciidork_eval::eval;
use asciidork_parser::prelude::*;

use indoc::indoc;
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
      ".Title\nfoo",
      indoc! {r#"
        <div class="paragraph">
          <div class="title">Title</div>
          <p>foo</p>
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
  ];
  let bump = &Bump::new();
  let re = Regex::new(r"(?m)\n\s*").unwrap();
  for (input, expected) in cases {
    let expected = re.replace_all(expected, "");
    let parser = Parser::new(bump, input);
    let doc = parser.parse().unwrap().document;
    let asciidoctor_html = AsciidoctorHtml::new();
    assert_eq!(eval(doc, asciidoctor_html).unwrap(), expected);
  }
}

// #[test]
// fn test_isolate() {
//   let input = indoc! {r#"
//     .Title
//     image::foo.png[]
//     :!figure-caption:
//     .Next
//     image::bar.png[]
//   "#};
//   let expected = indoc! {r#"
//     <div class="imageblock">
//       <div class="content">
//         <img src="foo.png" alt="foo">
//       </div>
//       <div class="title">Figure 1. Title</div>
//     </div>
//     <div class="imageblock">
//       <div class="content">
//         <img src="bar.png" alt="bar">
//       </div>
//       <div class="title">Next</div>
//     </div>
//   "#};
//   let bump = &Bump::new();
//   let re = Regex::new(r"(?m)\n\s*").unwrap();
//   let expected = re.replace_all(expected, "");
//   let parser = Parser::new(bump, input);
//   let doc = parser.parse().unwrap().document;
//   let asciidoctor_html = AsciidoctorHtml::new();
//   assert_eq!(eval(doc, asciidoctor_html).unwrap(), expected);
// }
