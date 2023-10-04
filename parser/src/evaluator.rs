use crate::parser::Node;

pub fn eval(node: Node) -> String {
  format!("<div>{}</div>", node.text)
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::parser::Parser;
  use bumpalo::Bump;

  #[test]
  fn test_eval() {
    let bump = &Bump::new();
    let mut parser = Parser::new(bump, "hello");
    let node = parser.parse();
    assert_eq!(eval(node), "<div>hello</div>");
  }
}
