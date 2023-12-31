pub use indoc::indoc;

pub fn format_html(input: &str) -> String {
  let mut out = String::with_capacity(input.len());
  let dom = tl::parse(input, tl::ParserOptions::default()).unwrap();
  format_nodes(
    0,
    dom.children().iter().map(|h| h.get(dom.parser()).unwrap()),
    dom.parser(),
    &mut out,
  );
  out
}

fn format_nodes<'a>(
  depth: usize,
  nodes: impl Iterator<Item = &'a tl::Node<'a>>,
  parser: &tl::Parser,
  out: &mut String,
) {
  println!("depth: {}", depth);
  for (i, node) in nodes.enumerate() {
    eprintln!("n: {}, out: {}", i, out);
    if let Some(raw) = node.as_raw() {
      out.push_str(&raw.as_utf8_str());
      continue;
    }
    let Some(tag) = node.as_tag() else {
      continue;
    };
    if i > 0 {
      out.push('\n');
    }
    let tag_name = &tag.name().as_utf8_str();
    out.push_str(&"  ".repeat(depth));
    out.push('<');
    out.push_str(tag_name);
    tag.attributes().iter().for_each(|(key, value)| {
      out.push(' ');
      out.push_str(&key);
      if let Some(value) = value {
        out.push_str("=\"");
        out.push_str(&value);
        out.push('"');
      }
    });
    out.push('>');
    if let Some(children) = node.children() {
      let nodes = children.all(parser);
      // if nodes.len() == 1 {
      //   let inner_html = &tag.inner_html(parser);
      //   if inner_html.len() > 20 {
      //     out.push('\n');
      //     out.push_str(&"  ".repeat(depth + 1));
      //   }
      //   out.push_str(inner_html);
      //   if inner_html.len() > 20 {
      //     out.push('\n');
      //     out.push_str(&"  ".repeat(depth));
      //   }
      //   out.push_str("</");
      //   out.push_str(tag_name);
      //   out.push('>');
      // } else {
      //   out.push('\n');
      // dbg!(nodes);
      format_nodes(depth + 1, nodes.iter(), parser, out);
      // out.push('\n');
      // out.push_str(&"  ".repeat(depth));
      out.push_str("</");
      out.push_str(tag_name);
      out.push('>');
      // }
    }
  }
  // if out.ends_with('\n') {
  //   out.pop();
  // }
}

// fn newline(inline: bool, out: &mut String) {
//   if !inline {
//     out.push('\n');
//   }
// }

// fn newline_and_indent(inline: bool, depth: usize, out: &mut String) {
//   if !inline {
//     out.push('\n');
//     out.push_str(&"  ".repeat(depth));
//   }
// }

#[cfg(test)]
mod tests {
  use super::*;
  use indoc::indoc;

  #[test]
  fn test_format_html() {
    let case = vec![
      (
        r#"<p>foo</p> bar"#,
        indoc! {r#"
          <p>foo</p> bar
        "#},
      ),
      // (
      //   r#"<p>foo</p><p>bar</p>"#,
      //   indoc! {r#"
      //     <p>foo</p>
      //     <p>bar</p>
      //   "#},
      // ),
      // (
      //   r#"<div class="p-1"><p>hello</p></div>"#,
      //   indoc! {r#"
      //     <div class="p-1">
      //       <p>hello</p>
      //     </div>
      //   "#},
      // ),
      // (
      //   r#"<div class="p-1"><p>foo</p><p>bar</p></div>"#,
      //   indoc! {r#"
      //     <div class="p-1">
      //       <p>hello</p>
      //     </div>
      //   "#},
      // ),
      // (
      //   r#"<div class="p-1"><p>hello my name is jared henderson</p></div>"#,
      //   indoc! {r#"
      //     <div class="p-1">
      //       <p>
      //         hello my name is jared henderson
      //       </p>
      //     </div>
      //   "#},
      // ),
      // (
      //   r#"<div class="paragraph"><p><em>foo</em> bar</p></div>"#,
      //   indoc! {r#"
      //     <div class="paragraph">
      //       <p>
      //         <em>foo</em> bar
      //       </p>
      //     </div>
      //   "#},
      // ),
    ];
    for (input, output) in case {
      assert_eq!(format_html(input), output.trim());
    }
  }
}
