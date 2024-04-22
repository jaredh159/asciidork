use regex::Regex;

use crate::internal::*;
use crate::tasks::parse_inlines_utils::*;
use crate::variants::token::*;
use ast::variants::{inline::*, r#macro::*};

struct Accum<'bmp> {
  inlines: InlineNodes<'bmp>,
  text: CollectText<'bmp>,
}

impl<'bmp> Accum<'bmp> {
  fn commit(&mut self) {
    self.text.commit_inlines(&mut self.inlines);
  }

  fn push_node(&mut self, node: Inline<'bmp>, loc: SourceLocation) {
    self.commit();
    self.inlines.push(InlineNode::new(node, loc));
    self.text.loc = loc.clamp_end();
  }
}

impl<'bmp, 'src> Parser<'bmp, 'src> {
  pub(crate) fn parse_inlines(
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
    let inlines = BumpVec::new_in(self.bump).into();
    if lines.is_empty() {
      return Ok(inlines);
    }
    let span_loc = lines.loc().unwrap().clamp_start();
    let text = CollectText::new_in(span_loc, self.bump);
    let subs = self.ctx.subs;
    let mut acc = Accum { inlines, text };

    while let Some(mut line) = lines.consume_current() {
      if self.should_stop_at(&line) {
        acc.inlines.remove_trailing_newline();
        lines.restore_if_nonempty(line);
        return Ok(acc.inlines);
      }

      if line.is_comment() && !subs.callouts() {
        if lines.is_empty() {
          acc.inlines.discard_trailing_newline();
        }
        let token = line.consume_current().unwrap();
        let comment = line.consume_to_string(self.bump);
        let loc = SourceLocation::new(token.loc.start, comment.loc.end + 1);
        acc.push_node(LineComment(comment.src), loc);
        continue;
      }

      loop {
        if line.starts_with_seq(stop_tokens) {
          line.discard(stop_tokens.len());
          acc.commit();
          lines.restore_if_nonempty(line);
          return Ok(acc.inlines);
        }

        let Some(token) = line.consume_current() else {
          if !lines.is_empty() {
            acc.commit();
            acc.text.loc.end += 1;
            acc.push_node(JoiningNewline, acc.text.loc);
          }
          break;
        };

        match token.kind {
          MacroName if subs.macros() && line.continues_inline_macro() => {
            let mut macro_loc = token.loc;
            let line_end = line.last_location().unwrap();
            acc.commit();
            match token.lexeme {
              "image:" => {
                let target = line.consume_macro_target(self.bump);
                let attrs = self.parse_attr_list(&mut line)?;
                finish_macro(&line, &mut macro_loc, line_end, &mut acc.text);
                acc.push_node(
                  Macro(Image { flow: Flow::Inline, target, attrs }),
                  macro_loc,
                );
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
                acc.push_node(Macro(Keyboard { keys, keys_src }), macro_loc);
              }
              "footnote:" => {
                let id = line.consume_optional_macro_target(self.bump);
                lines.restore_if_nonempty(line);
                let note = self.parse_inlines_until(lines, &[CloseBracket])?;
                extend(&mut macro_loc, &note, 1);
                acc.push_node(Macro(Footnote { id, text: note }), macro_loc);
                break;
              }
              "xref:" => {
                let id = line.consume_macro_target(self.bump);
                self.ctx.xrefs.insert(id.src.clone(), id.loc);
                lines.restore_if_nonempty(line);
                let target_nodes = self.parse_inlines_until(lines, &[CloseBracket])?;
                let target = if target_nodes.is_empty() {
                  macro_loc.end = id.loc.end + 2;
                  None
                } else {
                  extend(&mut macro_loc, &target_nodes, 1);
                  Some(target_nodes)
                };
                acc.push_node(Macro(Xref { id, target }), macro_loc);
                break;
              }
              "mailto:" | "link:" => {
                let target = line.consume_macro_target(self.bump);
                let attrs = self.parse_attr_list(&mut line)?;
                finish_macro(&line, &mut macro_loc, line_end, &mut acc.text);
                let scheme = token.to_url_scheme();
                acc.push_node(
                  Macro(Link { scheme, target, attrs: Some(attrs) }),
                  macro_loc,
                );
              }
              "https:" | "http:" | "irc:" => {
                let target = line.consume_url(Some(&token), self.bump);
                line.discard_assert(OpenBracket);
                let attrs = self.parse_attr_list(&mut line)?;
                finish_macro(&line, &mut macro_loc, line_end, &mut acc.text);
                let scheme = Some(token.to_url_scheme().unwrap());
                acc.push_node(
                  Macro(Link { scheme, target, attrs: Some(attrs) }),
                  macro_loc,
                );
              }
              "pass:" => {
                let target = line.consume_optional_macro_target(self.bump);
                self.ctx.subs = Substitutions::from_pass_macro_target(&target);
                let mut attrs = self.parse_attr_list(&mut line)?;
                self.ctx.subs = subs;
                finish_macro(&line, &mut macro_loc, line_end, &mut acc.text);
                let content = if !attrs.positional.is_empty() && attrs.positional[0].is_some() {
                  attrs.positional[0].take().unwrap()
                } else {
                  bvec![in self.bump].into() // ...should probably be a diagnostic
                };
                acc.push_node(Macro(Pass { target, content }), macro_loc);
              }
              "icon:" => {
                let target = line.consume_macro_target(self.bump);
                let attrs = self.parse_attr_list(&mut line)?;
                finish_macro(&line, &mut macro_loc, line_end, &mut acc.text);
                acc.push_node(Macro(Icon { target, attrs }), macro_loc);
              }
              "btn:" => {
                line.discard_assert(OpenBracket);
                let btn = line.consume_to_string_until(CloseBracket, self.bump);
                line.discard_assert(CloseBracket);
                finish_macro(&line, &mut macro_loc, line_end, &mut acc.text);
                acc.push_node(Macro(Button(btn)), macro_loc);
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
                finish_macro(&line, &mut macro_loc, line_end, &mut acc.text);
                acc.push_node(Macro(Menu(items)), macro_loc);
              }
              _ => todo!("unhandled macro type: `{}`", token.lexeme),
            }
          }

          TermDelimiter
            if subs.callouts()
              && token.len() == 2
              && line.current_is(Whitespace)
              && token.lexeme == ";;" // this is a happy accident
              && line.continues_valid_callout_nums() =>
          {
            self.push_callout_tuck(&token, &mut line, &mut acc);
          }

          Hash
            if subs.callouts()
              && line.current_is(Whitespace)
              && line.continues_valid_callout_nums() =>
          {
            self.push_callout_tuck(&token, &mut line, &mut acc);
          }

          ForwardSlashes
            if subs.callouts()
              && token.len() == 2
              && line.current_is(Whitespace)
              && line.continues_valid_callout_nums() =>
          {
            self.push_callout_tuck(&token, &mut line, &mut acc);
          }

          CalloutNumber if subs.callouts() && line.continues_valid_callout_nums() => {
            self.recover_custom_line_comment(&mut acc);
            acc.text.trim_end();
            let loc = SourceLocation::new(acc.text.loc.end, token.loc.end);
            let number = token.parse_callout_num();
            acc.push_node(CalloutNum(self.ctx.push_callout(number)), loc);
          }

          CalloutNumber if subs.special_chars() => {
            let start = token.loc.clamp_start().incr_end();
            let end = token.loc.clamp_end().decr_start();
            acc.push_node(SpecialChar(SpecialCharKind::LessThan), start);
            acc.text.push_str(&token.lexeme[1..token.lexeme.len() - 1]);
            acc.push_node(SpecialChar(SpecialCharKind::GreaterThan), end);
          }

          LessThan if line.continues_xref_shorthand() => {
            let mut loc = token.loc;
            line.discard_assert(LessThan);
            let mut inner = line.extract_line_before(&[GreaterThan, GreaterThan], self.bump);
            let id = inner.consume_to_string_until(Comma, self.bump);
            self.ctx.xrefs.insert(id.src.clone(), id.loc);
            let target = if !inner.is_empty() {
              inner.discard_assert(Comma);
              let mut target_lines = inner.into_lines_in(self.bump);
              Some(self.parse_inlines(&mut target_lines)?)
            } else {
              None
            };
            line.discard_assert(GreaterThan);
            loc.end = line.consume_current().unwrap().loc.end;
            acc.push_node(Macro(Xref { id, target }), loc);
          }

          LessThan
            if subs.macros()
              && line.current_token().is_url_scheme()
              && line.is_continuous_thru(GreaterThan) =>
          {
            acc.push_node(Discarded, token.loc);
            let scheme_token = line.consume_current().unwrap();
            let mut loc = scheme_token.loc;
            let line_end = line.last_location().unwrap();
            let target = line.consume_url(Some(&scheme_token), self.bump);
            finish_macro(&line, &mut loc, line_end, &mut acc.text);
            let scheme = Some(scheme_token.to_url_scheme().unwrap());
            acc.push_node(Macro(Link { scheme, target, attrs: None }), loc);
            acc.push_node(Discarded, line.consume_current().unwrap().loc);
          }

          MaybeEmail if subs.macros() && EMAIL_RE.is_match(token.lexeme) => {
            acc.push_node(
              Macro(Link {
                scheme: Some(UrlScheme::Mailto),
                target: token.to_source_string(self.bump),
                attrs: None,
              }),
              token.loc,
            );
          }

          Underscore
            if subs.inline_formatting()
              && starts_constrained(&[Underscore], &token, &line, lines) =>
          {
            self.parse_constrained(&token, Italic, &mut acc, line, lines)?;
            break;
          }

          Underscore
            if subs.inline_formatting()
              && starts_unconstrained(Underscore, &token, &line, lines) =>
          {
            self.parse_unconstrained(&token, Italic, &mut acc, line, lines)?;
            break;
          }

          Star if subs.inline_formatting() && starts_constrained(&[Star], &token, &line, lines) => {
            self.parse_constrained(&token, Bold, &mut acc, line, lines)?;
            break;
          }

          Star if subs.inline_formatting() && starts_unconstrained(Star, &token, &line, lines) => {
            self.parse_unconstrained(&token, Bold, &mut acc, line, lines)?;
            break;
          }

          OpenBracket if subs.inline_formatting() && line.contains_seq(&[CloseBracket, Hash]) => {
            let mut parse_token = token.clone();
            let attr_list = self.parse_formatted_text_attr_list(&mut line)?;
            debug_assert!(line.current_is(Hash));
            line.discard_assert(Hash);
            parse_token.kind = Hash;
            let wrap = |inner| TextSpan(attr_list, inner);
            if starts_unconstrained(Hash, line.current_token().unwrap(), &line, lines) {
              self.parse_unconstrained(&parse_token, wrap, &mut acc, line, lines)?;
            } else {
              self.parse_constrained(&parse_token, wrap, &mut acc, line, lines)?;
            };
            break;
          }

          Backtick
            if subs.inline_formatting()
              && line.current_is(Plus)
              && contains_seq(&[Plus, Backtick], &line, lines) =>
          {
            self.ctx.subs.remove(Subs::InlineFormatting);
            self.ctx.subs.remove(Subs::AttrRefs);
            self.parse_inner(
              &token,
              [Plus, Backtick],
              |mut inner| {
                assert!(inner.len() == 1, "invalid lit mono");
                match inner.pop().unwrap() {
                  InlineNode { content: Text(lit), loc } => LitMono(SourceString::new(lit, loc)),
                  _ => panic!("invalid lit mono"),
                }
              },
              &mut acc,
              line,
              lines,
            )?;
            self.ctx.subs = subs;
            break;
          }

          Caret if subs.inline_formatting() && line.is_continuous_thru(Caret) => {
            self.parse_inner(&token, [Caret], Superscript, &mut acc, line, lines)?;
            break;
          }

          Backtick
            if subs.inline_formatting()
              && starts_constrained(&[Backtick], &token, &line, lines) =>
          {
            self.parse_constrained(&token, Mono, &mut acc, line, lines)?;
            break;
          }

          Backtick
            if subs.inline_formatting() && starts_unconstrained(Backtick, &token, &line, lines) =>
          {
            self.parse_unconstrained(&token, Mono, &mut acc, line, lines)?;
            break;
          }

          DoubleQuote
            if subs.inline_formatting()
              && line.current_is(Backtick)
              && starts_constrained(&[Backtick, DoubleQuote], &token, &line, lines) =>
          {
            self.parse_inner(
              &token,
              [Backtick, DoubleQuote],
              |inner| Quote(Double, inner),
              &mut acc,
              line,
              lines,
            )?;
            break;
          }

          SingleQuote
            if subs.inline_formatting()
              && line.current_is(Backtick)
              && starts_constrained(&[Backtick, SingleQuote], &token, &line, lines) =>
          {
            self.parse_inner(
              &token,
              [Backtick, SingleQuote],
              |inner| Quote(Single, inner),
              &mut acc,
              line,
              lines,
            )?;
            break;
          }

          Tilde if subs.inline_formatting() && line.is_continuous_thru(Tilde) => {
            self.parse_inner(&token, [Tilde], Subscript, &mut acc, line, lines)?;
            break;
          }

          Backtick if subs.inline_formatting() && line.current_is(DoubleQuote) => {
            push_simple(CurlyQuote(RightDouble), &token, line, &mut acc, lines);
            break;
          }

          DoubleQuote
            if subs.inline_formatting()
              && line.current_is(Backtick)
              && stop_tokens != [Backtick] =>
          {
            push_simple(CurlyQuote(LeftDouble), &token, line, &mut acc, lines);
            break;
          }

          Backtick if subs.inline_formatting() && line.current_is(SingleQuote) => {
            push_simple(CurlyQuote(RightSingle), &token, line, &mut acc, lines);
            break;
          }

          SingleQuote
            if subs.inline_formatting()
              && line.current_is(Backtick)
              && stop_tokens != [Backtick] =>
          {
            push_simple(CurlyQuote(LeftSingle), &token, line, &mut acc, lines);
            break;
          }

          Hash if subs.inline_formatting() && contains_seq(&[Hash], &line, lines) => {
            self.parse_constrained(&token, Highlight, &mut acc, line, lines)?;
            break;
          }

          Plus
            if line.starts_with_seq(&[Plus, Plus])
              && contains_seq(&[Plus, Plus, Plus], &line, lines) =>
          {
            self.ctx.subs = Substitutions::none();
            self.parse_inner(
              &token,
              [Plus, Plus, Plus],
              InlinePassthrough,
              &mut acc,
              line,
              lines,
            )?;
            self.ctx.subs = subs;
            break;
          }

          Plus
            if subs.inline_formatting()
              && line.current_is(Plus)
              && starts_unconstrained(Plus, &token, &line, lines) =>
          {
            self.ctx.subs.remove(Subs::InlineFormatting);
            self.parse_unconstrained(&token, InlinePassthrough, &mut acc, line, lines)?;
            self.ctx.subs = subs;
            break;
          }

          Plus if subs.inline_formatting() && starts_constrained(&[Plus], &token, &line, lines) => {
            self.ctx.subs.remove(Subs::InlineFormatting);
            self.parse_constrained(&token, InlinePassthrough, &mut acc, line, lines)?;
            self.ctx.subs = subs;
            break;
          }

          Ampersand | LessThan | GreaterThan if subs.special_chars() => {
            acc.push_node(
              SpecialChar(match token.kind {
                Ampersand => SpecialCharKind::Ampersand,
                LessThan => SpecialCharKind::LessThan,
                GreaterThan => SpecialCharKind::GreaterThan,
                _ => unreachable!(),
              }),
              token.loc,
            );
          }

          SingleQuote if line.current_is(Word) && subs.inline_formatting() => {
            if acc.text.is_empty() || acc.text.ends_with(char::is_whitespace) {
              acc.text.push_token(&token);
            } else {
              acc.push_node(CurlyQuote(LegacyImplicitApostrophe), token.loc);
            }
          }

          Whitespace if token.lexeme.len() > 1 && subs.inline_formatting() => {
            acc.push_node(MultiCharWhitespace(token.to_string(self.bump)), token.loc);
          }

          Whitespace if line.current_is(Plus) && line.num_tokens() == 1 => {
            let mut loc = token.loc;
            line.discard_assert(Plus);
            loc.end += 2; // plus and newline
            acc.push_node(LineBreak, loc);
            break;
          }

          OpenBrace if subs.attr_refs() && line.is_continuous_thru(CloseBrace) => {
            let mut loc = token.loc;
            let aref = line.consume_to_string_until(CloseBrace, self.bump).src;
            let close_brace = line.consume_current().unwrap();
            loc.end = close_brace.loc.end;
            acc.push_node(AttributeReference(aref), loc);
          }

          Discard => acc.text.loc = token.loc.clamp_end(),

          Backslash if escapes_pattern(&line, &subs) => {
            acc.push_node(Discarded, token.loc);
            // pushing the next token as text prevents recognizing the pattern
            let next_token = line.consume_current().unwrap();
            acc.text.push_token(&next_token);
          }

          _ if subs.macros() && token.is_url_scheme() && line.src.starts_with("//") => {
            let mut loc = token.loc;
            let line_end = line.last_location().unwrap();
            let target = line.consume_url(Some(&token), self.bump);
            finish_macro(&line, &mut loc, line_end, &mut acc.text);
            let scheme = Some(token.to_url_scheme().unwrap());
            acc.push_node(Macro(Link { scheme, target, attrs: None }), loc);
          }

          _ => {
            acc.text.push_token(&token);
          }
        }
      }
    }

    acc.commit();
    Ok(acc.inlines)
  }

  fn parse_inner<const N: usize>(
    &mut self,
    start: &Token<'src>,
    stop_tokens: [TokenKind; N],
    wrap: impl FnOnce(InlineNodes<'bmp>) -> Inline<'bmp>,
    state: &mut Accum<'bmp>,
    mut line: Line<'bmp, 'src>,
    lines: &mut ContiguousLines<'bmp, 'src>,
  ) -> Result<()> {
    let mut loc = start.loc;
    stop_tokens
      .iter()
      .take(N - 1)
      .for_each(|&kind| line.discard_assert(kind));
    lines.restore_if_nonempty(line);
    let inner = self.parse_inlines_until(lines, &stop_tokens)?;
    extend(&mut loc, &inner, N);
    state.push_node(wrap(inner), loc);
    push_newline_if_needed(state, lines);
    Ok(())
  }

  fn parse_unconstrained(
    &mut self,
    token: &Token<'src>,
    wrap: impl FnOnce(InlineNodes<'bmp>) -> Inline<'bmp>,
    state: &mut Accum<'bmp>,
    line: Line<'bmp, 'src>,
    lines: &mut ContiguousLines<'bmp, 'src>,
  ) -> Result<()> {
    self.parse_inner(token, [token.kind, token.kind], wrap, state, line, lines)
  }

  fn parse_constrained(
    &mut self,
    token: &Token<'src>,
    wrap: impl FnOnce(InlineNodes<'bmp>) -> Inline<'bmp>,
    state: &mut Accum<'bmp>,
    line: Line<'bmp, 'src>,
    lines: &mut ContiguousLines<'bmp, 'src>,
  ) -> Result<()> {
    self.parse_inner(token, [token.kind], wrap, state, line, lines)
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

  fn should_stop_at(&self, line: &Line<'bmp, 'src>) -> bool {
    if line.current_is(DelimiterLine) && self.ctx.can_nest_blocks {
      return true;
    }

    // description list
    (
      self.ctx.list.stack.parsing_description_list()
      && line.starts_description_list_item() || line.is_list_continuation()
    )

    // list continuation
    || (self.ctx.list.parsing_continuations && line.is_list_continuation())

    // special case: ending verbatim delimited block, non-matching delimiters
    // within the verbatim block are rendered as is
    || (
      self.ctx.delimiter.is_some()
      && self.ctx.delimiter == line.current_token().and_then(|t| t.to_delimeter())
    )
  }

  fn push_callout_tuck(
    &mut self,
    token: &Token<'src>,
    line: &mut Line<'bmp, 'src>,
    state: &mut Accum<'bmp>,
  ) {
    let mut tuck = token.to_source_string(self.bump);
    line.discard_assert(Whitespace);
    tuck.src.push(' ');
    tuck.loc.end += 1;
    state.push_node(CalloutTuck(tuck.src), tuck.loc);
  }

  fn recover_custom_line_comment(&mut self, state: &mut Accum<'bmp>) {
    let Some(ref comment_bytes) = self.ctx.custom_line_comment else {
      return;
    };
    let mut line_txt = state.text.str().as_bytes();
    let line_len = line_txt.len();
    let mut back = comment_bytes.len();
    if line_txt.ends_with(&[b' ']) {
      back += 1;
      line_txt = &line_txt[..line_len - 1];
    }
    if line_txt.ends_with(comment_bytes) {
      let tuck = state.text.str().split_at(line_len - back).1;
      let tuck = BumpString::from_str_in(tuck, self.bump);
      state.text.drop_last(back);
      let tuck_loc = SourceLocation::new(state.text.loc.end, state.text.loc.end + back);
      state.push_node(CalloutTuck(tuck), tuck_loc);
    }
  }
}

fn escapes_pattern(line: &Line, subs: &Substitutions) -> bool {
  // escaped email
  subs.macros() && (line.current_is(MaybeEmail) || line.current_token().is_url_scheme())
  // escaped attr ref
  || subs.attr_refs() && line.current_is(OpenBrace) && line.is_continuous_thru(CloseBrace)
}

fn push_newline_if_needed<'bmp>(state: &mut Accum<'bmp>, lines: &ContiguousLines<'bmp, '_>) {
  // LOGIC: if we have a current line and it is entirely unconsumed
  // it means we've finished parsing something at the end of the prev line
  // so we need to join with a newline, examples:

  // 1) here we have a line, but it's partially consumed
  //                v -- ended here: don't add newline
  // footnote:[_foo_]

  // 2) here we're sitting at the beginning of line 3, so we need a newline
  // "`foo
  //      v -- ended here: add newline
  // bar`"
  // baz

  // 3) here we have no next line, so we don't add a newline
  // "`foo
  //      v -- ended here: don't add newline
  // bar`"

  if lines
    .current()
    .map_or(false, |line| line.is_fully_unconsumed())
  {
    state.text.loc.end += 1;
    state.push_node(JoiningNewline, state.text.loc);
  }
}

fn push_simple<'bmp, 'src>(
  inline_node: Inline<'bmp>,
  token: &Token<'src>,
  mut line: Line<'bmp, 'src>,
  state: &mut Accum<'bmp>,
  lines: &mut ContiguousLines<'bmp, 'src>,
) {
  let mut loc = token.loc;
  line.discard(1);
  loc.end += 1;
  lines.restore_if_nonempty(line);
  state.push_node(inline_node, loc);
  push_newline_if_needed(state, lines);
}

#[cfg(test)]
mod tests {
  use super::*;
  use test_utils::{assert_eq, *};

  #[test]
  fn test_line_comments() {
    let cases = vec![(
      "foo\n// baz\nbar",
      nodes![
        node!("foo"; 0..3),
        node!(JoiningNewline, 3..4),
        node!(LineComment(bstr!(" baz")), 4..11),
        node!("bar"; 11..14),
      ],
    )];
    run(cases);
  }

  #[test]
  fn test_joining_newlines() {
    let cases = vec![
      (
        "{foo}",
        nodes![node!(AttributeReference(bstr!("foo")), 0..5)],
      ),
      (
        "\\{foo}",
        nodes![node!(Discarded, 0..1), node!("{foo}"; 1..6)],
      ),
      (
        "_foo_\nbar",
        nodes![
          node!(Italic(nodes![node!("foo"; 1..4)]), 0..5),
          node!(JoiningNewline, 5..6),
          node!("bar"; 6..9),
        ],
      ),
      (
        "__foo__\nbar",
        nodes![
          node!(Italic(nodes![node!("foo"; 2..5)]), 0..7),
          node!(JoiningNewline, 7..8),
          node!("bar"; 8..11),
        ],
      ),
      (
        "foo \"`bar`\"\nbaz",
        nodes![
          node!("foo "; 0..4),
          node!(Quote(QuoteKind::Double, nodes![node!("bar"; 6..9)]), 4..11),
          node!(JoiningNewline, 11..12),
          node!("baz"; 12..15),
        ],
      ),
      (
        "\"`foo\nbar`\"\nbaz",
        nodes![
          node!(
            Quote(
              QuoteKind::Double,
              nodes![
                node!("foo"; 2..5),
                node!(JoiningNewline, 5..6),
                node!("bar"; 6..9),
              ],
            ),
            0..11,
          ),
          node!(JoiningNewline, 11..12),
          node!("baz"; 12..15),
        ],
      ),
      (
        "bar`\"\nbaz",
        nodes![
          node!("bar"; 0..3),
          node!(CurlyQuote(RightDouble), 3..5),
          node!(JoiningNewline, 5..6),
          node!("baz"; 6..9),
        ],
      ),
      (
        "^foo^\nbar",
        nodes![
          node!(Superscript(nodes![node!("foo"; 1..4)]), 0..5),
          node!(JoiningNewline, 5..6),
          node!("bar"; 6..9),
        ],
      ),
      (
        "~foo~\nbar",
        nodes![
          node!(Subscript(nodes![node!("foo"; 1..4)]), 0..5),
          node!(JoiningNewline, 5..6),
          node!("bar"; 6..9),
        ],
      ),
      (
        "`+{name}+`\nbar",
        nodes![
          node!(LitMono(src!("{name}", 2..8)), 0..10),
          node!(JoiningNewline, 10..11),
          node!("bar"; 11..14),
        ],
      ),
      (
        "+_foo_+\nbar",
        nodes![
          node!(InlinePassthrough(nodes![node!("_foo_"; 1..6)]), 0..7,),
          node!(JoiningNewline, 7..8),
          node!("bar"; 8..11),
        ],
      ),
      (
        "+++_<foo>&_+++\nbar",
        nodes![
          node!(InlinePassthrough(nodes![node!("_<foo>&_"; 3..11)]), 0..14,),
          node!(JoiningNewline, 14..15),
          node!("bar"; 15..18),
        ],
      ),
    ];

    run(cases);
  }

  #[test]
  fn test_line_breaks() {
    let cases = vec![
      (
        "foo +\nbar",
        nodes![
          node!("foo"; 0..3),
          node!(LineBreak, 3..6),
          node!("bar"; 6..9),
        ],
      ),
      (
        "foo+\nbar", // not valid linebreak
        nodes![
          node!("foo+"; 0..4),
          node!(JoiningNewline, 4..5),
          node!("bar"; 5..8),
        ],
      ),
    ];

    run(cases);
  }

  #[test]
  fn test_parse_inlines() {
    let cases = vec![
      (
        "+_foo_+",
        nodes![node!(InlinePassthrough(nodes![node!("_foo_"; 1..6)]), 0..7,)],
      ),
      (
        "`*_foo_*`",
        nodes![node!(
          Mono(nodes![node!(
            Bold(nodes![node!(Italic(nodes![node!("foo"; 3..6)]), 2..7)]),
            1..8,
          )]),
          0..9,
        )],
      ),
      (
        "+_foo\nbar_+",
        // not sure if this is "spec", but it's what asciidoctor currently does
        nodes![node!(
          InlinePassthrough(nodes![
            node!("_foo"; 1..5),
            node!(JoiningNewline, 5..6),
            node!("bar_"; 6..10),
          ]),
          0..11,
        )],
      ),
      (
        "+_<foo>&_+",
        nodes![node!(
          InlinePassthrough(nodes![
            node!("_"; 1..2),
            node!(SpecialChar(SpecialCharKind::LessThan), 2..3),
            node!("foo"; 3..6),
            node!(SpecialChar(SpecialCharKind::GreaterThan), 6..7),
            node!(SpecialChar(SpecialCharKind::Ampersand), 7..8),
            node!("_"; 8..9),
          ]),
          0..10,
        )],
      ),
      (
        "rofl +_foo_+ lol",
        nodes![
          node!("rofl "; 0..5),
          node!(InlinePassthrough(nodes![node!("_foo_"; 6..11)]), 5..12,),
          node!(" lol"; 12..16),
        ],
      ),
      (
        "++_foo_++bar",
        nodes![
          node!(InlinePassthrough(nodes![node!("_foo_"; 2..7)]), 0..9,),
          node!("bar"; 9..12),
        ],
      ),
      (
        "+++_<foo>&_+++ bar",
        nodes![
          node!(InlinePassthrough(nodes![node!("_<foo>&_"; 3..11)]), 0..14,),
          node!(" bar"; 14..18),
        ],
      ),
      (
        "foo #bar#",
        nodes![
          node!("foo "; 0..4),
          node!(Highlight(nodes![node!("bar"; 5..8)]), 4..9),
        ],
      ),
      (
        "image::foo.png[]", // unexpected block macro, parse as text
        nodes![node!("image::foo.png[]"; 0..16)],
      ),
      (
        "foo `bar`",
        nodes![
          node!("foo "; 0..4),
          node!(Mono(nodes![node!("bar"; 5..8)]), 4..9),
        ],
      ),
      (
        "foo b``ar``",
        nodes![
          node!("foo b"; 0..5),
          node!(Mono(nodes![node!("ar"; 7..9)]), 5..11),
        ],
      ),
      (
        "foo *bar*",
        nodes![
          node!("foo "; 0..4),
          node!(Bold(nodes![node!("bar"; 5..8)]), 4..9),
        ],
      ),
      (
        "foo b**ar**",
        nodes![
          node!("foo b"; 0..5),
          node!(Bold(nodes![node!("ar"; 7..9)]), 5..11),
        ],
      ),
      (
        "foo ~bar~ baz",
        nodes![
          node!("foo "; 0..4),
          node!(Subscript(nodes![node!("bar"; 5..8)]), 4..9),
          node!(" baz"; 9..13),
        ],
      ),
      (
        "foo _bar\nbaz_",
        nodes![
          node!("foo "; 0..4),
          node!(
            Italic(nodes![
              node!("bar"; 5..8),
              node!(JoiningNewline, 8..9),
              node!("baz"; 9..12),
            ]),
            4..13,
          ),
        ],
      ),
      ("foo __bar", nodes![node!("foo __bar"; 0..9)]),
      (
        "foo _bar baz_",
        nodes![
          node!("foo "; 0..4),
          node!(Italic(nodes![node!("bar baz"; 5..12)]), 4..13),
        ],
      ),
      (
        "foo _bar_",
        nodes![
          node!("foo "; 0..4),
          node!(Italic(nodes![node!("bar"; 5..8)]), 4..9),
        ],
      ),
      (
        "foo b__ar__",
        nodes![
          node!("foo b"; 0..5),
          node!(Italic(nodes![node!("ar"; 7..9)]), 5..11),
        ],
      ),
      ("foo 'bar'", nodes![node!("foo 'bar'"; 0..9)]),
      ("foo \"bar\"", nodes![node!("foo \"bar\""; 0..9)]),
      (
        "foo `\"bar\"`",
        nodes![
          node!("foo "; 0..4),
          node!(Mono(nodes![node!("\"bar\""; 5..10)]), 4..11),
        ],
      ),
      (
        "foo `'bar'`",
        nodes![
          node!("foo "; 0..4),
          node!(Mono(nodes![node!("'bar'"; 5..10)]), 4..11),
        ],
      ),
      (
        "foo \"`bar`\"",
        nodes![
          node!("foo "; 0..4),
          node!(Quote(QuoteKind::Double, nodes![node!("bar"; 6..9)]), 4..11,),
        ],
      ),
      (
        "foo \"`bar baz`\"",
        nodes![
          node!("foo "; 0..4),
          node!(
            Quote(QuoteKind::Double, nodes![node!("bar baz"; 6..13)]),
            4..15,
          ),
        ],
      ),
      (
        "foo \"`bar\nbaz`\"",
        nodes![
          node!("foo "; 0..4),
          node!(
            Quote(
              QuoteKind::Double,
              nodes![
                node!("bar"; 6..9),
                node!(JoiningNewline, 9..10),
                node!("baz"; 10..13),
              ],
            ),
            4..15,
          ),
        ],
      ),
      (
        "foo '`bar`'",
        nodes![
          node!("foo "; 0..4),
          node!(Quote(QuoteKind::Single, nodes![node!("bar"; 6..9)]), 4..11,),
        ],
      ),
      (
        "Olaf's wrench",
        nodes![
          node!("Olaf"; 0..4),
          node!(CurlyQuote(LegacyImplicitApostrophe), 4..5),
          node!("s wrench"; 5..13),
        ],
      ),
      (
        "foo   bar",
        nodes![
          node!("foo"; 0..3),
          node!(MultiCharWhitespace(bstr!("   ")), 3..6),
          node!("bar"; 6..9),
        ],
      ),
      (
        "`+{name}+`",
        nodes![node!(LitMono(src!("{name}", 2..8)), 0..10)],
      ),
      (
        "`+_foo_+`",
        nodes![node!(LitMono(src!("_foo_", 2..7)), 0..9)],
      ),
      (
        "foo <bar> & lol",
        nodes![
          node!("foo "; 0..4),
          node!(SpecialChar(SpecialCharKind::LessThan), 4..5),
          node!("bar"; 5..8),
          node!(SpecialChar(SpecialCharKind::GreaterThan), 8..9),
          node!(" "; 9..10),
          node!(SpecialChar(SpecialCharKind::Ampersand), 10..11),
          node!(" lol"; 11..15),
        ],
      ),
      (
        "^bar^",
        nodes![node!(Superscript(nodes![node!("bar"; 1..4)]), 0..5)],
      ),
      (
        "^bar^",
        nodes![node!(Superscript(nodes![node!("bar"; 1..4)]), 0..5)],
      ),
      ("foo ^bar", nodes![node!("foo ^bar"; 0..8)]),
      ("foo bar^", nodes![node!("foo bar^"; 0..8)]),
      (
        "foo ^bar^ foo",
        nodes![
          node!("foo "; 0..4),
          node!(Superscript(nodes![node!("bar"; 5..8)]), 4..9),
          node!(" foo"; 9..13),
        ],
      ),
      (
        "doublefootnote:[ymmv _i_]bar",
        nodes![
          node!("double"; 0..6),
          node!(
            Macro(Footnote {
              id: None,
              text: nodes![
                node!("ymmv "; 16..21),
                node!(Italic(nodes![node!("i"; 22..23)]), 21..24),
              ],
            }),
            6..25,
          ),
          node!("bar"; 25..28),
        ],
      ),
    ];

    // repeated passes necessary?
    // yikes: `link:pass:[My Documents/report.pdf][Get Report]`

    run(cases);
  }

  #[test]
  fn test_button_menu_macro() {
    let cases = vec![
      (
        "press the btn:[OK] button",
        nodes![
          node!("press the "; 0..10),
          node!(Macro(Button(src!("OK", 15..17))), 10..18),
          node!(" button"; 18..25),
        ],
      ),
      (
        "btn:[Open]",
        nodes![node!(Macro(Button(src!("Open", 5..9))), 0..10)],
      ),
      (
        "select menu:File[Save].",
        nodes![
          node!("select "; 0..7),
          node!(
            Macro(Menu(vecb![src!("File", 12..16), src!("Save", 17..21)])),
            7..22,
          ),
          node!("."; 22..23),
        ],
      ),
      (
        "menu:View[Zoom > Reset]",
        nodes![node!(
          Macro(Menu(vecb![
            src!("View", 5..9),
            src!("Zoom", 10..14),
            src!("Reset", 17..22),
          ])),
          0..23,
        )],
      ),
    ];
    run(cases);
  }

  fn run(cases: Vec<(&str, InlineNodes)>) {
    for (input, expected) in cases {
      let mut parser = Parser::new(leaked_bump(), input);
      let mut block = parser.read_lines().unwrap();
      let inlines = parser.parse_inlines(&mut block).unwrap();
      assert_eq!(inlines, expected, from: input);
    }
  }
}
