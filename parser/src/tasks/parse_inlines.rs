use regex::Regex;

use crate::internal::*;
use crate::tasks::parse_inlines_utils::*;
use crate::variants::token::*;
use ast::variants::{inline::*, r#macro::*};

impl<'bmp, 'src> Parser<'bmp, 'src> {
  pub(super) fn parse_inlines(
    &mut self,
    lines: &mut ContiguousLines<'bmp, 'src>,
  ) -> Result<InlineNodes<'bmp>> {
    self.parse_inlines_until(lines, &[])
  }

  fn parse_inlines_until(
    &mut self,
    lines: &mut ContiguousLines<'bmp, 'src>,
    stop_tokens: &[TokenKind],
  ) -> Result<InlineNodes<'bmp>> {
    let mut inlines = BumpVec::new_in(self.bump).into();
    if lines.is_empty() {
      return Ok(inlines);
    }
    let span_loc = lines.loc().unwrap().clamp_start();
    let mut text = CollectText::new_in(span_loc, self.bump);
    let subs = self.ctx.subs;

    while let Some(mut line) = lines.consume_current() {
      if self.ctx.list.parsing_continuations && line.is_list_continuation() {
        inlines.remove_trailing_newline();
        lines.restore_if_nonempty(line);
        return Ok(inlines);
      }

      loop {
        if line.starts_with_seq(stop_tokens) {
          line.discard(stop_tokens.len());
          text.commit_inlines(&mut inlines);
          lines.restore_if_nonempty(line);
          return Ok(inlines);
        }

        if self.ctx.delimiter.is_some() && line.current_is(DelimiterLine) {
          inlines.remove_trailing_newline();
          lines.restore_if_nonempty(line);
          return Ok(inlines);
        }

        let Some(token) = line.consume_current() else {
          if !lines.is_empty() {
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
                let mut keys = BumpVec::new_in(self.bump);
                let re = Regex::new(r"(?:\s*([^\s,+]+|[,+])\s*)").unwrap();
                for captures in re.captures_iter(&keys_src).step_by(2) {
                  let key = captures.get(1).unwrap().as_str();
                  keys.push(BumpString::from_str_in(key, self.bump));
                }
                inlines.push(node(Macro(Keyboard { keys, keys_src }), macro_loc));
                text.loc = macro_loc.clamp_end();
              }
              "footnote:" => {
                let id = line.consume_optional_macro_target(self.bump);
                lines.restore_if_nonempty(line);
                let note = self.parse_inlines_until(lines, &[CloseBracket])?;
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
                  Macro(Link { scheme, target, attrs: Some(attrs) }),
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
                  Macro(Link { scheme, target, attrs: Some(attrs) }),
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
                  bvec![in self.bump].into() // ...should probably be a diagnostic
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
                      BumpString::from_str_in(trimmed, self.bump),
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
            inlines.push(node(Macro(Link { scheme, target, attrs: None }), loc));
            inlines.push(node(Discarded, line.consume_current().unwrap().loc));
            text.loc = loc.incr_end().clamp_end();
          }

          MaybeEmail if subs.macros && EMAIL_RE.is_match(token.lexeme) => {
            text.commit_inlines(&mut inlines);
            inlines.push(node(
              Macro(Link {
                scheme: Some(UrlScheme::Mailto),
                target: SourceString::new(
                  BumpString::from_str_in(token.lexeme, self.bump),
                  token.loc,
                ),
                attrs: None,
              }),
              token.loc,
            ));
            text.loc = token.loc.clamp_end();
          }

          Underscore
            if subs.inline_formatting
              && starts_constrained(&[Underscore], &token, &line, lines) =>
          {
            self.parse_constrained(&token, Italic, &mut text, &mut inlines, line, lines)?;
            break;
          }

          Underscore
            if subs.inline_formatting && starts_unconstrained(Underscore, &token, &line, lines) =>
          {
            self.parse_unconstrained(&token, Italic, &mut text, &mut inlines, line, lines)?;
            break;
          }

          Star if subs.inline_formatting && starts_constrained(&[Star], &token, &line, lines) => {
            self.parse_constrained(&token, Bold, &mut text, &mut inlines, line, lines)?;
            break;
          }

          Star if subs.inline_formatting && starts_unconstrained(Star, &token, &line, lines) => {
            self.parse_unconstrained(&token, Bold, &mut text, &mut inlines, line, lines)?;
            break;
          }

          OpenBracket if subs.inline_formatting && line.contains_seq(&[CloseBracket, Hash]) => {
            let mut parse_token = token.clone();
            let attr_list = self.parse_formatted_text_attr_list(&mut line)?;
            debug_assert!(line.current_is(Hash));
            line.discard_assert(Hash);
            parse_token.kind = Hash;
            let wrap = |inner| TextSpan(attr_list, inner);
            if starts_unconstrained(Hash, line.current_token().unwrap(), &line, lines) {
              self.parse_unconstrained(&parse_token, wrap, &mut text, &mut inlines, line, lines)?;
            } else {
              self.parse_constrained(&parse_token, wrap, &mut text, &mut inlines, line, lines)?;
            };
            break;
          }

          Backtick
            if subs.inline_formatting
              && line.current_is(Plus)
              && contains_seq(&[Plus, Backtick], &line, lines) =>
          {
            let mut wrap_loc = token.loc;
            line.discard_assert(Plus);
            text.commit_inlines(&mut inlines);
            lines.restore_if_nonempty(line);
            self.ctx.subs.inline_formatting = false;
            let mut inner = self.parse_inlines_until(lines, &[Plus, Backtick])?;
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
            lines.restore_if_nonempty(line);
            let inner = self.parse_inlines_until(lines, &[Caret])?;
            extend(&mut loc, &inner, 1);
            inlines.push(node(Superscript(inner), loc));
            text.loc = loc.clamp_end();
            break;
          }

          Backtick
            if subs.inline_formatting && starts_constrained(&[Backtick], &token, &line, lines) =>
          {
            self.parse_constrained(&token, Mono, &mut text, &mut inlines, line, lines)?;
            break;
          }

          Backtick
            if subs.inline_formatting && starts_unconstrained(Backtick, &token, &line, lines) =>
          {
            self.parse_unconstrained(&token, Mono, &mut text, &mut inlines, line, lines)?;
            break;
          }

          DoubleQuote
            if subs.inline_formatting
              && line.current_is(Backtick)
              && starts_constrained(&[Backtick, DoubleQuote], &token, &line, lines) =>
          {
            let mut loc = token.loc;
            line.discard_assert(Backtick);
            text.commit_inlines(&mut inlines);
            lines.restore_if_nonempty(line);
            let quoted = self.parse_inlines_until(lines, &[Backtick, DoubleQuote])?;
            extend(&mut loc, &quoted, 2);
            inlines.push(node(Quote(Double, quoted), loc));
            text.loc = loc.clamp_end();
            break;
          }

          SingleQuote
            if subs.inline_formatting
              && line.current_is(Backtick)
              && starts_constrained(&[Backtick, SingleQuote], &token, &line, lines) =>
          {
            let mut loc = token.loc;
            line.discard_assert(Backtick);
            text.commit_inlines(&mut inlines);
            lines.restore_if_nonempty(line);
            let quoted = self.parse_inlines_until(lines, &[Backtick, SingleQuote])?;
            extend(&mut loc, &quoted, 2);
            inlines.push(node(Quote(Single, quoted), loc));
            text.loc = loc.clamp_end();
            break;
          }

          Tilde if subs.inline_formatting && line.is_continuous_thru(Tilde) => {
            let mut loc = token.loc;
            text.commit_inlines(&mut inlines);
            lines.restore_if_nonempty(line);
            let inner = self.parse_inlines_until(lines, &[Tilde])?;
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
            lines.restore_if_nonempty(line);
            inlines.push(node(Curly(RightDouble), loc));
            text.loc = loc.clamp_end();
            break;
          }

          DoubleQuote
            if subs.inline_formatting && line.current_is(Backtick) && stop_tokens != [Backtick] =>
          {
            let mut loc = token.loc;
            line.discard_assert(Backtick);
            loc.end += 1;
            text.commit_inlines(&mut inlines);
            lines.restore_if_nonempty(line);
            inlines.push(node(Curly(LeftDouble), loc));
            text.loc = loc.clamp_end();
            break;
          }

          Backtick if subs.inline_formatting && line.current_is(SingleQuote) => {
            let mut loc = token.loc;
            line.discard_assert(SingleQuote);
            loc.end += 1;
            text.commit_inlines(&mut inlines);
            lines.restore_if_nonempty(line);
            inlines.push(node(Curly(RightSingle), loc));
            text.loc = loc.clamp_end();
            break;
          }

          SingleQuote
            if subs.inline_formatting && line.current_is(Backtick) && stop_tokens != [Backtick] =>
          {
            let mut loc = token.loc;
            line.discard_assert(Backtick);
            loc.end += 1;
            text.commit_inlines(&mut inlines);
            lines.restore_if_nonempty(line);
            inlines.push(node(Curly(LeftSingle), loc));
            text.loc = loc.clamp_end();
            break;
          }

          Hash if subs.inline_formatting && contains_seq(&[Hash], &line, lines) => {
            self.parse_constrained(&token, Highlight, &mut text, &mut inlines, line, lines)?;
            break;
          }

          Plus
            if line.starts_with_seq(&[Plus, Plus])
              && contains_seq(&[Plus, Plus, Plus], &line, lines) =>
          {
            let mut loc = token.loc;
            line.discard_assert(Plus);
            line.discard_assert(Plus);
            text.commit_inlines(&mut inlines);
            lines.restore_if_nonempty(line);
            self.ctx.subs = Substitutions::none();
            let passthrough = self.parse_inlines_until(lines, &[Plus, Plus, Plus])?;
            extend(&mut loc, &passthrough, 3);
            self.ctx.subs = subs;
            inlines.push(node(InlinePassthrough(passthrough), loc));
            text.loc = loc.clamp_end();
            break;
          }

          Plus
            if subs.inline_formatting
              && line.current_is(Plus)
              && starts_unconstrained(Plus, &token, &line, lines) =>
          {
            self.ctx.subs.inline_formatting = false;
            self.parse_unconstrained(
              &token,
              InlinePassthrough,
              &mut text,
              &mut inlines,
              line,
              lines,
            )?;
            self.ctx.subs = subs;
            break;
          }

          Plus if subs.inline_formatting && starts_constrained(&[Plus], &token, &line, lines) => {
            self.ctx.subs.inline_formatting = false;
            self.parse_constrained(
              &token,
              InlinePassthrough,
              &mut text,
              &mut inlines,
              line,
              lines,
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
              MultiCharWhitespace(BumpString::from_str_in(token.lexeme, self.bump)),
              token.loc,
            ));
            text.loc = token.loc.clamp_end();
          }

          Discard => text.loc = token.loc.clamp_end(),

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
            inlines.push(node(Macro(Link { scheme, target, attrs: None }), loc));
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
    wrap: impl FnOnce(InlineNodes<'bmp>) -> Inline<'bmp>,
    text: &mut CollectText<'bmp>,
    inlines: &mut InlineNodes<'bmp>,
    mut line: Line<'bmp, 'src>,
    lines: &mut ContiguousLines<'bmp, 'src>,
  ) -> Result<()> {
    let mut loc = token.loc;
    let line_end = line.last_location().unwrap().end;
    line.discard_assert(token.kind);
    text.commit_inlines(inlines);
    lines.restore_if_nonempty(line);
    let inner = self.parse_inlines_until(lines, &[token.kind, token.kind])?;
    extend(&mut loc, &inner, 2);
    inlines.push(node(wrap(inner), loc));
    text.loc = loc.clamp_end();
    // if we've consumed the whole line, and there are lines to come, add a newline
    if text.loc.end == line_end && lines.current().is_some() {
      text.loc.end += 1;
      inlines.push(node(JoiningNewline, text.loc));
      text.loc = text.loc.clamp_end();
    }
    Ok(())
  }

  fn parse_constrained(
    &mut self,
    token: &Token<'src>,
    wrap: impl FnOnce(InlineNodes<'bmp>) -> Inline<'bmp>,
    text: &mut CollectText<'bmp>,
    inlines: &mut InlineNodes<'bmp>,
    line: Line<'bmp, 'src>,
    lines: &mut ContiguousLines<'bmp, 'src>,
  ) -> Result<()> {
    let mut loc = token.loc;
    let line_end = line.last_location().map_or(loc.end, |t| t.end);
    text.commit_inlines(inlines);
    lines.restore_if_nonempty(line);
    let inner = self.parse_inlines_until(lines, &[token.kind])?;
    extend(&mut loc, &inner, 1);
    inlines.push(node(wrap(inner), loc));
    text.loc = loc.clamp_end();
    // if we've consumed the whole line, and there are lines to come, add a newline
    if text.loc.end == line_end && lines.current().is_some() {
      text.loc.end += 1;
      inlines.push(node(JoiningNewline, text.loc));
      text.loc = text.loc.clamp_end();
    }
    Ok(())
  }

  fn merge_inlines(
    &self,
    a: &mut BumpVec<'bmp, Inline<'bmp>>,
    b: &mut BumpVec<'bmp, Inline<'bmp>>,
    append: Option<&str>,
  ) {
    if let (Some(Text(a_text)), Some(Text(b_text))) = (a.last_mut(), b.first_mut()) {
      a_text.push_str(b_text);
      b.remove(0);
    }
    a.append(b);
    match (append, a.last_mut()) {
      (Some(append), Some(Text(text))) => text.push_str(append),
      (Some(append), _) => a.push(Text(BumpString::from_str_in(append, self.bump))),
      _ => {}
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::test::*;
  use pretty_assertions::assert_eq;

  #[test]
  fn test_joining_newlines() {
    let b = &Bump::new();
    let cases = vec![
      (
        "_foo_\nbar",
        b.inodes([
          n(Italic(b.inodes([n_text("foo", 1, 4, b)])), l(0, 5)),
          n(JoiningNewline, l(5, 6)),
          n_text("bar", 6, 9, b),
        ]),
      ),
      (
        "__foo__\nbar",
        b.inodes([
          n(Italic(b.inodes([n_text("foo", 2, 5, b)])), l(0, 7)),
          n(JoiningNewline, l(7, 8)),
          n_text("bar", 8, 11, b),
        ]),
      ),
    ];

    run(cases, b);
  }

  #[test]
  fn test_parse_inlines() {
    let b = &Bump::new();
    let cases = vec![
      (
        "+_foo_+",
        b.inodes([n(
          InlinePassthrough(b.inodes([n_text("_foo_", 1, 6, b)])),
          l(0, 7),
        )]),
      ),
      (
        "`*_foo_*`",
        b.inodes([n(
          Mono(b.inodes([n(
            Bold(b.inodes([n(Italic(b.inodes([n_text("foo", 3, 6, b)])), l(2, 7))])),
            l(1, 8),
          )])),
          l(0, 9),
        )]),
      ),
      (
        "+_foo\nbar_+",
        // not sure if this is "spec", but it's what asciidoctor currently does
        b.inodes([n(
          InlinePassthrough(b.inodes([
            n_text("_foo", 1, 5, b),
            n(JoiningNewline, l(5, 6)),
            n_text("bar_", 6, 10, b),
          ])),
          l(0, 11),
        )]),
      ),
      (
        "+_<foo>&_+",
        b.inodes([n(
          InlinePassthrough(b.inodes([
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
        b.inodes([
          n_text("rofl ", 0, 5, b),
          n(
            InlinePassthrough(b.inodes([n_text("_foo_", 6, 11, b)])),
            l(5, 12),
          ),
          n_text(" lol", 12, 16, b),
        ]),
      ),
      (
        "++_foo_++bar",
        b.inodes([
          n(
            InlinePassthrough(b.inodes([n_text("_foo_", 2, 7, b)])),
            l(0, 9),
          ),
          n_text("bar", 9, 12, b),
        ]),
      ),
      (
        "+++_<foo>&_+++ bar",
        b.inodes([
          n(
            InlinePassthrough(b.inodes([n_text("_<foo>&_", 3, 11, b)])),
            l(0, 14),
          ),
          n_text(" bar", 14, 18, b),
        ]),
      ),
      (
        "foo #bar#",
        b.inodes([
          n_text("foo ", 0, 4, b),
          n(Highlight(b.inodes([n_text("bar", 5, 8, b)])), l(4, 9)),
        ]),
      ),
      (
        "image::foo.png[]", // unexpected block macro, parse as text
        b.inodes([n_text("image::foo.png[]", 0, 16, b)]),
      ),
      (
        "foo `bar`",
        b.inodes([
          n_text("foo ", 0, 4, b),
          n(Mono(b.inodes([n_text("bar", 5, 8, b)])), l(4, 9)),
        ]),
      ),
      (
        "foo b``ar``",
        b.inodes([
          n_text("foo b", 0, 5, b),
          n(Mono(b.inodes([n_text("ar", 7, 9, b)])), l(5, 11)),
        ]),
      ),
      (
        "foo *bar*",
        b.inodes([
          n_text("foo ", 0, 4, b),
          n(Bold(b.inodes([n_text("bar", 5, 8, b)])), l(4, 9)),
        ]),
      ),
      (
        "foo b**ar**",
        b.inodes([
          n_text("foo b", 0, 5, b),
          n(Bold(b.inodes([n_text("ar", 7, 9, b)])), l(5, 11)),
        ]),
      ),
      (
        "foo ~bar~ baz",
        b.inodes([
          n_text("foo ", 0, 4, b),
          n(Subscript(b.inodes([n_text("bar", 5, 8, b)])), l(4, 9)),
          n_text(" baz", 9, 13, b),
        ]),
      ),
      (
        "foo _bar\nbaz_",
        b.inodes([
          n_text("foo ", 0, 4, b),
          n(
            Italic(b.inodes([
              n_text("bar", 5, 8, b),
              n(JoiningNewline, l(8, 9)),
              n_text("baz", 9, 12, b),
            ])),
            l(4, 13),
          ),
        ]),
      ),
      ("foo __bar", b.inodes([n_text("foo __bar", 0, 9, b)])),
      (
        "foo _bar baz_",
        b.inodes([
          n_text("foo ", 0, 4, b),
          n(Italic(b.inodes([n_text("bar baz", 5, 12, b)])), l(4, 13)),
        ]),
      ),
      (
        "foo _bar_",
        b.inodes([
          n_text("foo ", 0, 4, b),
          n(Italic(b.inodes([n_text("bar", 5, 8, b)])), l(4, 9)),
        ]),
      ),
      (
        "foo b__ar__",
        b.inodes([
          n_text("foo b", 0, 5, b),
          n(Italic(b.inodes([n_text("ar", 7, 9, b)])), l(5, 11)),
        ]),
      ),
      ("foo 'bar'", b.inodes([n_text("foo 'bar'", 0, 9, b)])),
      ("foo \"bar\"", b.inodes([n_text("foo \"bar\"", 0, 9, b)])),
      (
        "foo `\"bar\"`",
        b.inodes([
          n_text("foo ", 0, 4, b),
          n(Mono(b.inodes([n_text("\"bar\"", 5, 10, b)])), l(4, 11)),
        ]),
      ),
      (
        "foo `'bar'`",
        b.inodes([
          n_text("foo ", 0, 4, b),
          n(Mono(b.inodes([n_text("'bar'", 5, 10, b)])), l(4, 11)),
        ]),
      ),
      (
        "foo \"`bar`\"",
        b.inodes([
          n_text("foo ", 0, 4, b),
          n(
            Quote(QuoteKind::Double, b.inodes([n_text("bar", 6, 9, b)])),
            l(4, 11),
          ),
        ]),
      ),
      (
        "foo \"`bar baz`\"",
        b.inodes([
          n_text("foo ", 0, 4, b),
          n(
            Quote(QuoteKind::Double, b.inodes([n_text("bar baz", 6, 13, b)])),
            l(4, 15),
          ),
        ]),
      ),
      (
        "foo \"`bar\nbaz`\"",
        b.inodes([
          n_text("foo ", 0, 4, b),
          n(
            Quote(
              QuoteKind::Double,
              b.inodes([
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
        b.inodes([
          n_text("foo ", 0, 4, b),
          n(
            Quote(QuoteKind::Single, b.inodes([n_text("bar", 6, 9, b)])),
            l(4, 11),
          ),
        ]),
      ),
      (
        "Olaf's wrench",
        b.inodes([
          n_text("Olaf", 0, 4, b),
          n(Curly(LegacyImplicitApostrophe), l(4, 5)),
          n_text("s wrench", 5, 13, b),
        ]),
      ),
      (
        "foo   bar",
        b.inodes([
          n_text("foo", 0, 3, b),
          n(MultiCharWhitespace(b.s("   ")), l(3, 6)),
          n_text("bar", 6, 9, b),
        ]),
      ),
      (
        "`+{name}+`",
        b.inodes([n(LitMono(b.src("{name}", l(2, 8))), l(0, 10))]),
      ),
      (
        "`+_foo_+`",
        b.inodes([n(LitMono(b.src("_foo_", l(2, 7))), l(0, 9))]),
      ),
      (
        "foo <bar> & lol",
        b.inodes([
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
        b.inodes([n(Superscript(b.inodes([n_text("bar", 1, 4, b)])), l(0, 5))]),
      ),
      (
        "^bar^",
        b.inodes([n(Superscript(b.inodes([n_text("bar", 1, 4, b)])), l(0, 5))]),
      ),
      ("foo ^bar", b.inodes([n_text("foo ^bar", 0, 8, b)])),
      ("foo bar^", b.inodes([n_text("foo bar^", 0, 8, b)])),
      (
        "foo ^bar^ foo",
        b.inodes([
          n_text("foo ", 0, 4, b),
          n(Superscript(b.inodes([n_text("bar", 5, 8, b)])), l(4, 9)),
          n_text(" foo", 9, 13, b),
        ]),
      ),
      (
        "doublefootnote:[ymmv _i_]bar",
        b.inodes([
          n_text("double", 0, 6, b),
          n(
            Macro(Footnote {
              id: None,
              text: b.inodes([
                n_text("ymmv ", 16, 21, b),
                n(Italic(b.inodes([n_text("i", 22, 23, b)])), l(21, 24)),
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
        b.inodes([
          n_text("press the ", 0, 10, b),
          n(Macro(Button(b.src("OK", l(15, 17)))), l(10, 18)),
          n_text(" button", 18, 25, b),
        ]),
      ),
      (
        "btn:[Open]",
        b.inodes([n(Macro(Button(b.src("Open", l(5, 9)))), l(0, 10))]),
      ),
      (
        "select menu:File[Save].",
        b.inodes([
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
        b.inodes([n(
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

  fn run(cases: Vec<(&str, InlineNodes)>, bump: &Bump) {
    for (input, expected) in cases {
      let mut parser = Parser::new(bump, input);
      let mut block = parser.read_lines().unwrap();
      let inlines = parser.parse_inlines(&mut block).unwrap();
      assert_eq!(inlines, expected, "input was: `{}`", input);
    }
  }
}
