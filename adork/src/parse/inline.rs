use crate::ast;
use crate::parse::Parser;
use crate::tok::{self, Token, TokenType::*};

impl Parser {
  pub(super) fn parse_inlines(&self, mut line: tok::Line) -> Vec<ast::Inline> {
    let mut inlines = Vec::new();
    loop {
      match line.consume_current() {
        Some(token) if token.is(Word) => inlines.push(self.gather_words(&token, &mut line)),
        _ => break,
      };
    }
    inlines
  }

  fn gather_words(&self, first: &Token, line: &mut tok::Line) -> ast::Inline {
    let mut text = self.lexeme_string(first);
    loop {
      match line.current_token() {
        Some(token) if token.is(Word) => text.push_str(self.lexeme_str(token)),
        Some(token) if token.is(Whitespace) => text.push_str(" "),
        _ => break,
      };
      line.next();
    }
    ast::Inline::Text(text)
  }
}

#[cfg(test)]
mod tests {
  use crate::ast;
  use crate::t;

  #[test]
  fn test_parse_inlines() {
    let (line, parser) = t::line_test("foo   bar\n");
    let inlines = parser.parse_inlines(line);
    assert_eq!(inlines, vec![ast::Inline::Text("foo bar".to_string())]);
  }
}
