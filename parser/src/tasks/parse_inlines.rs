use regex::Regex;

use crate::ast::*;
use crate::block::Block;
use crate::line::Line;
use crate::parser::Substitutions;
use crate::tasks::parse_inlines_utils::*;
use crate::tasks::text_span::TextSpan;
use crate::token::{Token, TokenIs, TokenKind, TokenKind::*};
use crate::utils::bump::*;
use crate::{Parser, Result};

impl<'bmp, 'src> Parser<'bmp, 'src> {
  pub(super) fn parse_inlines(
    &mut self,
    block: &mut Block<'bmp, 'src>,
  ) -> Result<Vec<'bmp, InlineNode<'bmp>>> {
    self.parse_inlines_until(block, &[])
  }

  fn parse_inlines_until(
    &mut self,
    block: &mut Block<'bmp, 'src>,
    stop_tokens: &[TokenKind],
  ) -> Result<Vec<'bmp, InlineNode<'bmp>>> {
    let mut inlines = Vec::new_in(self.bump);
    if block.is_empty() {
      return Ok(inlines);
    }
    let span_loc = block.location().unwrap().clamp_start();
    let mut text = TextSpan::new_in(span_loc, self.bump);
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

        if self.ctx.delimiter.is_some() && line.current_is(DelimiterLine) {
          if matches!(inlines.last().map(|n| &n.content), Some(JoiningNewline)) {
            inlines.pop();
          }
          block.restore(line);
          return Ok(inlines);
        }

        let Some(token) = line.consume_current() else {
          if !block.is_empty() {
            text.commit_inlines(&mut inlines);
            text.loc.end += 1;
            inlines.push(node(JoiningNewline, text.loc));
            text.loc = text.loc.clamp_end();
          }
          break;
        };

        match token.kind {
          MacroName if subs.macros && line.continues_inline_macro() => {
            let mut macro_loc = token.loc;
            let line_end = line.last_location().unwrap();
            text.commit_inlines(&mut inlines);
            match token.lexeme {
              "image:" => {
                let target = line.consume_macro_target(self.bump);
                let attrs = self.parse_attr_list(&mut line)?;
                finish_macro(&line, &mut macro_loc, line_end, &mut text);
                inlines.push(node(
                  Macro(Image { flow: Flow::Inline, target, attrs }),
                  macro_loc,
                ));
              }
              "kbd:" => {
                line.discard_assert(OpenBracket);
                let keys_src = line.consume_to_string_until(CloseBracket, self.bump);
                line.discard_assert(CloseBracket);
                macro_loc.end = keys_src.loc.end + 1;
                let mut keys = Vec::new_in(self.bump);
                let re = Regex::new(r"(?:\s*([^\s,+]+|[,+])\s*)").unwrap();
                for captures in re.captures_iter(&keys_src).step_by(2) {
                  let key = captures.get(1).unwrap().as_str();
                  keys.push(String::from_str_in(key, self.bump));
                }
                inlines.push(node(Macro(Keyboard { keys, keys_src }), macro_loc));
                text.loc = macro_loc.clamp_end();
              }
              "footnote:" => {
                let id = line.consume_optional_macro_target(self.bump);
                block.restore(line);
                let note = self.parse_inlines_until(block, &[CloseBracket])?;
                extend(&mut macro_loc, &note, 1);
                inlines.push(node(Macro(Footnote { id, text: note }), macro_loc));
                text.loc = macro_loc.clamp_end();
                break;
              }
              "mailto:" | "link:" => {
                let target = line.consume_macro_target(self.bump);
                let attrs = self.parse_attr_list(&mut line)?;
                finish_macro(&line, &mut macro_loc, line_end, &mut text);
                let scheme = token.to_url_scheme();
                inlines.push(node(
                  Macro(Macro::Link { scheme, target, attrs: Some(attrs) }),
                  macro_loc,
                ));
              }
              "https:" | "http:" | "irc:" => {
                let target = line.consume_url(Some(&token), self.bump);
                line.discard_assert(OpenBracket);
                let attrs = self.parse_attr_list(&mut line)?;
                finish_macro(&line, &mut macro_loc, line_end, &mut text);
                let scheme = Some(token.to_url_scheme().unwrap());
                inlines.push(node(
                  Macro(Macro::Link { scheme, target, attrs: Some(attrs) }),
                  macro_loc,
                ));
              }
              "pass:" => {
                let target = line.consume_optional_macro_target(self.bump);
                self.ctx.subs = Substitutions::from_pass_macro_target(&target);
                let mut attrs = self.parse_attr_list(&mut line)?;
                self.ctx.subs = subs;
                finish_macro(&line, &mut macro_loc, line_end, &mut text);
                let content = if !attrs.positional.is_empty() && attrs.positional[0].is_some() {
                  attrs.positional[0].take().unwrap()
                } else {
                  bvec![in self.bump] // ...should probably be a diagnostic
                };
                inlines.push(node(Macro(Pass { target, content }), macro_loc));
              }
              "icon:" => {
                let target = line.consume_macro_target(self.bump);
                let attrs = self.parse_attr_list(&mut line)?;
                finish_macro(&line, &mut macro_loc, line_end, &mut text);
                inlines.push(node(Macro(Icon { target, attrs }), macro_loc));
              }
              "btn:" => {
                line.discard_assert(OpenBracket);
                let btn = line.consume_to_string_until(CloseBracket, self.bump);
                line.discard_assert(CloseBracket);
                finish_macro(&line, &mut macro_loc, line_end, &mut text);
                inlines.push(node(Macro(Button(btn)), macro_loc));
              }
              "menu:" => {
                let first = line.consume_macro_target(self.bump);
                let mut items = bvec![in self.bump; first];
                let rest = line.consume_to_string_until(CloseBracket, self.bump);

                let mut pos = rest.loc.start;
                rest.split('>').for_each(|substr| {
                  let mut trimmed = substr.trim_start();
                  pos += substr.len() - trimmed.len();
                  trimmed = trimmed.trim_end();
                  if !trimmed.is_empty() {
                    items.push(SourceString::new(
                      String::from_str_in(trimmed, self.bump),
                      SourceLocation::new(pos, pos + trimmed.len()),
                    ));
                  }
                  pos += substr.len() + 1;
                });
                line.discard_assert(CloseBracket);
                finish_macro(&line, &mut macro_loc, line_end, &mut text);
                inlines.push(node(Macro(Menu(items)), macro_loc));
              }
              _ => todo!("unhandled macro type: `{}`", token.lexeme),
            }
          }

          LessThan
            if subs.macros
              && line.current_token().is_url_scheme()
              && line.is_continuous_thru(GreaterThan) =>
          {
            text.commit_inlines(&mut inlines);
            inlines.push(node(Discarded, token.loc));
            let scheme_token = line.consume_current().unwrap();
            let mut loc = scheme_token.loc;
            let line_end = line.last_location().unwrap();
            let target = line.consume_url(Some(&scheme_token), self.bump);
            finish_macro(&line, &mut loc, line_end, &mut text);
            let scheme = Some(scheme_token.to_url_scheme().unwrap());
            inlines.push(node(
              Macro(Macro::Link { scheme, target, attrs: None }),
              loc,
            ));
            inlines.push(node(Discarded, line.consume_current().unwrap().loc));
            text.loc = loc.incr_end().clamp_end();
          }

          MaybeEmail if subs.macros && EMAIL_RE.is_match(token.lexeme) => {
            text.commit_inlines(&mut inlines);
            inlines.push(node(
              Macro(Macro::Link {
                scheme: Some(UrlScheme::Mailto),
                target: SourceString::new(String::from_str_in(token.lexeme, self.bump), token.loc),
                attrs: None,
              }),
              token.loc,
            ));
            text.loc = token.loc.clamp_end();
          }

          Underscore
            if subs.inline_formatting
              && starts_constrained(&[Underscore], &token, &line, block) =>
          {
            self.parse_constrained(&token, Italic, &mut text, &mut inlines, line, block)?;
            break;
          }

          Underscore
            if subs.inline_formatting && starts_unconstrained(Underscore, &token, &line, block) =>
          {
            self.parse_unconstrained(&token, Italic, &mut text, &mut inlines, line, block)?;
            break;
          }

          Star if subs.inline_formatting && starts_constrained(&[Star], &token, &line, block) => {
            self.parse_constrained(&token, Bold, &mut text, &mut inlines, line, block)?;
            break;
          }

          Star if subs.inline_formatting && starts_unconstrained(Star, &token, &line, block) => {
            self.parse_unconstrained(&token, Bold, &mut text, &mut inlines, line, block)?;
            break;
          }

          OpenBracket if subs.inline_formatting && line.contains_seq(&[CloseBracket, Hash]) => {
            let mut parse_token = token.clone();
            let attr_list = self.parse_formatted_text_attr_list(&mut line)?;
            debug_assert!(line.current_is(Hash));
            line.discard_assert(Hash);
            parse_token.kind = Hash;
            let wrap = |inner| TextSpan(attr_list, inner);
            if starts_unconstrained(Hash, line.current_token().unwrap(), &line, block) {
              self.parse_unconstrained(&parse_token, wrap, &mut text, &mut inlines, line, block)?;
            } else {
              self.parse_constrained(&parse_token, wrap, &mut text, &mut inlines, line, block)?;
            };
            break;
          }

          Backtick
            if subs.inline_formatting
              && line.current_is(Plus)
              && contains_seq(&[Plus, Backtick], &line, block) =>
          {
            let mut wrap_loc = token.loc;
            line.discard_assert(Plus);
            text.commit_inlines(&mut inlines);
            block.restore(line);
            self.ctx.subs.inline_formatting = false;
            let mut inner = self.parse_inlines_until(block, &[Plus, Backtick])?;
            extend(&mut wrap_loc, &inner, 2);
            self.ctx.subs = subs;
            assert!(inner.len() == 1, "invalid lit mono");
            match inner.pop().unwrap() {
              InlineNode { content: Text(lit), loc } => {
                inlines.push(node(LitMono(SourceString::new(lit, loc)), wrap_loc))
              }
              _ => panic!("invalid lit mono"),
            }
            break;
          }

          Caret if subs.inline_formatting && line.is_continuous_thru(Caret) => {
            let mut loc = token.loc;
            text.commit_inlines(&mut inlines);
            block.restore(line);
            let inner = self.parse_inlines_until(block, &[Caret])?;
            extend(&mut loc, &inner, 1);
            inlines.push(node(Superscript(inner), loc));
            text.loc = loc.clamp_end();
            break;
          }

          DoubleQuote
            if subs.inline_formatting
              && line.current_is(Backtick)
              && starts_constrained(&[Backtick, DoubleQuote], &token, &line, block) =>
          {
            let mut loc = token.loc;
            line.discard_assert(Backtick);
            text.commit_inlines(&mut inlines);
            block.restore(line);
            let quoted = self.parse_inlines_until(block, &[Backtick, DoubleQuote])?;
            extend(&mut loc, &quoted, 2);
            inlines.push(node(Quote(Double, quoted), loc));
            text.loc = loc.clamp_end();
            break;
          }

          SingleQuote
            if subs.inline_formatting
              && line.current_is(Backtick)
              && starts_constrained(&[Backtick, SingleQuote], &token, &line, block) =>
          {
            let mut loc = token.loc;
            line.discard_assert(Backtick);
            text.commit_inlines(&mut inlines);
            block.restore(line);
            let quoted = self.parse_inlines_until(block, &[Backtick, SingleQuote])?;
            extend(&mut loc, &quoted, 2);
            inlines.push(node(Quote(Single, quoted), loc));
            text.loc = loc.clamp_end();
            break;
          }

          Tilde if subs.inline_formatting && line.is_continuous_thru(Tilde) => {
            let mut loc = token.loc;
            text.commit_inlines(&mut inlines);
            block.restore(line);
            let inner = self.parse_inlines_until(block, &[Tilde])?;
            extend(&mut loc, &inner, 1);
            inlines.push(node(Subscript(inner), loc));
            text.loc = loc.clamp_end();
            break;
          }

          Backtick if subs.inline_formatting && line.current_is(DoubleQuote) => {
            let mut loc = token.loc;
            line.discard_assert(DoubleQuote);
            loc.end += 1;
            text.commit_inlines(&mut inlines);
            block.restore(line);
            inlines.push(node(Curly(RightDouble), loc));
            text.loc = loc.clamp_end();
            break;
          }

          DoubleQuote if subs.inline_formatting && line.current_is(Backtick) => {
            let mut loc = token.loc;
            line.discard_assert(Backtick);
            loc.end += 1;
            text.commit_inlines(&mut inlines);
            block.restore(line);
            inlines.push(node(Curly(LeftDouble), loc));
            text.loc = loc.clamp_end();
            break;
          }

          Backtick if subs.inline_formatting && line.current_is(SingleQuote) => {
            let mut loc = token.loc;
            line.discard_assert(SingleQuote);
            loc.end += 1;
            text.commit_inlines(&mut inlines);
            block.restore(line);
            inlines.push(node(Curly(RightSingle), loc));
            text.loc = loc.clamp_end();
            break;
          }

          SingleQuote if subs.inline_formatting && line.current_is(Backtick) => {
            let mut loc = token.loc;
            line.discard_assert(Backtick);
            loc.end += 1;
            text.commit_inlines(&mut inlines);
            block.restore(line);
            inlines.push(node(Curly(LeftSingle), loc));
            text.loc = loc.clamp_end();
            break;
          }

          Backtick
            if subs.inline_formatting && starts_constrained(&[Backtick], &token, &line, block) =>
          {
            self.parse_constrained(&token, Mono, &mut text, &mut inlines, line, block)?;
            break;
          }

          Backtick
            if subs.inline_formatting && starts_unconstrained(Backtick, &token, &line, block) =>
          {
            self.parse_unconstrained(&token, Mono, &mut text, &mut inlines, line, block)?;
            break;
          }

          Hash if subs.inline_formatting && contains_seq(&[Hash], &line, block) => {
            self.parse_constrained(&token, Highlight, &mut text, &mut inlines, line, block)?;
            break;
          }

          Plus
            if line.starts_with_seq(&[Plus, Plus])
              && contains_seq(&[Plus, Plus, Plus], &line, block) =>
          {
            let mut loc = token.loc;
            line.discard_assert(Plus);
            line.discard_assert(Plus);
            text.commit_inlines(&mut inlines);
            block.restore(line);
            self.ctx.subs = Substitutions::none();
            let passthrough = self.parse_inlines_until(block, &[Plus, Plus, Plus])?;
            extend(&mut loc, &passthrough, 3);
            self.ctx.subs = subs;
            inlines.push(node(InlinePassthrough(passthrough), loc));
            text.loc = loc.clamp_end();
            break;
          }

          Plus
            if subs.inline_formatting
              && line.current_is(Plus)
              && starts_unconstrained(Plus, &token, &line, block) =>
          {
            self.ctx.subs.inline_formatting = false;
            self.parse_unconstrained(
              &token,
              InlinePassthrough,
              &mut text,
              &mut inlines,
              line,
              block,
            )?;
            self.ctx.subs = subs;
            break;
          }

          Plus if subs.inline_formatting && starts_constrained(&[Plus], &token, &line, block) => {
            self.ctx.subs.inline_formatting = false;
            self.parse_constrained(
              &token,
              InlinePassthrough,
              &mut text,
              &mut inlines,
              line,
              block,
            )?;
            self.ctx.subs = subs;
            break;
          }

          Ampersand | LessThan | GreaterThan if subs.special_chars => {
            text.commit_inlines(&mut inlines);
            inlines.push(node(
              SpecialChar(match token.kind {
                Ampersand => SpecialCharKind::Ampersand,
                LessThan => SpecialCharKind::LessThan,
                GreaterThan => SpecialCharKind::GreaterThan,
                _ => unreachable!(),
              }),
              token.loc,
            ));
            text.loc = token.loc.clamp_end();
          }

          SingleQuote if line.current_is(Word) && subs.inline_formatting => {
            if text.is_empty() || text.ends_with(char::is_whitespace) {
              text.push_token(&token);
            } else {
              text.commit_inlines(&mut inlines);
              inlines.push(node(Curly(LegacyImplicitApostrophe), token.loc));
              text.loc = token.loc.clamp_end();
            }
          }

          Whitespace if token.lexeme.len() > 1 && subs.inline_formatting => {
            text.commit_inlines(&mut inlines);
            inlines.push(node(
              MultiCharWhitespace(String::from_str_in(token.lexeme, self.bump)),
              token.loc,
            ));
            text.loc = token.loc.clamp_end();
          }

          Backslash
            if subs.macros
              && (line.current_is(MaybeEmail) || line.current_token().is_url_scheme()) =>
          {
            text.commit_inlines(&mut inlines);
            inlines.push(node(Discarded, token.loc));
            text.loc = token.loc.clamp_end();
            // pushing the next token as text prevents macro subs for escaped token
            let next_token = line.consume_current().unwrap();
            text.push_token(&next_token);
          }

          _ if subs.macros && token.is_url_scheme() && line.src.starts_with("//") => {
            let mut loc = token.loc;
            let line_end = line.last_location().unwrap();
            text.commit_inlines(&mut inlines);
            let target = line.consume_url(Some(&token), self.bump);
            finish_macro(&line, &mut loc, line_end, &mut text);
            let scheme = Some(token.to_url_scheme().unwrap());
            inlines.push(node(
              Macro(Macro::Link { scheme, target, attrs: None }),
              loc,
            ));
            text.loc = loc.clamp_end();
          }

          _ => {
            text.push_token(&token);
          }
        }
      }
    }

    text.commit_inlines(&mut inlines);

    Ok(inlines)
  }

  fn parse_unconstrained(
    &mut self,
    token: &Token<'src>,
    wrap: impl FnOnce(Vec<'bmp, InlineNode<'bmp>>) -> Inline<'bmp>,
    text: &mut TextSpan<'bmp>,
    inlines: &mut Vec<'bmp, InlineNode<'bmp>>,
    mut line: Line<'bmp, 'src>,
    block: &mut Block<'bmp, 'src>,
  ) -> Result<()> {
    let mut loc = token.loc;
    line.discard_assert(token.kind);
    text.commit_inlines(inlines);
    block.restore(line);
    let inner = self.parse_inlines_until(block, &[token.kind, token.kind])?;
    extend(&mut loc, &inner, 2);
    inlines.push(node(wrap(inner), loc));
    text.loc = loc.clamp_end();
    Ok(())
  }

  fn parse_constrained(
    &mut self,
    token: &Token<'src>,
    wrap: impl FnOnce(Vec<'bmp, InlineNode<'bmp>>) -> Inline<'bmp>,
    text: &mut TextSpan<'bmp>,
    inlines: &mut Vec<'bmp, InlineNode<'bmp>>,
    line: Line<'bmp, 'src>,
    block: &mut Block<'bmp, 'src>,
  ) -> Result<()> {
    let mut loc = token.loc;
    text.commit_inlines(inlines);
    block.restore(line);
    let inner = self.parse_inlines_until(block, &[token.kind])?;
    extend(&mut loc, &inner, 1);
    inlines.push(node(wrap(inner), loc));
    text.loc = loc.clamp_end();
    Ok(())
  }

  fn merge_inlines(
    &self,
    a: &mut Vec<'bmp, Inline<'bmp>>,
    b: &mut Vec<'bmp, Inline<'bmp>>,
    append: Option<&str>,
  ) {
    if let (Some(Text(a_text)), Some(Text(b_text))) = (a.last_mut(), b.first_mut()) {
      a_text.push_str(b_text);
      b.remove(0);
    }
    a.append(b);
    match (append, a.last_mut()) {
      (Some(append), Some(Text(text))) => text.push_str(append),
      (Some(append), _) => a.push(Text(String::from_str_in(append, self.bump))),
      _ => {}
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::ast::*;
  use crate::test::*;
  use crate::utils::bump::*;

  #[test]
  fn test_parse_inlines() {
    let b = &Bump::new();
    let cases = vec![
      (
        "+_foo_+",
        b.vec([n(
          InlinePassthrough(b.vec([n_text("_foo_", 1, 6, b)])),
          l(0, 7),
        )]),
      ),
      (
        "`*_foo_*`",
        b.vec([n(
          Mono(b.vec([n(
            Bold(b.vec([n(Italic(b.vec([n_text("foo", 3, 6, b)])), l(2, 7))])),
            l(1, 8),
          )])),
          l(0, 9),
        )]),
      ),
      (
        "+_foo\nbar_+",
        // not sure if this is "spec", but it's what asciidoctor currently does
        b.vec([n(
          InlinePassthrough(b.vec([
            n_text("_foo", 1, 5, b),
            n(JoiningNewline, l(5, 6)),
            n_text("bar_", 6, 10, b),
          ])),
          l(0, 11),
        )]),
      ),
      (
        "+_<foo>&_+",
        b.vec([n(
          InlinePassthrough(b.vec([
            n_text("_", 1, 2, b),
            n(SpecialChar(SpecialCharKind::LessThan), l(2, 3)),
            n_text("foo", 3, 6, b),
            n(SpecialChar(SpecialCharKind::GreaterThan), l(6, 7)),
            n(SpecialChar(SpecialCharKind::Ampersand), l(7, 8)),
            n_text("_", 8, 9, b),
          ])),
          l(0, 10),
        )]),
      ),
      (
        "rofl +_foo_+ lol",
        b.vec([
          n_text("rofl ", 0, 5, b),
          n(
            InlinePassthrough(b.vec([n_text("_foo_", 6, 11, b)])),
            l(5, 12),
          ),
          n_text(" lol", 12, 16, b),
        ]),
      ),
      (
        "++_foo_++bar",
        b.vec([
          n(
            InlinePassthrough(b.vec([n_text("_foo_", 2, 7, b)])),
            l(0, 9),
          ),
          n_text("bar", 9, 12, b),
        ]),
      ),
      (
        "+++_<foo>&_+++ bar",
        b.vec([
          n(
            InlinePassthrough(b.vec([n_text("_<foo>&_", 3, 11, b)])),
            l(0, 14),
          ),
          n_text(" bar", 14, 18, b),
        ]),
      ),
      (
        "foo #bar#",
        b.vec([
          n_text("foo ", 0, 4, b),
          n(Highlight(b.vec([n_text("bar", 5, 8, b)])), l(4, 9)),
        ]),
      ),
      (
        "image::foo.png[]", // unexpected block macro, parse as text
        b.vec([n_text("image::foo.png[]", 0, 16, b)]),
      ),
      (
        "foo `bar`",
        b.vec([
          n_text("foo ", 0, 4, b),
          n(Mono(b.vec([n_text("bar", 5, 8, b)])), l(4, 9)),
        ]),
      ),
      (
        "foo b``ar``",
        bvec![in b;
          n_text("foo b", 0, 5, b),
          n(Mono(b.vec([n_text("ar", 7, 9, b)])), l(5, 11)),
        ],
      ),
      (
        "foo *bar*",
        b.vec([
          n_text("foo ", 0, 4, b),
          n(Bold(b.vec([n_text("bar", 5, 8, b)])), l(4, 9)),
        ]),
      ),
      (
        "foo b**ar**",
        b.vec([
          n_text("foo b", 0, 5, b),
          n(Bold(b.vec([n_text("ar", 7, 9, b)])), l(5, 11)),
        ]),
      ),
      (
        "foo ~bar~ baz",
        b.vec([
          n_text("foo ", 0, 4, b),
          n(Subscript(b.vec([n_text("bar", 5, 8, b)])), l(4, 9)),
          n_text(" baz", 9, 13, b),
        ]),
      ),
      (
        "foo _bar\nbaz_",
        b.vec([
          n_text("foo ", 0, 4, b),
          n(
            Italic(b.vec([
              n_text("bar", 5, 8, b),
              n(JoiningNewline, l(8, 9)),
              n_text("baz", 9, 12, b),
            ])),
            l(4, 13),
          ),
        ]),
      ),
      ("foo __bar", b.vec([n_text("foo __bar", 0, 9, b)])),
      (
        "foo _bar baz_",
        b.vec([
          n_text("foo ", 0, 4, b),
          n(Italic(b.vec([n_text("bar baz", 5, 12, b)])), l(4, 13)),
        ]),
      ),
      (
        "foo _bar_",
        b.vec([
          n_text("foo ", 0, 4, b),
          n(Italic(b.vec([n_text("bar", 5, 8, b)])), l(4, 9)),
        ]),
      ),
      (
        "foo b__ar__",
        b.vec([
          n_text("foo b", 0, 5, b),
          n(Italic(b.vec([n_text("ar", 7, 9, b)])), l(5, 11)),
        ]),
      ),
      ("foo 'bar'", b.vec([n_text("foo 'bar'", 0, 9, b)])),
      ("foo \"bar\"", b.vec([n_text("foo \"bar\"", 0, 9, b)])),
      (
        "foo `\"bar\"`",
        b.vec([
          n_text("foo ", 0, 4, b),
          n(Curly(RightDouble), l(4, 6)),
          n_text("bar", 6, 9, b),
          n(Curly(LeftDouble), l(9, 11)),
        ]),
      ),
      (
        "foo `'bar'`",
        b.vec([
          n_text("foo ", 0, 4, b),
          n(Curly(RightSingle), l(4, 6)),
          n_text("bar", 6, 9, b),
          n(Curly(LeftSingle), l(9, 11)),
        ]),
      ),
      (
        "foo \"`bar`\"",
        b.vec([
          n_text("foo ", 0, 4, b),
          n(
            Quote(QuoteKind::Double, b.vec([n_text("bar", 6, 9, b)])),
            l(4, 11),
          ),
        ]),
      ),
      (
        "foo \"`bar baz`\"",
        b.vec([
          n_text("foo ", 0, 4, b),
          n(
            Quote(QuoteKind::Double, b.vec([n_text("bar baz", 6, 13, b)])),
            l(4, 15),
          ),
        ]),
      ),
      (
        "foo \"`bar\nbaz`\"",
        b.vec([
          n_text("foo ", 0, 4, b),
          n(
            Quote(
              QuoteKind::Double,
              b.vec([
                n_text("bar", 6, 9, b),
                n(JoiningNewline, l(9, 10)),
                n_text("baz", 10, 13, b),
              ]),
            ),
            l(4, 15),
          ),
        ]),
      ),
      (
        "foo '`bar`'",
        b.vec([
          n_text("foo ", 0, 4, b),
          n(
            Quote(QuoteKind::Single, b.vec([n_text("bar", 6, 9, b)])),
            l(4, 11),
          ),
        ]),
      ),
      (
        "Olaf's wrench",
        b.vec([
          n_text("Olaf", 0, 4, b),
          n(Curly(LegacyImplicitApostrophe), l(4, 5)),
          n_text("s wrench", 5, 13, b),
        ]),
      ),
      (
        "foo   bar",
        b.vec([
          n_text("foo", 0, 3, b),
          n(MultiCharWhitespace(b.s("   ")), l(3, 6)),
          n_text("bar", 6, 9, b),
        ]),
      ),
      (
        "`+{name}+`",
        b.vec([n(LitMono(b.src("{name}", l(2, 8))), l(0, 10))]),
      ),
      (
        "`+_foo_+`",
        b.vec([n(LitMono(b.src("_foo_", l(2, 7))), l(0, 9))]),
      ),
      (
        "foo <bar> & lol",
        b.vec([
          n_text("foo ", 0, 4, b),
          n(SpecialChar(SpecialCharKind::LessThan), l(4, 5)),
          n_text("bar", 5, 8, b),
          n(SpecialChar(SpecialCharKind::GreaterThan), l(8, 9)),
          n_text(" ", 9, 10, b),
          n(SpecialChar(SpecialCharKind::Ampersand), l(10, 11)),
          n_text(" lol", 11, 15, b),
        ]),
      ),
      (
        "^bar^",
        b.vec([n(Superscript(b.vec([n_text("bar", 1, 4, b)])), l(0, 5))]),
      ),
      (
        "^bar^",
        b.vec([n(Superscript(b.vec([n_text("bar", 1, 4, b)])), l(0, 5))]),
      ),
      ("foo ^bar", b.vec([n_text("foo ^bar", 0, 8, b)])),
      ("foo bar^", b.vec([n_text("foo bar^", 0, 8, b)])),
      (
        "foo ^bar^ foo",
        b.vec([
          n_text("foo ", 0, 4, b),
          n(Superscript(bvec![in b; n_text("bar", 5, 8, b)]), l(4, 9)),
          n_text(" foo", 9, 13, b),
        ]),
      ),
      (
        "doublefootnote:[ymmv _i_]bar",
        b.vec([
          n_text("double", 0, 6, b),
          n(
            Macro(Footnote {
              id: None,
              text: b.vec([
                n_text("ymmv ", 16, 21, b),
                n(Italic(b.vec([n_text("i", 22, 23, b)])), l(21, 24)),
              ]),
            }),
            l(6, 25),
          ),
          n_text("bar", 25, 28, b),
        ]),
      ),
    ];

    // repeated passes necessary?
    // yikes: `link:pass:[My Documents/report.pdf][Get Report]`

    run(cases, b);
  }

  #[test]
  fn test_button_menu_macro() {
    let b = &Bump::new();
    let cases = vec![
      (
        "press the btn:[OK] button",
        b.vec([
          n_text("press the ", 0, 10, b),
          n(Macro(Button(b.src("OK", l(15, 17)))), l(10, 18)),
          n_text(" button", 18, 25, b),
        ]),
      ),
      (
        "btn:[Open]",
        b.vec([n(Macro(Button(b.src("Open", l(5, 9)))), l(0, 10))]),
      ),
      (
        "select menu:File[Save].",
        b.vec([
          n_text("select ", 0, 7, b),
          n(
            Macro(Menu(
              b.vec([b.src("File", l(12, 16)), b.src("Save", l(17, 21))]),
            )),
            l(7, 22),
          ),
          n_text(".", 22, 23, b),
        ]),
      ),
      (
        "menu:View[Zoom > Reset]",
        b.vec([n(
          Macro(Menu(b.vec([
            b.src("View", l(5, 9)),
            b.src("Zoom", l(10, 14)),
            b.src("Reset", l(17, 22)),
          ]))),
          l(0, 23),
        )]),
      ),
    ];
    run(cases, b);
  }

  #[test]
  fn test_image_macro() {
    let b = &Bump::new();
    let cases = vec![
      (
        "foo image:sunset.jpg[] bar",
        b.vec([
          n_text("foo ", 0, 4, b),
          n(
            Macro(Image {
              flow: Flow::Inline,
              target: b.src("sunset.jpg", l(10, 20)),
              attrs: AttrList::new(l(20, 22), b),
            }),
            l(4, 22),
          ),
          n_text(" bar", 22, 26, b),
        ]),
      ),
      (
        "foo image:sunset.jpg[]",
        b.vec([
          n_text("foo ", 0, 4, b),
          n(
            Macro(Image {
              flow: Flow::Inline,
              target: b.src("sunset.jpg", l(10, 20)),
              attrs: AttrList::new(l(20, 22), b),
            }),
            l(4, 22),
          ),
        ]),
      ),
    ];
    run(cases, b);
  }

  #[test]
  fn test_kbd_macro() {
    let b = &Bump::new();
    let cases = vec![
      (
        "kbd:[F11]",
        b.vec([n(
          Macro(Keyboard {
            keys: bvec![in b; b.s("F11")],
            keys_src: b.src("F11", l(5, 8)),
          }),
          l(0, 9),
        )]),
      ),
      (
        "foo kbd:[F11] bar",
        b.vec([
          n_text("foo ", 0, 4, b),
          n(
            Macro(Keyboard {
              keys: bvec![in b; b.s("F11")],
              keys_src: b.src("F11", l(9, 12)),
            }),
            l(4, 13),
          ),
          n_text(" bar", 13, 17, b),
        ]),
      ),
      (
        "kbd:[Ctrl++]",
        b.vec([n(
          Macro(Keyboard {
            keys: bvec![in b; b.s("Ctrl"), b.s("+")],
            keys_src: b.src("Ctrl++", l(5, 11)),
          }),
          l(0, 12),
        )]),
      ),
      (
        "kbd:[\\ ]",
        b.vec([n(
          Macro(Keyboard {
            keys: bvec![in b; b.s("\\")],
            keys_src: b.src("\\ ", l(5, 7)),
          }),
          l(0, 8),
        )]),
      ),
      ("kbd:[\\]", b.vec([n_text("kbd:[\\]", 0, 7, b)])),
    ];

    run(cases, b);
  }

  #[test]
  fn test_custom_inline_styles() {
    let b = &Bump::new();
    let role = |role: &'static str, loc: SourceLocation| -> AttrList {
      AttrList {
        roles: b.vec([b.src(role, SourceLocation::new(loc.start + 2, loc.end - 1))]),
        ..AttrList::new(loc, b)
      }
    };
    let cases = vec![
      (
        "foo [.nowrap]#bar#",
        b.vec([
          n_text("foo ", 0, 4, b),
          n(
            TextSpan(role("nowrap", l(4, 13)), b.vec([n_text("bar", 14, 17, b)])),
            l(4, 18),
          ),
        ]),
      ),
      (
        "[.big]##O##nce upon an infinite loop",
        b.vec([
          n(
            TextSpan(role("big", l(0, 6)), b.vec([n_text("O", 8, 9, b)])),
            l(0, 11),
          ),
          n_text("nce upon an infinite loop", 11, 36, b),
        ]),
      ),
      (
        "Do werewolves believe in [.small]#small print#?",
        b.vec([
          n_text("Do werewolves believe in ", 0, 25, b),
          n(
            TextSpan(
              role("small", l(25, 33)),
              b.vec([n_text("small print", 34, 45, b)]),
            ),
            l(25, 46),
          ),
          n_text("?", 46, 47, b),
        ]),
      ),
    ];
    run(cases, b);
  }

  #[test]
  fn test_link_macro_and_autolinks() {
    let b = &Bump::new();
    let bare_example_com = |loc: SourceLocation| -> Inline {
      Macro(Link {
        scheme: Some(UrlScheme::Https),
        target: b.src("https://example.com", loc),
        attrs: None,
      })
    };
    let cases = vec![
      (
        "foo https://example.com",
        b.vec([
          n_text("foo ", 0, 4, b),
          n(bare_example_com(l(4, 23)), l(4, 23)),
        ]),
      ),
      (
        "foo https://example.com.",
        b.vec([
          n_text("foo ", 0, 4, b),
          n(bare_example_com(l(4, 23)), l(4, 23)),
          n_text(".", 23, 24, b),
        ]),
      ),
      (
        "foo \\https://example.com.", // escaped autolink
        b.vec([
          n_text("foo ", 0, 4, b),
          n(Discarded, l(4, 5)),
          n_text("https://example.com.", 5, 25, b),
        ]),
      ),
      (
        "foo https://example.com bar",
        b.vec([
          n_text("foo ", 0, 4, b),
          n(bare_example_com(l(4, 23)), l(4, 23)),
          n_text(" bar", 23, 27, b),
        ]),
      ),
      (
        "foo http://example.com bar",
        b.vec([
          n_text("foo ", 0, 4, b),
          n(
            Macro(Link {
              scheme: Some(UrlScheme::Http),
              target: b.src("http://example.com", l(4, 22)),
              attrs: None,
            }),
            l(4, 22),
          ),
          n_text(" bar", 22, 26, b),
        ]),
      ),
      (
        "foo ftp://example.com bar",
        b.vec([
          n_text("foo ", 0, 4, b),
          n(
            Macro(Link {
              scheme: Some(UrlScheme::Ftp),
              target: b.src("ftp://example.com", l(4, 21)),
              attrs: None,
            }),
            l(4, 21),
          ),
          n_text(" bar", 21, 25, b),
        ]),
      ),
      (
        "foo irc://example.com bar",
        b.vec([
          n_text("foo ", 0, 4, b),
          n(
            Macro(Link {
              scheme: Some(UrlScheme::Irc),
              target: b.src("irc://example.com", l(4, 21)),
              attrs: None,
            }),
            l(4, 21),
          ),
          n_text(" bar", 21, 25, b),
        ]),
      ),
      (
        "Ask in the https://chat.asciidoc.org[*community chat*].",
        b.vec([
          n_text("Ask in the ", 0, 11, b),
          n(
            Macro(Link {
              scheme: Some(UrlScheme::Https),
              target: b.src("https://chat.asciidoc.org", l(11, 36)),
              attrs: Some(AttrList {
                positional: b.vec([Some(b.vec([n(
                  Bold(b.vec([n_text("community chat", 38, 52, b)])),
                  l(37, 53),
                )]))]),
                ..AttrList::new(l(36, 54), b)
              }),
            }),
            l(11, 54),
          ),
          n_text(".", 54, 55, b),
        ]),
      ),
      (
        "Chat with other Fedora users in the irc://irc.freenode.org/#fedora[Fedora IRC channel].",
        b.vec([
          n_text("Chat with other Fedora users in the ", 0, 36, b),
          n(
            Macro(Link {
              scheme: Some(UrlScheme::Irc),
              target: b.src("irc://irc.freenode.org/#fedora", l(36, 66)),
              attrs: Some(AttrList {
                positional: b.vec([Some(b.vec([n_text("Fedora IRC channel", 67, 85, b)]))]),
                ..AttrList::new(l(66, 86), b)
              }),
            }),
            l(36, 86),
          ),
          n_text(".", 86, 87, b),
        ]),
      ),
      (
        "foo link:downloads/report.pdf[Get Report] bar",
        b.vec([
          n_text("foo ", 0, 4, b),
          n(
            Macro(Link {
              scheme: None,
              target: b.src("downloads/report.pdf", l(9, 29)),
              attrs: Some(AttrList {
                positional: b.vec([Some(b.vec([n_text("Get Report", 30, 40, b)]))]),
                ..AttrList::new(l(29, 41), b)
              }),
            }),
            l(4, 41),
          ),
          n_text(" bar", 41, 45, b),
        ]),
      ),
      (
        "foo <https://example.com> bar",
        b.vec([
          n_text("foo ", 0, 4, b),
          n(Discarded, l(4, 5)),
          n(bare_example_com(l(5, 24)), l(5, 24)),
          n(Discarded, l(24, 25)),
          n_text(" bar", 25, 29, b),
        ]),
      ),
      (
        "foo <https://example.com>",
        b.vec([
          n_text("foo ", 0, 4, b),
          n(Discarded, l(4, 5)),
          n(bare_example_com(l(5, 24)), l(5, 24)),
          n(Discarded, l(24, 25)),
        ]),
      ),
    ];

    run(cases, b);
  }

  #[test]
  fn test_email_macro_and_autolinks() {
    let b = &Bump::new();
    let cases = vec![
      (
        "mailto:join@discuss.example.org[Subscribe,Subscribe me]",
        b.vec([n(
          Macro(Link {
            scheme: Some(UrlScheme::Mailto),
            target: b.src("join@discuss.example.org", l(7, 31)),
            attrs: Some(AttrList {
              positional: b.vec([
                Some(b.vec([n_text("Subscribe", 32, 41, b)])),
                Some(b.vec([n_text("Subscribe me", 42, 54, b)])),
              ]),
              ..AttrList::new(l(31, 55), b)
            }),
          }),
          l(0, 55),
        )]),
      ),
      (
        "foo mailto:foo@bar.com[,,Hi] bar",
        b.vec([
          n_text("foo ", 0, 4, b),
          n(
            Macro(Link {
              scheme: Some(UrlScheme::Mailto),
              target: b.src("foo@bar.com", l(11, 22)),
              attrs: Some(AttrList {
                positional: b.vec([None, None, Some(b.vec([n_text("Hi", 25, 27, b)]))]),
                ..AttrList::new(l(22, 28), b)
              }),
            }),
            l(4, 28),
          ),
          n_text(" bar", 28, 32, b),
        ]),
      ),
      (
        "foo foo@bar.com bar",
        b.vec([
          n_text("foo ", 0, 4, b),
          n(
            Macro(Link {
              scheme: Some(UrlScheme::Mailto),
              target: b.src("foo@bar.com", l(4, 15)),
              attrs: None,
            }),
            l(4, 15),
          ),
          n_text(" bar", 15, 19, b),
        ]),
      ),
      (
        "foo@bar.com",
        b.vec([n(
          Macro(Link {
            scheme: Some(UrlScheme::Mailto),
            target: b.src("foo@bar.com", l(0, 11)),
            attrs: None,
          }),
          l(0, 11),
        )]),
      ),
      (
        "\\foo@bar.com bar", // escaped autolink
        b.vec([n(Discarded, l(0, 1)), n_text("foo@bar.com bar", 1, 16, b)]),
      ),
    ];

    run(cases, b);
  }

  #[test]
  fn test_icon_macro() {
    let b = &Bump::new();
    let cases = vec![
      (
        "foo icon:tags[] bar",
        b.vec([
          n_text("foo ", 0, 4, b),
          n(
            Macro(Icon {
              target: b.src("tags", l(9, 13)),
              attrs: AttrList { ..AttrList::new(l(13, 15), b) },
            }),
            l(4, 15),
          ),
          n_text(" bar", 15, 19, b),
        ]),
      ),
      (
        "icon:heart[2x,role=red]",
        b.vec([n(
          Macro(Icon {
            target: b.src("heart", l(5, 10)),
            attrs: AttrList {
              positional: b.vec([Some(b.vec([n_text("2x", 11, 13, b)]))]),
              named: Named::from(b.vec([(b.src("role", l(14, 18)), b.src("red", l(19, 22)))])),
              ..AttrList::new(l(10, 23), b)
            },
          }),
          l(0, 23),
        )]),
      ),
    ];

    run(cases, b);
  }

  #[test]
  fn test_pass_macro() {
    let b = &Bump::new();
    let cases = vec![
      (
        "pass:[_<foo>_]",
        b.vec([n(
          Macro(Pass {
            target: None,
            content: b.vec([n_text("_<foo>_", 6, 13, b)]),
          }),
          l(0, 14),
        )]),
      ),
      (
        "pass:c[<_foo_>].",
        b.vec([
          n(
            Macro(Pass {
              target: Some(b.src("c", l(5, 6))),
              content: b.vec([
                n(SpecialChar(SpecialCharKind::LessThan), l(7, 8)),
                n_text("_foo_", 8, 13, b),
                n(SpecialChar(SpecialCharKind::GreaterThan), l(13, 14)),
              ]),
            }),
            l(0, 15),
          ),
          n_text(".", 15, 16, b),
        ]),
      ),
      (
        "pass:c,q[<_foo_>]",
        b.vec([n(
          Macro(Pass {
            target: Some(b.src("c,q", l(5, 8))),
            content: b.vec([
              n(SpecialChar(SpecialCharKind::LessThan), l(9, 10)),
              n(Italic(b.vec([n_text("foo", 11, 14, b)])), l(10, 15)),
              n(SpecialChar(SpecialCharKind::GreaterThan), l(15, 16)),
            ]),
          }),
          l(0, 17),
        )]),
      ),
    ];

    run(cases, b);
  }

  fn run(cases: std::vec::Vec<(&str, Vec<InlineNode>)>, bump: &Bump) {
    for (input, expected) in cases {
      let mut parser = crate::Parser::new(bump, input);
      let mut block = parser.read_block().unwrap();
      let inlines = parser.parse_inlines(&mut block).unwrap();
      assert_eq!(inlines, expected);
    }
  }
}
