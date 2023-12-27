use backend_asciidoctor::{eval, AsciidoctorHtml};
use parser::prelude::*;

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
      r#"<div class="paragraph"><div class="title">Title</div><p>foo</p></div>"#,
    ),
  ];
  let bump = &Bump::new();
  for (input, expected) in cases {
    let parser = Parser::new(bump, input);
    let doc = parser.parse().unwrap().document;
    let asciidoctor_html = AsciidoctorHtml::new();
    assert_eq!(eval(doc, asciidoctor_html).unwrap(), expected);
  }
}
