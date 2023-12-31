use backend_asciidoctor::{eval, AsciidoctorHtml};
use parser::prelude::*;

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
