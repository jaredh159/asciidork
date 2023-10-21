use bumpalo::collections::{String, Vec};

use crate::ast::{AttrList, Inline, Inline::*, Macro};
use crate::block::Block;
use crate::line::Line;
use crate::parser::Substitutions;
use crate::tasks::utils::Text;
use crate::token::{Token, TokenIs, TokenKind, TokenKind::*};
use crate::{Parser, Result};

impl<'alloc, 'src> Parser<'alloc, 'src> {
  pub(super) fn parse_inlines(
    &mut self,
    mut block: Block<'alloc, 'src>,
  ) -> Result<Vec<'alloc, Inline<'alloc>>> {
    self.parse_inlines_until(&mut block, &[])
  }

  fn parse_inlines_until(
    &mut self,
    block: &mut Block<'alloc, 'src>,
    stop_tokens: &[TokenKind],
  ) -> Result<Vec<'alloc, Inline<'alloc>>> {
    let mut inlines = Vec::new_in(self.allocator);
    let mut text = Text::new_in(self.allocator);
    let subs = self.ctx.subs;
    while let Some(mut line) = block.consume_current() {
      loop {
        if line.starts_with_seq(stop_tokens) {
          line.discard(stop_tokens.len());
          text.commit_inlines(&mut inlines);
          if !line.is_empty() {
            block.restore(line);
          }
          return Ok(inlines);
        }

        match line.consume_current() {
          Some(token) if token.is(Whitespace) => text.push_str(" "),

          Some(token) if subs.macros && token.is(MacroName) && line.continues_inline_macro() => {
            match token.lexeme {
              "image:" => {
                let target = line.consume_macro_target(self.allocator);
                let attr_list = self.parse_attr_list(&mut line)?;
                text.commit_inlines(&mut inlines);
                inlines.push(Macro(Macro::Image(target, attr_list)));
              }
              "kbd:" => {
                line.discard(1); // `[`
                let attr_list = self.parse_attr_list(&mut line)?;
                text.commit_inlines(&mut inlines);
                inlines.push(Macro(Macro::Keyboard(attr_list)));
              }
              "footnote:" => {
                let id = line.consume_optional_macro_target(self.allocator);
                let attr_list = self.parse_attr_list(&mut line)?;
                text.commit_inlines(&mut inlines);
                inlines.push(Macro(Macro::Footnote(id, attr_list)));
              }
              _ => text.push_token(&token),
            }
          }

          Some(token)
            if subs.macros && token.is(LessThan) && line.current_token().is_url_scheme() =>
          {
            let scheme = line.consume_current().unwrap();
            text.commit_inlines(&mut inlines);
            inlines.push(Macro(Macro::Link(
              scheme.to_url_scheme().unwrap(),
              line.consume_url(Some(&scheme), self.allocator),
              AttrList::role("bare", self.allocator),
            )));
            line.discard(1); // `>`
          }

          Some(token) if subs.macros && token.is_url_scheme() => {
            text.commit_inlines(&mut inlines);
            inlines.push(Macro(Macro::Link(
              token.to_url_scheme().unwrap(),
              line.consume_url(Some(&token), self.allocator),
              AttrList::role("bare", self.allocator),
            )));
          }

          Some(token)
            if subs.inline_formatting
              && token.is(OpenBracket)
              && line.contains_seq(&[CloseBracket, Hash]) =>
          {
            text.commit_inlines(&mut inlines);
            let attr_list = self.parse_formatted_text_attr_list(&mut line)?;
            debug_assert!(line.current_is(Hash));
            line.discard(1); // `#`
            let wrap = |inlines| TextSpan(attr_list, inlines);
            if starts_unconstrained(Hash, line.current_token().unwrap(), &line, block) {
              self.parse_unconstrained(Hash, wrap, &mut text, &mut inlines, line, block)?;
            } else {
              self.parse_constrained(Hash, wrap, &mut text, &mut inlines, line, block)?;
            };
            break;
          }

          Some(token)
            if subs.inline_formatting && token.is(Caret) && line.is_continuous_thru(Caret) =>
          {
            text.commit_inlines(&mut inlines);
            block.restore(line);
            inlines.push(Superscript(self.parse_inlines_until(block, &[Caret])?));
            break;
          }

          Some(token)
            if subs.inline_formatting && token.is(Tilde) && line.is_continuous_thru(Tilde) =>
          {
            text.commit_inlines(&mut inlines);
            block.restore(line);
            inlines.push(Subscript(self.parse_inlines_until(block, &[Tilde])?));
            break;
          }

          Some(token)
            if subs.inline_formatting
              && token.is(DoubleQuote)
              && line.current_is(Backtick)
              && starts_constrained(&[Backtick, DoubleQuote], &token, &line, block) =>
          {
            line.discard(1); // backtick
            text.push_str("“");
            text.commit_inlines(&mut inlines);
            block.restore(line);
            let mut quoted = self.parse_inlines_until(block, &[Backtick, DoubleQuote])?;
            self.merge_inlines(&mut inlines, &mut quoted, Some("”"));
            break;
          }

          Some(token)
            if subs.inline_formatting
              && token.is(SingleQuote)
              && line.current_is(Backtick)
              && starts_constrained(&[Backtick, SingleQuote], &token, &line, block) =>
          {
            line.discard(1); // backtick
            text.push_str("‘");
            text.commit_inlines(&mut inlines);
            block.restore(line);
            let mut quoted = self.parse_inlines_until(block, &[Backtick, SingleQuote])?;
            self.merge_inlines(&mut inlines, &mut quoted, Some("’"));
            break;
          }

          Some(token)
            if subs.inline_formatting
              && token.is(Backtick)
              && line.current_is(Plus)
              && contains_seq(&[Plus, Backtick], &line, block) =>
          {
            line.discard(1); // `+`
            text.commit_inlines(&mut inlines);
            block.restore(line);
            self.ctx.subs.inline_formatting = false;
            let mut inner = self.parse_inlines_until(block, &[Plus, Backtick])?;
            self.ctx.subs = subs;
            assert!(inner.len() == 1, "invalid lit mono");
            match inner.pop().unwrap() {
              Text(lit) => inlines.push(LitMono(lit)),
              _ => panic!("invalid lit mono"),
            }
            break;
          }

          Some(token)
            if token.is(Plus)
              && line.starts_with_seq(&[Plus, Plus])
              && contains_seq(&[Plus, Plus, Plus], &line, block) =>
          {
            line.discard(2); // `++`
            text.commit_inlines(&mut inlines);
            block.restore(line);
            self.ctx.subs = Substitutions::none();
            let mut passthrough = self.parse_inlines_until(block, &[Plus, Plus, Plus])?;
            self.ctx.subs = subs;
            self.merge_inlines(&mut inlines, &mut passthrough, None);
            break;
          }

          Some(token)
            if subs.inline_formatting
              && token.is(Plus)
              && line.current_is(Plus)
              && starts_unconstrained(Plus, &token, &line, block) =>
          {
            line.discard(1); // `+`
            text.commit_inlines(&mut inlines);
            block.restore(line);
            self.ctx.subs.inline_formatting = false;
            let mut passthrough = self.parse_inlines_until(block, &[Plus, Plus])?;
            self.ctx.subs = subs;
            self.merge_inlines(&mut inlines, &mut passthrough, None);
            break;
          }

          Some(token)
            if subs.inline_formatting
              && token.is(Plus)
              && starts_constrained(&[Plus], &token, &line, block) =>
          {
            text.commit_inlines(&mut inlines);
            block.restore(line);
            self.ctx.subs.inline_formatting = false;
            let mut passthrough = self.parse_inlines_until(block, &[Plus])?;
            self.ctx.subs = subs;
            self.merge_inlines(&mut inlines, &mut passthrough, None);
            break;
          }

          Some(token)
            if subs.inline_formatting && token.is(Hash) && contains_seq(&[Hash], &line, block) =>
          {
            text.commit_inlines(&mut inlines);
            block.restore(line);
            inlines.push(Highlight(self.parse_inlines_until(block, &[Hash])?));
            break;
          }

          Some(token)
            if subs.inline_formatting && starts_unconstrained(Underscore, &token, &line, block) =>
          {
            self.parse_unconstrained(Underscore, Italic, &mut text, &mut inlines, line, block)?;
            break;
          }

          Some(token)
            if subs.inline_formatting
              && starts_constrained(&[Underscore], &token, &line, block) =>
          {
            self.parse_constrained(Underscore, Italic, &mut text, &mut inlines, line, block)?;
            break;
          }

          Some(token)
            if subs.inline_formatting && starts_unconstrained(Star, &token, &line, block) =>
          {
            self.parse_unconstrained(Star, Bold, &mut text, &mut inlines, line, block)?;
            break;
          }

          Some(token)
            if subs.inline_formatting && starts_constrained(&[Star], &token, &line, block) =>
          {
            self.parse_constrained(Star, Bold, &mut text, &mut inlines, line, block)?;
            break;
          }

          Some(token)
            if subs.inline_formatting && starts_unconstrained(Backtick, &token, &line, block) =>
          {
            self.parse_unconstrained(Backtick, Mono, &mut text, &mut inlines, line, block)?;
            break;
          }

          Some(token)
            if subs.inline_formatting && starts_constrained(&[Backtick], &token, &line, block) =>
          {
            self.parse_constrained(Backtick, Mono, &mut text, &mut inlines, line, block)?;
            break;
          }

          Some(token) if subs.special_chars && token.is(Ampersand) => {
            text.push_str("&amp;");
          }

          Some(token) if subs.special_chars && token.is(LessThan) => {
            text.push_str("&lt;");
          }

          Some(token) if subs.special_chars && token.is(GreaterThan) => {
            text.push_str("&gt;");
          }

          Some(token) => text.push_token(&token),

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
    kind: TokenKind,
    wrap: impl FnOnce(Vec<'alloc, Inline<'alloc>>) -> Inline<'alloc>,
    text: &mut Text<'alloc>,
    inlines: &mut Vec<'alloc, Inline<'alloc>>,
    mut line: Line<'alloc, 'src>,
    block: &mut Block<'alloc, 'src>,
  ) -> Result<()> {
    line.discard(1); // second token
    text.commit_inlines(inlines);
    block.restore(line);
    inlines.push(wrap(self.parse_inlines_until(block, &[kind, kind])?));
    Ok(())
  }

  fn parse_constrained(
    &mut self,
    kind: TokenKind,
    wrap: impl FnOnce(Vec<'alloc, Inline<'alloc>>) -> Inline<'alloc>,
    text: &mut Text<'alloc>,
    inlines: &mut Vec<'alloc, Inline<'alloc>>,
    line: Line<'alloc, 'src>,
    block: &mut Block<'alloc, 'src>,
  ) -> Result<()> {
    text.commit_inlines(inlines);
    block.restore(line);
    inlines.push(wrap(self.parse_inlines_until(block, &[kind])?));
    Ok(())
  }

  fn merge_inlines(
    &self,
    a: &mut Vec<'alloc, Inline<'alloc>>,
    b: &mut Vec<'alloc, Inline<'alloc>>,
    append: Option<&str>,
  ) {
    if let (Some(Text(a_text)), Some(Text(b_text))) = (a.last_mut(), b.first_mut()) {
      a_text.push_str(b_text);
      b.remove(0);
    }
    a.append(b);
    match (append, a.last_mut()) {
      (Some(append), Some(Text(text))) => text.push_str(append),
      (Some(append), _) => a.push(Text(String::from_str_in(append, self.allocator))),
      _ => {}
    }
  }
}

fn starts_constrained(
  stop_tokens: &[TokenKind],
  token: &Token,
  line: &Line,
  block: &mut Block,
) -> bool {
  debug_assert!(!stop_tokens.is_empty());
  token.is(*stop_tokens.last().expect("non-empty stop tokens"))
    && (line.terminates_constrained(stop_tokens) || block.terminates_constrained(stop_tokens))
}

fn starts_unconstrained(kind: TokenKind, token: &Token, line: &Line, block: &Block) -> bool {
  token.is(kind) && line.current_is(kind) && contains_seq(&[kind; 2], line, block)
}

fn contains_seq(seq: &[TokenKind], line: &Line, block: &Block) -> bool {
  line.contains_seq(seq) || block.contains_seq(seq)
}

impl<'alloc> Text<'alloc> {
  fn commit_inlines(&mut self, inlines: &mut Vec<'alloc, Inline<'alloc>>) {
    match (self.is_empty(), inlines.last_mut()) {
      (false, Some(Inline::Text(text))) => text.push_str(&self.take()),
      (false, _) => inlines.push(Inline::Text(self.take())),
      _ => {}
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::ast::{AttrList, Inline::*, Macro, UrlScheme};
  use bumpalo::collections::String;
  use bumpalo::vec as bvec;
  use bumpalo::Bump;

  // todo...repeated
  macro_rules! s {
    (in $bump:expr; $s:expr) => {
      String::from_str_in($s, $bump)
    };
  }

  macro_rules! t {
    (in $bump:expr; $s:expr) => {
      Text(String::from_str_in($s, $bump))
    };
  }

  #[test]
  fn test_parse_inlines() {
    let b = &Bump::new();
    let bare_example_com = Macro(Macro::Link(
      UrlScheme::Https,
      s!(in b; "https://example.com"),
      AttrList::role("bare", b),
    ));
    let cases = vec![
      (
        "foo [.nowrap]#bar#",
        bvec![in b;
          t!(in b; "foo "),
          TextSpan(AttrList::role("nowrap", b), bvec![in b; t!(in b; "bar")]),
        ],
      ),
      (
        "[.big]##O##nce upon an infinite loop",
        bvec![in b;
          TextSpan(AttrList::role("big", b), bvec![in b; t!(in b;"O")]),
          t!(in b; "nce upon an infinite loop"),
        ],
      ),
      (
        "Do werewolves believe in [.small]#small print#?",
        bvec![in b;
          t!(in b; "Do werewolves believe in "),
          TextSpan(AttrList::role("small", b), bvec![in b; t!(in b; "small print")]),
          t!(in b; "?"),
        ],
      ),
      (
        "`*_foo_*`",
        bvec![in b; Mono(bvec![in b; Bold(bvec![in b; Italic(bvec![in b; t!(in b; "foo")])])])],
      ),
      (
        "foo _bar_",
        bvec![in b; t!(in b; "foo "), Italic(bvec![in b; t!(in b; "bar")])],
      ),
      (
        "foo _bar baz_",
        bvec![in b; t!(in b; "foo "), Italic(bvec![in b; t!(in b; "bar baz")])],
      ),
      (
        "foo _bar\nbaz_",
        bvec![in b; t!(in b; "foo "), Italic(bvec![in b; t!(in b; "bar baz")])],
      ),
      ("foo 'bar'", bvec![in b; t!(in b; "foo 'bar'")]),
      ("foo \"bar\"", bvec![in b; t!(in b; "foo \"bar\"")]),
      ("foo \"`bar`\"", bvec![in b; t!(in b; "foo “bar”")]),
      ("foo \"`bar baz`\"", bvec![in b; t!(in b; "foo “bar baz”")]),
      ("foo \"`bar\nbaz`\"", bvec![in b; t!(in b; "foo “bar baz”")]),
      ("foo '`bar`'", bvec![in b; t!(in b; "foo ‘bar’")]),
      ("foo '`bar baz`'", bvec![in b; t!(in b; "foo ‘bar baz’")]),
      ("foo '`bar\nbaz`'", bvec![in b; t!(in b; "foo ‘bar baz’")]),
      (
        "foo *bar*",
        bvec![in b; t!(in b; "foo "), Bold(bvec![in b; t!(in b; "bar")])],
      ),
      (
        "foo `bar`",
        bvec![in b; t!(in b; "foo "), Mono(bvec![in b; t!(in b; "bar")])],
      ),
      (
        "foo __ba__r",
        bvec![in b; t!(in b; "foo "), Italic(bvec![in b; t!(in b; "ba")]), t!(in b; "r")],
      ),
      (
        "foo **ba**r",
        bvec![in b; t!(in b; "foo "), Bold(bvec![in b; t!(in b; "ba")]), t!(in b; "r")],
      ),
      (
        "foo ``ba``r",
        bvec![in b; t!(in b; "foo "), Mono(bvec![in b; t!(in b; "ba")]), t!(in b; "r")],
      ),
      ("foo __bar", bvec![in b; t!(in b; "foo __bar")]),
      (
        "foo ^bar^",
        bvec![in b; t!(in b; "foo "), Superscript(bvec![in b; t!(in b; "bar")])],
      ),
      (
        "foo #bar#",
        bvec![in b; t!(in b; "foo "), Highlight(bvec![in b; t!(in b; "bar")])],
      ),
      ("foo ^bar", bvec![in b; t!(in b; "foo ^bar")]),
      ("foo bar^", bvec![in b; t!(in b; "foo bar^")]),
      (
        "foo ~bar~ baz",
        bvec![in b; t!(in b; "foo "), Subscript(bvec![in b; t!(in b; "bar")]), t!(in b; " baz")],
      ),
      ("foo   bar\n", bvec![in b; t!(in b; "foo bar")]),
      ("foo bar", bvec![in b; t!(in b; "foo bar")]),
      ("foo   bar\nbaz", bvec![in b; t!(in b; "foo bar baz")]),
      ("`+{name}+`", bvec![in b; LitMono(s!(in b; "{name}"))]),
      ("`+_foo_+`", bvec![in b; LitMono(s!(in b; "_foo_"))]),
      (
        "foo <bar> & lol",
        bvec![in b; Text(s!(in b; "foo &lt;bar&gt; &amp; lol"))],
      ),
      ("+_foo_+", bvec![in b; Text(s!(in b; "_foo_"))]),
      (
        "+_<foo>&_+",
        bvec![in b; Text(s!(in b; "_&lt;foo&gt;&amp;_"))],
      ),
      (
        "rofl +_foo_+ lol",
        bvec![in b; Text(s!(in b; "rofl _foo_ lol"))],
      ),
      ("++_foo_++bar", bvec![in b; Text(s!(in b; "_foo_bar"))]),
      (
        "lol ++_foo_++bar",
        bvec![in b; Text(s!(in b; "lol _foo_bar"))],
      ),
      ("+++_<foo>&_+++", bvec![in b; Text(s!(in b; "_<foo>&_"))]),
      (
        "foo image:sunset.jpg[]",
        bvec![in b;
          Text(s!(in b; "foo ")),
          Macro(Macro::Image(s!(in b; "sunset.jpg"), AttrList::new_in(b))),
        ],
      ),
      (
        "doublefootnote:[ymmv]bar",
        bvec![in b;
          Text(s!(in b; "double")),
          Macro(Macro::Footnote(None, AttrList::positional("ymmv", b))),
          Text(s!(in b; "bar")),
        ],
      ),
      (
        "kbd:[F11]",
        bvec![in b; Macro(Macro::Keyboard(AttrList::positional("F11", b)))],
      ),
      (
        "foo https://example.com",
        bvec![in b; Text(s!(in b; "foo ")), bare_example_com.clone()],
      ),
      (
        "foo https://example.com.",
        bvec![in b; Text(s!(in b; "foo ")), bare_example_com.clone(), Text(s!(in b; "."))],
      ),
      (
        "foo https://example.com bar",
        bvec![in b; Text(s!(in b; "foo ")), bare_example_com.clone(), Text(s!(in b; " bar"))],
      ),
      (
        "foo <https://example.com> bar",
        bvec![in b; Text(s!(in b; "foo ")), bare_example_com.clone(), Text(s!(in b; " bar"))],
      ),
    ];

    // repeated passes necessary?
    // yikes: `link:pass:[My Documents/report.pdf][Get Report]`

    for (input, expected) in cases {
      let mut parser = crate::Parser::new(b, input);
      let block = parser.read_block().unwrap();
      let inlines = parser.parse_inlines(block).unwrap();
      assert_eq!(inlines, expected);
    }
  }

  impl<'alloc> AttrList<'alloc> {
    pub fn positional(role: &'static str, allocator: &'alloc Bump) -> AttrList<'alloc> {
      AttrList {
        positional: bvec![in allocator; String::from_str_in(role, allocator)],
        ..AttrList::new_in(allocator)
      }
    }
  }
}
