use crate::ast::Inline::{self, *};
use crate::parse::utils::Text;
use crate::parse::{Parser, Result};
use crate::tok::{self, Token, TokenType, TokenType::*};

impl Parser {
  pub(super) fn parse_inlines<B>(&mut self, block: B) -> Result<Vec<Inline>>
  where
    B: Into<tok::Block>,
  {
    let mut block: tok::Block = block.into();
    self.parse_inlines_until(&mut block, [])
  }

  fn parse_inlines_until<const N: usize>(
    &mut self,
    block: &mut tok::Block,
    stop_tokens: [TokenType; N],
  ) -> Result<Vec<Inline>> {
    let mut inlines = Vec::new();
    let mut text = Text::new();

    while let Some(mut line) = block.consume_current() {
      loop {
        if line.starts_with_seq(&stop_tokens) {
          line.discard(N);
          text.commit_inlines(&mut inlines);
          if !line.is_empty() {
            block.restore(line);
          }
          return Ok(inlines);
        }

        match line.consume_current() {
          Some(token) if token.is(Whitespace) => text.push_str(" "),

          Some(token) if token.is(OpenBracket) && line.contains_seq(&[CloseBracket, Hash]) => {
            text.commit_inlines(&mut inlines);
            let attr_list = self.parse_formatted_text_attr_list(&mut line)?;
            debug_assert!(line.current_is(Hash));
            line.discard(1); // `#`
            block.restore(line);
            inlines.push(TextSpan(
              attr_list,
              self.parse_inlines_until(block, [Hash])?,
            ));
            break;
          }

          Some(token) if token.is(Caret) && line.is_continuous_thru(Caret) => {
            text.commit_inlines(&mut inlines);
            block.restore(line);
            inlines.push(Superscript(self.parse_inlines_until(block, [Caret])?));
            break;
          }

          Some(token) if token.is(Tilde) && line.is_continuous_thru(Tilde) => {
            text.commit_inlines(&mut inlines);
            block.restore(line);
            inlines.push(Subscript(self.parse_inlines_until(block, [Tilde])?));
            break;
          }

          Some(token)
            if token.is(DoubleQuote)
              && line.current_is(Backtick)
              && starts_constrained(&[Backtick, DoubleQuote], &token, &line, block) =>
          {
            line.discard(1); // backtick
            text.push_str("“");
            text.commit_inlines(&mut inlines);
            block.restore(line);
            let mut quoted = self.parse_inlines_until(block, [Backtick, DoubleQuote])?;
            merge_appending(&mut inlines, &mut quoted, "”");
            break;
          }

          Some(token)
            if token.is(SingleQuote)
              && line.current_is(Backtick)
              && starts_constrained(&[Backtick, SingleQuote], &token, &line, block) =>
          {
            line.discard(1); // backtick
            text.push_str("‘");
            text.commit_inlines(&mut inlines);
            block.restore(line);
            let mut quoted = self.parse_inlines_until(block, [Backtick, SingleQuote])?;
            merge_appending(&mut inlines, &mut quoted, "’");
            break;
          }

          Some(token)
            if token.is(Backtick)
              && line.current_is(Plus)
              && contains_seq(&[Plus, Backtick], &line, block) =>
          {
            line.discard(1);
            text.commit_inlines(&mut inlines);
            block.restore(line);
            inlines.push(LitMono(self.unsubstituted_until(block, &[Plus, Backtick])));
            break;
          }

          Some(token) if token.is(Hash) && contains_seq(&[Hash], &line, block) => {
            text.commit_inlines(&mut inlines);
            block.restore(line);
            inlines.push(Highlight(self.parse_inlines_until(block, [Hash])?));
            break;
          }

          Some(token) if starts_unconstrained(Underscore, &token, &line, block) => {
            self.parse_unconstrained(Underscore, Italic, &mut text, &mut inlines, line, block)?;
            break;
          }

          Some(token) if starts_constrained(&[Underscore], &token, &line, block) => {
            self.parse_constrained(Underscore, Italic, &mut text, &mut inlines, line, block)?;
            break;
          }

          Some(token) if starts_unconstrained(Star, &token, &line, block) => {
            self.parse_unconstrained(Star, Bold, &mut text, &mut inlines, line, block)?;
            break;
          }

          Some(token) if starts_constrained(&[Star], &token, &line, block) => {
            self.parse_constrained(Star, Bold, &mut text, &mut inlines, line, block)?;
            break;
          }

          Some(token) if starts_unconstrained(Backtick, &token, &line, block) => {
            self.parse_unconstrained(Backtick, Mono, &mut text, &mut inlines, line, block)?;
            break;
          }

          Some(token) if starts_constrained(&[Backtick], &token, &line, block) => {
            self.parse_constrained(Backtick, Mono, &mut text, &mut inlines, line, block)?;
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
    text.commit_inlines(&mut inlines);

    Ok(inlines)
  }

  fn parse_unconstrained(
    &mut self,
    token_type: TokenType,
    wrap: fn(Vec<Inline>) -> Inline,
    text: &mut Text,
    inlines: &mut Vec<Inline>,
    mut line: tok::Line,
    block: &mut tok::Block,
  ) -> Result<()> {
    line.discard(1); // second token
    text.commit_inlines(inlines);
    block.restore(line);
    inlines.push(wrap(self.parse_inlines_until(block, [token_type; 2])?));
    Ok(())
  }

  fn parse_constrained(
    &mut self,
    token_type: TokenType,
    wrap: fn(Vec<Inline>) -> Inline,
    text: &mut Text,
    inlines: &mut Vec<Inline>,
    line: tok::Line,
    block: &mut tok::Block,
  ) -> Result<()> {
    text.commit_inlines(inlines);
    block.restore(line);
    inlines.push(wrap(self.parse_inlines_until(block, [token_type; 1])?));
    Ok(())
  }

  fn unsubstituted_until(&self, block: &mut tok::Block, stop_tokens: &[TokenType]) -> String {
    let mut unsubstituted = String::new();
    while let Some(mut line) = block.consume_current() {
      loop {
        if line.starts_with_seq(stop_tokens) {
          line.discard(stop_tokens.len());
          if !line.is_empty() {
            block.restore(line);
          }
          return unsubstituted;
        }

        match line.consume_current() {
          Some(token) => unsubstituted.push_str(self.lexeme_str(&token)),
          None => unsubstituted.push('\n'),
        }
      }
    }
    unsubstituted
  }
}

fn starts_constrained(
  stop_tokens: &[TokenType],
  token: &Token,
  line: &tok::Line,
  block: &mut tok::Block,
) -> bool {
  debug_assert!(!stop_tokens.is_empty());
  token.is(*stop_tokens.last().unwrap())
    && (line.terminates_constrained(stop_tokens) || block.terminates_constrained(stop_tokens))
}

fn starts_unconstrained(
  token_type: TokenType,
  token: &Token,
  line: &tok::Line,
  block: &tok::Block,
) -> bool {
  token.is(token_type) && line.current_is(token_type) && contains_seq(&[token_type; 2], line, block)
}

fn contains_seq(seq: &[TokenType], line: &tok::Line, block: &tok::Block) -> bool {
  line.contains_seq(seq) || block.contains_seq(seq)
}

fn merge_appending(a: &mut Vec<Inline>, b: &mut Vec<Inline>, append: &str) {
  if let (Some(Inline::Text(a_text)), Some(Inline::Text(b_text))) = (a.last_mut(), b.first_mut()) {
    a_text.push_str(b_text);
    b.remove(0);
  }
  a.append(b);
  if let Some(Inline::Text(text)) = a.last_mut() {
    text.push_str(append);
  } else {
    a.push(Inline::Text(append.to_string()));
  }
}

impl Text {
  fn commit_inlines(&mut self, inlines: &mut Vec<Inline>) {
    if !self.is_empty() {
      inlines.push(Inline::Text(self.take()));
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::ast::{AttrList, Inline::*};
  use crate::t::*;

  #[test]
  fn test_parse_inlines() {
    let cases = vec![
      (
        "foo [.nowrap]#bar#",
        vec![
          t("foo "),
          TextSpan(
            AttrList {
              roles: vec![s("nowrap")],
              ..AttrList::new()
            },
            vec![t("bar")],
          ),
        ],
      ),
      (
        "`*_foo_*`",
        vec![Mono(vec![Bold(vec![Italic(vec![t("foo")])])])],
      ),
      ("foo _bar_", vec![t("foo "), Italic(vec![t("bar")])]),
      ("foo _bar baz_", vec![t("foo "), Italic(vec![t("bar baz")])]),
      (
        "foo _bar\nbaz_",
        vec![t("foo "), Italic(vec![t("bar baz")])],
      ),
      ("foo 'bar'", vec![t("foo 'bar'")]),
      ("foo \"bar\"", vec![t("foo \"bar\"")]),
      ("foo \"`bar`\"", vec![t("foo “bar”")]),
      ("foo \"`bar baz`\"", vec![t("foo “bar baz”")]),
      ("foo \"`bar\nbaz`\"", vec![t("foo “bar baz”")]),
      ("foo '`bar`'", vec![t("foo ‘bar’")]),
      ("foo '`bar baz`'", vec![t("foo ‘bar baz’")]),
      ("foo '`bar\nbaz`'", vec![t("foo ‘bar baz’")]),
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
      ("foo #bar#", vec![t("foo "), Highlight(vec![t("bar")])]),
      ("foo ^bar", vec![t("foo ^bar")]),
      ("foo bar^", vec![t("foo bar^")]),
      (
        "foo ~bar~ baz",
        vec![t("foo "), Subscript(vec![t("bar")]), t(" baz")],
      ),
      ("foo   bar\n", vec![t("foo bar")]),
      ("foo bar", vec![t("foo bar")]),
      ("foo   bar\nbaz", vec![t("foo bar baz")]),
      ("`+{name}+`", vec![LitMono(s("{name}"))]),
      ("`+_foo_+`", vec![LitMono(s("_foo_"))]),
    ];

    for (input, expected) in cases {
      let (block, mut parser) = block_test(input);
      let inlines = parser.parse_inlines(block).unwrap();
      assert_eq!(inlines, expected);
    }
  }
}
