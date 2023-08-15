use crate::ast::Inline;
use crate::parse::Parser;
use crate::tok::{self, Token, TokenType::*};

impl Parser {
  pub(super) fn parse_inlines<B>(&self, block: B) -> Vec<Inline>
  where
    B: Into<tok::Block>,
  {
    let mut block: tok::Block = block.into();
    let mut inlines = Vec::new();
    let mut text = Text::new();

    while let Some(mut line) = block.consume_current() {
      loop {
        match line.consume_current() {
          Some(token) if token.is(Whitespace) => text.push_str(" "),
          Some(token) if token.is(Caret) && line.is_continuous_thru(Caret) => {
            text.commit(&mut inlines);
            let superscript = line.extract_until(Caret);
            inlines.push(Inline::Superscript(self.parse_inlines(superscript)));
            line.consume_current(); // consume the second caret
          }
          Some(token) => text.push_token(&token, self),
          None => {
            text.push_str(" "); // join lines with space
            break;
          }
        }
      }
    }

    text.trim_end(); // remove last space from EOL
    text.commit(&mut inlines);

    inlines
  }

  fn gather_words(&self, first: &Token, line: &mut tok::Line) -> Inline {
    let mut text = self.lexeme_string(first);
    loop {
      match line.current_token() {
        Some(token) if token.is(Word) => text.push_str(self.lexeme_str(token)),
        Some(token) if token.is(Whitespace) => text.push_str(" "),
        _ => break,
      };
      line.next();
    }
    Inline::Text(text)
  }
}

struct Text(Option<String>);

impl Text {
  fn new() -> Text {
    Text(Some(String::new()))
  }

  fn push_str(&mut self, s: &str) {
    self.0.as_mut().unwrap().push_str(s);
  }

  fn push_token(&mut self, token: &Token, parser: &Parser) {
    self.push_str(parser.lexeme_str(token));
  }

  fn trim_end(&mut self) {
    if self.0.as_ref().unwrap().ends_with(' ') {
      self.0.as_mut().unwrap().pop();
    }
  }

  fn is_empty(&self) -> bool {
    self.0.as_ref().unwrap().len() == 0
  }

  fn commit(&mut self, inlines: &mut Vec<Inline>) {
    if !self.is_empty() {
      let text = self.0.replace(String::new()).unwrap();
      inlines.push(Inline::Text(text));
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::ast::Inline::*;
  use crate::t::*;

  #[test]
  fn test_parse_inlines() {
    let cases = vec![
      ("foo ^bar^", vec![t("foo "), Superscript(vec![t("bar")])]),
      ("foo ^bar", vec![t("foo ^bar")]),
      ("foo bar^", vec![t("foo bar^")]),
      ("foo   bar\n", vec![t("foo bar")]),
      ("foo bar", vec![t("foo bar")]),
      ("foo   bar\nbaz", vec![t("foo bar baz")]),
    ];
    for (input, expected) in cases {
      let (block, parser) = block_test(input);
      let inlines = parser.parse_inlines(block);
      assert_eq!(inlines, expected);
    }
  }
}
