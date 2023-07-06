use crate::parse::line::Line;
use crate::parse::Parser;
use crate::token::{Token, TokenType::*};
use std::io::BufRead;

// An inline element is a span of content within a block element or one of its attributes (e.g., a block title). Inline elements include formatted text (italic, bold, etc), inline macros, and element references. What fills in the gap between these elements is unsubstituted text. Inline elements are less structured than block elements as they are more geared towards substitutions than a tree structure.
#[derive(Debug, PartialEq, Eq)]
pub enum Inline {
  Text(String),
  Bold(Vec<Inline>),
  Italic(Vec<Inline>),
  Mono(String),
  LitMono(String),
}

impl<R: BufRead> Parser<R> {
  pub(super) fn parse_inlines(&self, mut line: Line) -> Vec<Inline> {
    let mut inlines = Vec::new();
    loop {
      match line.consume_current() {
        Some(token) if token.is(Word) => inlines.push(self.gather_text(&token, &mut line)),
        _ => break,
      };
    }
    inlines
  }

  fn gather_text(&self, first: &Token, line: &mut Line) -> Inline {
    let mut text = self.lexer.lexeme(first).to_string();
    loop {
      match line.current_token() {
        Some(token) if token.is(Word) => text.push_str(self.lexer.lexeme(token)),
        Some(token) if token.is(Whitespace) => text.push_str(" "),
        _ => break,
      };
      line.next();
    }
    Inline::Text(text)
  }
}

#[cfg(test)]
mod tests {
  use crate::{parse::inline::Inline, t};

  #[test]
  fn test_parse_inlines() {
    let (line, parser) = t::line_test("foo   bar\n");
    let inlines = parser.parse_inlines(line);
    assert_eq!(inlines, vec![Inline::Text("foo bar".to_string())]);
  }
}
