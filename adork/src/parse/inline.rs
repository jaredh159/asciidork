use crate::ast::Inline::{self, *};
use crate::parse::Parser;
use crate::tok::{self, Token, TokenType, TokenType::*};

impl Parser {
  pub(super) fn parse_inlines<B>(&self, block: B) -> Vec<Inline>
  where
    B: Into<tok::Block>,
  {
    let mut block: tok::Block = block.into();
    self.parse_inlines_until(&mut block, [])
  }

  fn parse_inlines_until<const N: usize>(
    &self,
    block: &mut tok::Block,
    stop_tokens: [TokenType; N],
  ) -> Vec<Inline> {
    let mut inlines = Vec::new();
    let mut text = Text::new();

    while let Some(mut line) = block.consume_current() {
      loop {
        if line.starts_with_seq(&stop_tokens) {
          line.consume::<N>();
          text.commit(&mut inlines);
          if !line.is_empty() {
            block.restore(line);
          }
          return inlines;
        }

        match line.consume_current() {
          Some(token) if token.is(Whitespace) => text.push_str(" "),

          Some(token) if token.is(Caret) && line.is_continuous_thru(Caret) => {
            text.commit(&mut inlines);
            block.restore(line);
            inlines.push(Superscript(self.parse_inlines_until(block, [Caret])));
            break;
          }

          Some(token) if token.is(Tilde) && line.is_continuous_thru(Tilde) => {
            text.commit(&mut inlines);
            block.restore(line);
            inlines.push(Subscript(self.parse_inlines_until(block, [Tilde])));
            break;
          }

          Some(token) if starts_unconstrained(Underscore, &token, &line, block) => {
            self.parse_unconstrained(Underscore, Italic, &mut text, &mut inlines, line, block);
            break;
          }

          Some(token) if starts_constrained(Underscore, &token, &line) => {
            self.parse_constrained(Underscore, Italic, &mut text, &mut inlines, line, block);
            break;
          }

          Some(token) if starts_unconstrained(Star, &token, &line, block) => {
            self.parse_unconstrained(Star, Bold, &mut text, &mut inlines, line, block);
            break;
          }

          Some(token) if starts_constrained(Star, &token, &line) => {
            self.parse_constrained(Star, Bold, &mut text, &mut inlines, line, block);
            break;
          }

          Some(token) if starts_unconstrained(Backtick, &token, &line, block) => {
            self.parse_unconstrained(Backtick, Mono, &mut text, &mut inlines, line, block);
            break;
          }

          Some(token) if starts_constrained(Backtick, &token, &line) => {
            self.parse_constrained(Backtick, Mono, &mut text, &mut inlines, line, block);
            break;
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

  fn parse_unconstrained(
    &self,
    token_type: TokenType,
    wrap: fn(Vec<Inline>) -> Inline,
    text: &mut Text,
    inlines: &mut Vec<Inline>,
    mut line: tok::Line,
    block: &mut tok::Block,
  ) {
    line.consume::<1>(); // second token
    text.commit(inlines);
    block.restore(line);
    inlines.push(wrap(self.parse_inlines_until(block, [token_type; 2])));
  }

  fn parse_constrained(
    &self,
    token_type: TokenType,
    wrap: fn(Vec<Inline>) -> Inline,
    text: &mut Text,
    inlines: &mut Vec<Inline>,
    line: tok::Line,
    block: &mut tok::Block,
  ) {
    text.commit(inlines);
    block.restore(line);
    inlines.push(wrap(self.parse_inlines_until(block, [token_type; 1])));
  }
}

fn starts_constrained(token_type: TokenType, token: &Token, line: &tok::Line) -> bool {
  token.is(token_type) && line.ends_constrained_inline(token_type)
}

fn starts_unconstrained(
  token_type: TokenType,
  token: &Token,
  line: &tok::Line,
  block: &tok::Block,
) -> bool {
  token.is(token_type)
    && line.current_is(token_type)
    && (line.contains_seq(&[token_type; 2]) || block.contains_seq(&[token_type; 2]))
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
      (
        "`*_foo_*`",
        vec![Mono(vec![Bold(vec![Italic(vec![t("foo")])])])],
      ),
      ("foo _bar_", vec![t("foo "), Italic(vec![t("bar")])]),
      ("foo *bar*", vec![t("foo "), Bold(vec![t("bar")])]),
      ("foo `bar`", vec![t("foo "), Mono(vec![t("bar")])]),
      (
        "foo __ba__r",
        vec![t("foo "), Italic(vec![t("ba")]), t("r")],
      ),
      ("foo **ba**r", vec![t("foo "), Bold(vec![t("ba")]), t("r")]),
      ("foo ``ba``r", vec![t("foo "), Mono(vec![t("ba")]), t("r")]),
      ("foo __bar", vec![t("foo __bar")]),
      ("foo ^bar^", vec![t("foo "), Superscript(vec![t("bar")])]),
      ("foo ^bar", vec![t("foo ^bar")]),
      ("foo bar^", vec![t("foo bar^")]),
      (
        "foo ~bar~ baz",
        vec![t("foo "), Subscript(vec![t("bar")]), t(" baz")],
      ),
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
