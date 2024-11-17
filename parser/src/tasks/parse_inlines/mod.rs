use regex::Regex;

mod inline_preproc;
mod inline_utils;

use crate::internal::*;
use crate::tasks::parse_attr_list::AnchorSrc;
use crate::variants::token::*;
use ast::variants::{inline::*, r#macro::*};
use inline_utils::*;

impl<'arena> Parser<'arena> {
  pub(crate) fn parse_inlines(
    &mut self,
    lines: &mut ContiguousLines<'arena>,
  ) -> Result<InlineNodes<'arena>> {
    self.parse_inlines_until(lines, &[])
  }

  pub(crate) fn parse_inlines_until(
    &mut self,
    lines: &mut ContiguousLines<'arena>,
    stop_tokens: &[TokenSpec],
  ) -> Result<InlineNodes<'arena>> {
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

        if line.may_contain_inline_pass() {
          self.replace_inline_pass(&mut line, lines)?;
          break;
        }

        let Some(token) = line.consume_current() else {
          acc.maybe_push_joining_newline(lines);
          break;
        };

        if token.loc.include_depth != acc.text.loc.include_depth {
          acc.commit();
          acc.text.loc = token.loc.clamp_start()
        }

        match token.kind {
          UriScheme if subs.macros() && line.continues_inline_macro() => {
            self.parse_uri_scheme_macro(&token, &mut line, &mut acc)?
          }

          MacroName if subs.macros() && line.continues_inline_macro() => {
            let mut macro_loc = token.loc;
            let line_end = line.last_location().unwrap();
            acc.commit();
            match token.lexeme.as_str() {
              "image:" => {
                let target = line.consume_macro_target(self.bump);
                let attrs = self.parse_inline_attr_list(&mut line)?;
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
                  keys.push(self.string(key));
                }
                acc.push_node(Macro(Keyboard { keys, keys_src }), macro_loc);
              }
              "footnote:" => {
                let id = line.consume_optional_macro_target(self.bump);
                lines.restore_if_nonempty(line);
                let note = self.parse_inlines_until(lines, &[Kind(CloseBracket)])?;
                extend(&mut macro_loc, &note, 1);
                let number = {
                  let mut num_footnotes = self.ctx.num_footnotes.borrow_mut();
                  *num_footnotes += 1;
                  *num_footnotes
                };
                acc.push_node(Macro(Footnote { number, id, text: note }), macro_loc);
                break;
              }
              "xref:" => {
                let mut id = line.consume_macro_target(self.bump);
                if id.src.starts_with('#') {
                  id.drop_first();
                }
                self.ctx.xrefs.borrow_mut().insert(id.src.clone(), id.loc);
                lines.restore_if_nonempty(line);
                let nodes = self.parse_inlines_until(lines, &[Kind(CloseBracket)])?;
                let linktext = if nodes.is_empty() {
                  macro_loc.end = id.loc.end + 2;
                  None
                } else {
                  extend(&mut macro_loc, &nodes, 1);
                  Some(nodes)
                };
                acc.push_node(Macro(Xref { id, linktext }), macro_loc);
                break;
              }
              "link:" => {
                let target = self
                  .macro_target_from_passthru(&mut line)
                  .unwrap_or_else(|| line.consume_macro_target(self.bump));
                let line_has_caret = line.contains(Caret);
                let mut attrs = self.parse_link_macro_attr_list(&mut line)?;
                let mut caret = false;
                if line_has_caret {
                  caret = link_macro_blank_window_shorthand(&mut attrs);
                }
                finish_macro(&line, &mut macro_loc, line_end, &mut acc.text);
                let scheme = token.to_url_scheme();
                acc.push_node(
                  Macro(Link {
                    scheme,
                    target,
                    attrs: Some(attrs),
                    caret,
                  }),
                  macro_loc,
                );
              }
              "icon:" => {
                let target = line.consume_macro_target(self.bump);
                let attrs = self.parse_inline_attr_list(&mut line)?;
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

                let mut pos = rest.loc.start as usize;
                rest.split('>').for_each(|substr| {
                  let mut trimmed = substr.trim_start();
                  pos += substr.len() - trimmed.len();
                  trimmed = trimmed.trim_end();
                  if !trimmed.is_empty() {
                    items.push(SourceString::new(
                      self.string(trimmed),
                      SourceLocation::new(pos as u32, (pos + trimmed.len()) as u32),
                    ));
                  }
                  pos += substr.len() + 1;
                });
                line.discard_assert(CloseBracket);
                finish_macro(&line, &mut macro_loc, line_end, &mut acc.text);
                acc.push_node(Macro(Menu(items)), macro_loc);
              }
              "anchor:" => {
                let id = line.consume_macro_target(self.bump);
                let mut attrs = self.parse_inline_attr_list(&mut line)?;
                self.document.anchors.borrow_mut().insert(
                  id.src.clone(),
                  Anchor {
                    reftext: attrs.take_positional(0),
                    title: InlineNodes::new(self.bump),
                  },
                );
                acc.push_node(InlineAnchor(id.src), id.loc);
              }
              _ => todo!("unhandled macro type: `{}`", token.lexeme),
            }
          }

          AttrRef if !subs.attr_refs() => {
            // turns out we didn't need to resolve the attr ref
            // probably because we're inside a pass macro, so
            // discard the resolved tokens and push the raw attr
            while let Some(peek) = line.current_token() {
              if peek.loc == token.loc {
                line.discard(1);
              } else {
                break;
              }
            }
            acc.text.push_token(&token);
          }

          // if we're in a table cell, and we have a blank attr ref
          // we need to preserve a node for the crazy table cell paragraph logic
          AttrRef
            if self.ctx.table_cell_ctx != TableCellContext::None
              // we know it was blank if there's no inserted token w/ same loc
              && line.current_token().map_or(false, |t| t.loc != token.loc) =>
          {
            acc.push_node(Discarded, token.loc)
          }

          TermDelimiter
            if subs.callouts()
              && token.len() == 2
              && line.current_is(Whitespace)
              && token.lexeme == ";;" // this is a happy accident
              && line.continues_valid_callout_nums() =>
          {
            push_callout_tuck(token, &mut line, &mut acc);
          }

          Hash
            if subs.callouts()
              && line.current_is(Whitespace)
              && line.continues_valid_callout_nums() =>
          {
            push_callout_tuck(token, &mut line, &mut acc);
          }

          ForwardSlashes
            if subs.callouts()
              && token.len() == 2
              && line.current_is(Whitespace)
              && line.continues_valid_callout_nums() =>
          {
            push_callout_tuck(token, &mut line, &mut acc);
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
            if line.current_is(Hash) {
              line.discard(1);
            }
            let mut inner = line.extract_line_before(&[Kind(GreaterThan), Kind(GreaterThan)]);
            let id = inner.consume_to_string_until(Comma, self.bump);
            self.ctx.xrefs.borrow_mut().insert(id.src.clone(), id.loc);
            let mut linktext = None;
            if !inner.is_empty() {
              inner.discard_assert(Comma);
              if !inner.is_empty() {
                linktext = Some(self.parse_inlines(&mut inner.into_lines())?);
              }
            }
            line.discard_assert(GreaterThan);
            loc.end = line.consume_current().unwrap().loc.end;
            acc.push_node(Macro(Xref { id, linktext }), loc);
          }

          LessThan
            if subs.macros()
              && line.current_token().is(UriScheme)
              && line.no_whitespace_until(GreaterThan) =>
          {
            acc.push_node(Discarded, token.loc);
            let scheme_token = line.consume_current().unwrap();
            let mut loc = scheme_token.loc;
            let line_end = line.last_location().unwrap();
            let target = line.consume_url(Some(&scheme_token), Some(GreaterThan), self.bump);
            if target.src == scheme_token.lexeme {
              // turns out we don't have a valid uri here, backtrack
              acc.pop_node();
              if subs.special_chars() {
                acc.push_node(SpecialChar(SpecialCharKind::LessThan), token.loc);
              } else {
                acc.push_node(Text(token.lexeme), token.loc);
              }
              acc.text.push_token(&scheme_token);
            } else {
              finish_macro(&line, &mut loc, line_end, &mut acc.text);
              let scheme = Some(scheme_token.to_url_scheme().unwrap());
              acc.push_node(
                Macro(Link {
                  scheme,
                  target,
                  attrs: None,
                  caret: false,
                }),
                loc,
              );
              acc.push_node(Discarded, line.consume_current().unwrap().loc);
            }
          }

          MaybeEmail if subs.macros() && EMAIL_RE.is_match(&token.lexeme) => {
            let loc = token.loc;
            acc.push_node(
              Macro(Link {
                scheme: Some(UrlScheme::Mailto),
                target: token.into_source_string(),
                attrs: None,
                caret: false,
              }),
              loc,
            );
          }

          Underscore
            if subs.inline_formatting()
              && starts_constrained(&[Kind(Underscore)], &token, &line, lines) =>
          {
            self.parse_node(Italic, [Kind(Underscore)], &token, &mut acc, line, lines)?;
            break;
          }

          Underscore
            if subs.inline_formatting()
              && starts_unconstrained(&[Kind(Underscore); 2], &token, &line, lines) =>
          {
            self.parse_node(Italic, [Kind(Underscore); 2], &token, &mut acc, line, lines)?;

            break;
          }

          Star
            if subs.inline_formatting()
              && starts_constrained(&[Kind(Star)], &token, &line, lines) =>
          {
            self.parse_node(Bold, [Kind(Star)], &token, &mut acc, line, lines)?;
            break;
          }

          Star
            if subs.inline_formatting()
              && starts_unconstrained(&[Kind(Star)], &token, &line, lines) =>
          {
            self.parse_node(Bold, [Kind(Star); 2], &token, &mut acc, line, lines)?;
            break;
          }

          OpenBracket
            if subs.inline_formatting() && line.contains_seq(&[Kind(CloseBracket), Kind(Hash)]) =>
          {
            let mut parse_token = token.clone();
            let attr_list = self.parse_formatted_text_attr_list(&mut line)?;
            debug_assert!(line.current_is(Hash));
            line.discard_assert(Hash);
            parse_token.kind = Hash;
            let span = |inner| TextSpan(attr_list, inner);
            self.parse_node(span, [Kind(Hash)], &parse_token, &mut acc, line, lines)?;
            if let Some(InlineNode { content: TextSpan(attrs, nodes), .. }) = acc.inlines.last() {
              if let Some(id) = &attrs.id {
                self.document.anchors.borrow_mut().insert(
                  id.src.clone(),
                  Anchor { reftext: None, title: nodes.clone() },
                );
              }
            }
            break;
          }

          OpenBracket
            if line.current_is(OpenBracket)
              && !line.peek_token().is(CloseBracket)
              && line.contains_seq(&[Kind(CloseBracket), Kind(CloseBracket)]) =>
          {
            line.discard(1); // second `[`
            if let Some(AnchorSrc { id, reftext, loc }) = self.parse_inline_anchor(&mut line)? {
              self.document.anchors.borrow_mut().insert(
                id.src.clone(),
                Anchor {
                  reftext,
                  title: InlineNodes::new(self.bump),
                },
              );
              acc.push_node(InlineAnchor(id.src), loc);
            } else {
              acc.text.push_token(&token);
            }
          }

          Backtick
            if subs.inline_formatting()
              && line.current_is(Plus)
              && contains_seq(&[Len(1, Plus), Kind(Backtick)], &line, lines) =>
          {
            self.ctx.subs.remove(Subs::InlineFormatting);
            self.ctx.subs.remove(Subs::AttrRefs);
            self.parse_node(
              |mut inner| {
                assert!(inner.len() == 1, "invalid lit mono");
                match inner.pop().unwrap() {
                  InlineNode { content: Text(lit), loc } => LitMono(SourceString::new(lit, loc)),
                  _ => panic!("invalid lit mono"),
                }
              },
              [Len(1, Plus), Kind(Backtick)],
              &token,
              &mut acc,
              line,
              lines,
            )?;
            self.ctx.subs = subs;
            break;
          }

          Caret if subs.inline_formatting() && line.no_whitespace_until(Caret) => {
            self.parse_node(Superscript, [Kind(Caret)], &token, &mut acc, line, lines)?;
            break;
          }

          Backtick
            if subs.inline_formatting()
              && starts_constrained(&[Kind(Backtick)], &token, &line, lines) =>
          {
            self.parse_node(Mono, [Kind(Backtick)], &token, &mut acc, line, lines)?;
            break;
          }

          Backtick
            if subs.inline_formatting()
              && starts_unconstrained(&[Kind(Backtick)], &token, &line, lines) =>
          {
            self.parse_node(Mono, [Kind(Backtick); 2], &token, &mut acc, line, lines)?;
            break;
          }

          DoubleQuote
            if subs.inline_formatting()
              && line.current_is(Backtick)
              && starts_constrained(&[Kind(Backtick), Kind(DoubleQuote)], &token, &line, lines) =>
          {
            self.parse_node(
              |inner| Quote(Double, inner),
              [Kind(Backtick), Kind(DoubleQuote)],
              &token,
              &mut acc,
              line,
              lines,
            )?;
            break;
          }

          SingleQuote
            if subs.inline_formatting()
              && line.current_is(Backtick)
              && starts_constrained(&[Kind(Backtick), Kind(SingleQuote)], &token, &line, lines) =>
          {
            self.parse_node(
              |inner| Quote(Single, inner),
              [Kind(Backtick), Kind(SingleQuote)],
              &token,
              &mut acc,
              line,
              lines,
            )?;
            break;
          }

          Tilde if subs.inline_formatting() && line.no_whitespace_until(Tilde) => {
            self.parse_node(Subscript, [Kind(Tilde)], &token, &mut acc, line, lines)?;
            break;
          }

          Backtick if subs.inline_formatting() && line.current_is(DoubleQuote) => {
            push_simple(CurlyQuote(RightDouble), &token, line, &mut acc, lines);
            break;
          }

          DoubleQuote
            if subs.inline_formatting()
              && line.current_is(Backtick)
              && stop_tokens != [Kind(Backtick)] =>
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
              && stop_tokens != [Kind(Backtick)] =>
          {
            push_simple(CurlyQuote(LeftSingle), &token, line, &mut acc, lines);
            break;
          }

          Hash
            if subs.inline_formatting()
              && starts_unconstrained(&[Kind(Hash); 2], &token, &line, lines) =>
          {
            self.parse_node(Highlight, [Kind(Hash); 2], &token, &mut acc, line, lines)?;
            break;
          }

          Hash if subs.inline_formatting() && contains_seq(&[Kind(Hash)], &line, lines) => {
            self.parse_node(Highlight, [Kind(Hash)], &token, &mut acc, line, lines)?;
            break;
          }

          Plus if token.len() == 3 && contains_len(Plus, 3, &line, lines) => {
            self.ctx.subs = Substitutions::none();
            self.parse_node(
              InlinePassthru,
              [TokenSpec::Len(3, Plus)],
              &token,
              &mut acc,
              line,
              lines,
            )?;
            self.ctx.subs = subs;
            break;
          }

          Plus
            if subs.inline_formatting()
              && token.len() == 2
              && starts_unconstrained(&[Len(2, Plus)], &token, &line, lines) =>
          {
            self.ctx.subs.remove(Subs::InlineFormatting);
            self.parse_node(
              InlinePassthru,
              [Len(2, Plus)],
              &token,
              &mut acc,
              line,
              lines,
            )?;
            self.ctx.subs = subs;
            break;
          }

          Plus
            if subs.inline_formatting()
              && starts_constrained(&[Len(1, Plus)], &token, &line, lines) =>
          {
            self.ctx.subs.remove(Subs::InlineFormatting);
            self.parse_node(
              InlinePassthru,
              [Len(1, Plus)],
              &token,
              &mut acc,
              line,
              lines,
            )?;
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
            acc.push_node(MultiCharWhitespace(token.lexeme), token.loc);
          }

          Whitespace if line.current_is(Plus) && line.num_tokens() == 1 => {
            let mut loc = token.loc;
            line.discard_assert(Plus);
            loc.end += 2; // plus and newline
            acc.push_node(LineBreak, loc);
            break;
          }

          TokenKind::Newline => acc.push_node(Inline::Newline, token.loc),

          Discard | AttrRef => acc.text.loc = token.loc.clamp_end(),

          Backslash if !line.is_empty() => {
            acc.push_node(Discarded, token.loc);
            // pushing the next token as text prevents recognizing the pattern
            let next_token = line.consume_current().unwrap();
            acc.text.push_token(&next_token);
          }

          _ if subs.macros() && token.is(UriScheme) => {
            let mut loc = token.loc;
            let line_end = line.last_location().unwrap();
            let target = line.consume_url(Some(&token), None, self.bump);
            if target.src == token.lexeme {
              acc.text.push_token(&token);
            } else {
              finish_macro(&line, &mut loc, line_end, &mut acc.text);
              let scheme = Some(token.to_url_scheme().unwrap());
              acc.push_node(
                Macro(Link {
                  scheme,
                  target,
                  attrs: None,
                  caret: false,
                }),
                loc,
              );
            }
          }

          PreprocPassthru => {
            let index: usize = token.lexeme[1..6].parse().unwrap();
            let content = self.ctx.passthrus[index].take().unwrap();
            acc.push_node(InlinePassthru(content), token.loc);
          }

          _ => {
            if acc.text.loc.end == token.loc.start {
              acc.text.push_token(&token);
            } else {
              // happens when ifdefs cause lines to be skipped
              acc.text.commit_inlines(&mut acc.inlines);
              acc.text.push_token(&token);
              acc.text.loc = token.loc;
            }
          }
        }
      }
    }

    acc.commit();
    Ok(acc.inlines)
  }

  fn parse_uri_scheme_macro(
    &mut self,
    token: &Token<'arena>,
    line: &mut Line<'arena>,
    acc: &mut Accum<'arena>,
  ) -> Result<()> {
    let mut macro_loc = token.loc;
    let line_end = line.last_location().unwrap();
    acc.commit();
    let target = line.consume_url(Some(token), Some(OpenBracket), self.bump);
    line.discard_assert(OpenBracket);
    let line_has_caret = line.contains(Caret);
    let mut attrs = self.parse_link_macro_attr_list(line)?;
    let mut caret = false;
    if line_has_caret {
      caret = link_macro_blank_window_shorthand(&mut attrs);
    }
    finish_macro(line, &mut macro_loc, line_end, &mut acc.text);
    let scheme = Some(token.to_url_scheme().unwrap());
    acc.push_node(
      Macro(Link {
        scheme,
        target,
        attrs: Some(attrs),
        caret,
      }),
      macro_loc,
    );
    Ok(())
  }

  fn parse_node<const N: usize>(
    &mut self,
    wrap: impl FnOnce(InlineNodes<'arena>) -> Inline<'arena>,
    stop_tokens: [TokenSpec; N],
    start: &Token<'arena>,
    state: &mut Accum<'arena>,
    mut line: Line<'arena>,
    lines: &mut ContiguousLines<'arena>,
  ) -> Result<()> {
    let mut loc = start.loc;
    let mut stop_len = start.len();
    stop_tokens.iter().take(N - 1).for_each(|spec| {
      let tok = line.consume_current().unwrap();
      debug_assert!(tok.kind == spec.token_kind());
      stop_len += tok.len();
    });
    lines.restore_if_nonempty(line);
    let inner = self.parse_inlines_until(lines, &stop_tokens)?;
    extend(&mut loc, &inner, stop_len);
    state.push_node(wrap(inner), loc);
    push_newline_if_needed(state, lines);
    Ok(())
  }

  fn should_stop_at(&self, line: &Line<'arena>) -> bool {
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

  fn recover_custom_line_comment(&mut self, state: &mut Accum<'arena>) {
    let Some(ref comment_bytes) = self.ctx.custom_line_comment else {
      return;
    };
    let mut line_txt = state.text.str().as_bytes();
    let line_len = line_txt.len();
    let mut back = comment_bytes.len() as u32;
    if line_txt.ends_with(&[b' ']) {
      back += 1;
      line_txt = &line_txt[..line_len - 1];
    }
    if line_txt.ends_with(comment_bytes) {
      let tuck = self.string(state.text.str().split_at(line_len - back as usize).1);
      state.text.drop_last(back);
      let tuck_loc = SourceLocation::new(state.text.loc.end, state.text.loc.end + back);
      state.push_node(CalloutTuck(tuck), tuck_loc);
    }
  }
}

fn link_macro_blank_window_shorthand(attr_list: &mut AttrList) -> bool {
  let Some(mut nodes) = attr_list.take_positional(0) else {
    return false;
  };
  let Some(node) = nodes.last_mut() else {
    attr_list.positional[0] = Some(nodes);
    return false;
  };
  if let Inline::Text(text) = &node.content {
    if text.ends_with('^') {
      let mut shortened = text.clone();
      shortened.pop();
      node.content = Inline::Text(shortened);
      node.loc.end -= 1;
      attr_list.positional[0] = Some(nodes);
      return true;
    }
  };
  attr_list.positional[0] = Some(nodes);
  false
}

fn push_callout_tuck<'arena>(
  token: Token<'arena>,
  line: &mut Line<'arena>,
  state: &mut Accum<'arena>,
) {
  let mut tuck = token.into_source_string();
  line.discard_assert(Whitespace);
  tuck.src.push(' ');
  tuck.loc.end += 1;
  state.push_node(CalloutTuck(tuck.src), tuck.loc);
}

fn push_newline_if_needed<'arena>(state: &mut Accum<'arena>, lines: &ContiguousLines<'arena>) {
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
    state.push_node(Inline::Newline, state.text.loc);
  }
}

fn push_simple<'arena>(
  inline_node: Inline<'arena>,
  token: &Token<'arena>,
  mut line: Line<'arena>,
  state: &mut Accum<'arena>,
  lines: &mut ContiguousLines<'arena>,
) {
  let mut loc = token.loc;
  line.discard(1);
  loc.end += 1;
  lines.restore_if_nonempty(line);
  state.push_node(inline_node, loc);
  push_newline_if_needed(state, lines);
}

#[derive(Debug)]
struct Accum<'arena> {
  inlines: InlineNodes<'arena>,
  text: CollectText<'arena>,
}

impl<'arena> Accum<'arena> {
  fn commit(&mut self) {
    self.text.commit_inlines(&mut self.inlines);
  }

  fn push_node(&mut self, node: Inline<'arena>, loc: SourceLocation) {
    self.commit();
    self.inlines.push(InlineNode::new(node, loc));
    self.text.loc = loc.clamp_end();
  }

  fn pop_node(&mut self) {
    self.inlines.pop();
  }

  fn maybe_push_joining_newline(&mut self, lines: &ContiguousLines<'arena>) {
    if !lines.is_empty() {
      self.commit();
      self.text.loc.end += 1;
      self.push_node(Inline::Newline, self.text.loc);
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use test_utils::*;

  #[test]
  fn test_inline_passthrus() {
    let cases = vec![
      (
        "+_foo_&+ bar",
        nodes![
          node!(
            InlinePassthru(nodes![
              node!("_foo_"; 1..6),
              node!(SpecialChar(SpecialCharKind::Ampersand), 6..7),
            ]),
            0..8,
          ),
          node!(" bar"; 8..12),
        ],
      ),
      (
        "baz ++_foo_&++ bar",
        nodes![
          node!("baz "; 0..4),
          node!(
            InlinePassthru(nodes![
              node!("_foo_"; 6..11),
              node!(SpecialChar(SpecialCharKind::Ampersand), 11..12),
            ]),
            4..14,
          ),
          node!(" bar"; 14..18),
        ],
      ),
      (
        "baz +++_foo_&+++ bar", // no specialchars subs on +++
        nodes![
          node!("baz "; 0..4),
          node!(InlinePassthru(nodes![node!("_foo_&"; 7..13)]), 4..16,),
          node!(" bar"; 16..20),
        ],
      ),
      (
        "+foo+ bar +baz+", // two passthrus on one line
        nodes![
          node!(InlinePassthru(just!("foo", 1..4)), 0..5),
          node!(" bar "; 5..10),
          node!(InlinePassthru(just!("baz", 11..14)), 10..15),
        ],
      ),
      (
        "+foo+bar", // single plus = not unconstrained, not a passthrough
        just!("+foo+bar", 0..8),
      ),
      (
        "+foo\nbar+ baz", // multi-line
        nodes![
          node!(
            InlinePassthru(nodes![
              node!("foo"; 1..4),
              node!(Inline::Newline, 4..5),
              node!("bar"; 5..8),
            ]),
            0..9
          ),
          node!(" baz"; 9..13),
        ],
      ),
      (
        "+foo\nbar+baz", // multi-line constrained can't terminate within word
        nodes![
          // no InlinePassthrough
          node!("+foo"; 0..4),
          node!(Inline::Newline, 4..5),
          node!("bar+baz"; 5..12),
        ],
      ),
      (
        "++foo\nbar++", // multi-line unconstrained
        nodes![node!(
          InlinePassthru(nodes![
            node!("foo"; 2..5),
            node!(Inline::Newline, 5..6),
            node!("bar"; 6..9),
          ]),
          0..11
        )],
      ),
      (
        "pass:[_foo_]",
        nodes![node!(InlinePassthru(just!("_foo_", 6..11)), 0..12)],
      ),
      (
        "pass:q[_foo_] bar", // subs=quotes
        nodes![
          node!(
            InlinePassthru(nodes![node!(Italic(just!("foo", 8..11)), 7..12)]),
            0..13
          ),
          node!(" bar"; 13..17),
        ],
      ),
      (
        "pass:a,c[_foo_\nbar]",
        nodes![node!(
          InlinePassthru(nodes![
            node!("_foo_"; 9..14),
            node!(Inline::Newline, 14..15),
            node!("bar"; 15..18),
          ]),
          0..19
        )],
      ),
    ];

    run(cases);
  }

  #[test]
  fn test_line_comments() {
    let cases = vec![(
      "foo\n// baz\nbar",
      nodes![
        node!("foo"; 0..3),
        node!(Inline::Newline, 3..4),
        node!(LineComment(bstr!(" baz")), 4..11),
        node!("bar"; 11..14),
      ],
    )];
    run(cases);
  }

  #[test]
  fn test_joining_newlines() {
    let cases = vec![
      ("{foo}", just!("{foo}", 0..5)),
      (
        "\\{foo}",
        nodes![node!(Discarded, 0..1), node!("{foo}"; 1..6)],
      ),
      ("{attribute-missing}", just!("skip", 0..19)),
      (
        "\\{attribute-missing}",
        nodes![node!(Discarded, 0..1), node!("{attribute-missing}"; 1..20)],
      ),
      (
        "_foo_\nbar",
        nodes![
          node!(Italic(nodes![node!("foo"; 1..4)]), 0..5),
          node!(Inline::Newline, 5..6),
          node!("bar"; 6..9),
        ],
      ),
      (
        "__foo__\nbar",
        nodes![
          node!(Italic(nodes![node!("foo"; 2..5)]), 0..7),
          node!(Inline::Newline, 7..8),
          node!("bar"; 8..11),
        ],
      ),
      (
        "foo \"`bar`\"\nbaz",
        nodes![
          node!("foo "; 0..4),
          node!(Quote(QuoteKind::Double, nodes![node!("bar"; 6..9)]), 4..11),
          node!(Inline::Newline, 11..12),
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
                node!(Inline::Newline, 5..6),
                node!("bar"; 6..9),
              ],
            ),
            0..11,
          ),
          node!(Inline::Newline, 11..12),
          node!("baz"; 12..15),
        ],
      ),
      (
        "bar`\"\nbaz",
        nodes![
          node!("bar"; 0..3),
          node!(CurlyQuote(RightDouble), 3..5),
          node!(Inline::Newline, 5..6),
          node!("baz"; 6..9),
        ],
      ),
      (
        "^foo^\nbar",
        nodes![
          node!(Superscript(nodes![node!("foo"; 1..4)]), 0..5),
          node!(Inline::Newline, 5..6),
          node!("bar"; 6..9),
        ],
      ),
      (
        "~foo~\nbar",
        nodes![
          node!(Subscript(nodes![node!("foo"; 1..4)]), 0..5),
          node!(Inline::Newline, 5..6),
          node!("bar"; 6..9),
        ],
      ),
      (
        "`+{name}+`\nbar",
        nodes![
          node!(LitMono(src!("{name}", 2..8)), 0..10),
          node!(Inline::Newline, 10..11),
          node!("bar"; 11..14),
        ],
      ),
      (
        "+_foo_+\nbar",
        nodes![
          node!(InlinePassthru(nodes![node!("_foo_"; 1..6)]), 0..7,),
          node!(Inline::Newline, 7..8),
          node!("bar"; 8..11),
        ],
      ),
      (
        "+++_<foo>&_+++\nbar",
        nodes![
          node!(InlinePassthru(nodes![node!("_<foo>&_"; 3..11)]), 0..14,),
          node!(Inline::Newline, 14..15),
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
          node!(Inline::Newline, 4..5),
          node!("bar"; 5..8),
        ],
      ),
    ];

    run(cases);
  }

  #[test]
  fn test_inline_anchors() {
    let cases = vec![
      (
        "[[foo]]bar",
        nodes![node!(InlineAnchor(bstr!("foo")), 0..7), node!("bar"; 7..10),],
      ),
      (
        "bar[[foo]]",
        nodes![node!("bar"; 0..3), node!(InlineAnchor(bstr!("foo")), 3..10),],
      ),
    ];

    run(cases);
  }

  #[test]
  fn test_parse_inlines() {
    let cases = vec![
      (
        "+_foo_+",
        nodes![node!(InlinePassthru(nodes![node!("_foo_"; 1..6)]), 0..7,)],
      ),
      (
        "+_{foo}_+",
        nodes![node!(InlinePassthru(nodes![node!("_{foo}_"; 1..8)]), 0..9,)],
      ),
      (
        "+_{attribute-missing}_+",
        nodes![node!(
          InlinePassthru(nodes![node!("_{attribute-missing}_"; 1..22)]),
          0..23,
        )],
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
          InlinePassthru(nodes![
            node!("_foo"; 1..5),
            node!(Inline::Newline, 5..6),
            node!("bar_"; 6..10),
          ]),
          0..11,
        )],
      ),
      (
        "+_<foo>&_+",
        nodes![node!(
          InlinePassthru(nodes![
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
          node!(InlinePassthru(nodes![node!("_foo_"; 6..11)]), 5..12,),
          node!(" lol"; 12..16),
        ],
      ),
      (
        "++_foo_++bar",
        nodes![
          node!(InlinePassthru(nodes![node!("_foo_"; 2..7)]), 0..9,),
          node!("bar"; 9..12),
        ],
      ),
      (
        "+++_<foo>&_+++ bar",
        nodes![
          node!(InlinePassthru(nodes![node!("_<foo>&_"; 3..11)]), 0..14,),
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
        "foo ##bar##baz",
        nodes![
          node!("foo "; 0..4),
          node!(Highlight(nodes![node!("bar"; 6..9)]), 4..11),
          node!("baz"; 11..14),
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
              node!(Inline::Newline, 8..9),
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
                node!(Inline::Newline, 9..10),
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
              number: 1,
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
      let mut parser = test_parser!(input);
      let mut block = parser.read_lines().unwrap().unwrap();
      let inlines = parser.parse_inlines(&mut block).unwrap();
      expect_eq!(inlines, expected, from: input);
    }
  }
}
