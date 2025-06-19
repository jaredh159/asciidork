mod inline_preproc;
mod inline_utils;

use crate::internal::*;
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

    let span_loc = lines.first_loc().unwrap().clamp_start();
    let text = CollectText::new_in(span_loc, self.bump);
    let subs = self.ctx.subs;
    let mut acc = Accum::new(inlines, text);

    while let Some(mut line) = lines.consume_current() {
      if self.should_stop_at(&line) {
        acc.inlines.remove_trailing_newline();
        lines.restore_if_nonempty(line);
        return Ok(acc.trimmed_inlines());
      }

      if line.is_comment() && !subs.callouts() {
        if lines.is_empty() {
          acc.inlines.discard_trailing_newline();
        }
        let token = line.consume_current().unwrap();
        if line.is_empty() {
          acc.push_node(LineComment(BumpString::new_in(self.bump)), token.loc);
        } else {
          let comment = line.consume_to_string(self.bump);
          let loc = SourceLocation::new(
            token.loc.start,
            comment.loc.end + 1,
            token.loc.include_depth,
          );
          acc.push_node(LineComment(comment.src), loc);
        }
        continue;
      }

      loop {
        if line.starts_with_seq(stop_tokens) {
          line.discard(stop_tokens.len());
          acc.commit();
          lines.restore_if_nonempty(line);
          return Ok(acc.trimmed_inlines());
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
          OpenParens
            if subs.char_replacement()
              && token.is_len(1)
              && line.starts_with_seq(&[Kind(Word), Len(1, CloseParens)]) =>
          {
            match line.current_token().unwrap().lexeme.as_ref() {
              "C" | "TM" | "R" => {
                let symbol = line.consume_current().unwrap();
                line.discard_assert(CloseParens);
                acc.push_node(
                  Symbol(match symbol.lexeme.as_ref() {
                    "C" => SymbolKind::Copyright,
                    "TM" => SymbolKind::Trademark,
                    _ => SymbolKind::Registered,
                  }),
                  symbol.loc.decr_start().incr_end(),
                );
              }
              _ => acc.push_text_token(&token),
            }
          }
          Dashes if subs.char_replacement() && token.is_len(2) => {
            acc.push_emdash(token, line.current_token_mut());
          }
          Dots if subs.char_replacement() && token.is_len(3) => {
            acc.push_node(Symbol(SymbolKind::Ellipsis), token.loc);
          }
          Dashes if token.is_len(1) && subs.char_replacement() && line.current_is(GreaterThan) => {
            line.discard(1);
            acc.push_node(Symbol(SymbolKind::SingleRightArrow), token.loc.incr_end());
          }
          EqualSigns
            if token.is_len(1) && subs.char_replacement() && line.current_is(GreaterThan) =>
          {
            line.discard(1);
            acc.push_node(Symbol(SymbolKind::DoubleRightArrow), token.loc.incr_end());
          }
          LessThan if subs.char_replacement() && line.current_satisfies(Len(1, Dashes)) => {
            line.discard(1);
            acc.push_node(Symbol(SymbolKind::SingleLeftArrow), token.loc.incr_end());
          }
          LessThan if subs.char_replacement() && line.current_satisfies(Len(1, EqualSigns)) => {
            line.discard(1);
            acc.push_node(Symbol(SymbolKind::DoubleLeftArrow), token.loc.incr_end());
          }
          MacroName if subs.macros() && line.continues_inline_macro(&token) => {
            let mut macro_loc = token.loc;
            let line_end = line.last_loc().unwrap();
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
              "kbd:" if line.current_is(OpenBracket) => {
                line.discard_assert(OpenBracket);
                let keys_src = line.consume_to_string_until(CloseBracket, self.bump);
                line.discard_assert(CloseBracket);
                macro_loc.end = keys_src.loc.end + 1;
                let mut keys = BumpVec::new_in(self.bump);
                for captures in regx::KBD_MACRO_KEYS.captures_iter(&keys_src).step_by(2) {
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
                let note = if note.is_empty() { None } else { Some(note) };
                acc.push_node(Macro(Footnote { id, text: note }), macro_loc);
                break;
              }
              "xref:" => {
                let target = line.consume_macro_target(self.bump);
                self.push_xref(&target);
                lines.restore_if_nonempty(line);
                let nodes = self.parse_inlines_until(lines, &[Kind(CloseBracket)])?;
                let linktext = if nodes.is_empty() {
                  macro_loc.end = target.loc.end + 2;
                  None
                } else {
                  extend(&mut macro_loc, &nodes, 1);
                  Some(nodes)
                };
                acc.push_node(
                  Macro(Xref {
                    target,
                    linktext,
                    kind: XrefKind::Macro,
                  }),
                  macro_loc,
                );
                break;
              }
              "link:" => {
                if !line.no_whitespace_until(OpenBracket) {
                  // turns out we didn't have a valid uri target here
                  acc.push_text_token(&token);
                  let next_token = line.consume_current().unwrap();
                  acc.push_text_token(&next_token);
                } else {
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
                      SourceLocation::new(
                        pos as u32,
                        (pos + trimmed.len()) as u32,
                        rest.loc.include_depth,
                      ),
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
                self.insert_anchor(
                  &id,
                  Anchor {
                    reftext: attrs.take_positional(0),
                    title: InlineNodes::new(self.bump),
                    source_loc: Some(id.loc),
                    source_idx: self.lexer.source_idx(),
                    is_biblio: false,
                  },
                )?;
                acc.push_node(InlineAnchor(id.src), id.loc);
              }
              _ => {
                let mut name = token.lexeme;
                let mut source = name.clone();
                let rest = line.reassemble_src();
                source.push_str(&rest);
                name.pop(); // trailing colon
                let target = if !line.current_is(OpenBracket) {
                  Some(line.consume_macro_target(self.bump))
                } else {
                  line.discard_assert(OpenBracket);
                  None
                };
                let attrs = self.parse_block_attr_list(&mut line)?;
                let loc = SourceLocation::spanning(token.loc, attrs.loc);
                acc.push_node(
                  Macro(Plugin(PluginMacro {
                    name,
                    target,
                    flow: Flow::Inline,
                    attrs,
                    source: SourceString::new(source, loc),
                  })),
                  loc,
                );
              }
            }
          }

          // to match rx, we intentionally fail to recognize bare url links from invalid
          // link macros like `link:http://foo.com`, so consume the uri scheme as text
          MacroName
            if subs.macros() && line.current_is(UriScheme) && token.lexeme.as_str() == "link:" =>
          {
            acc.push_text_token(&token);
            let next_token = line.consume_current().unwrap();
            acc.push_text_token(&next_token);
          }

          UriScheme if subs.macros() && line.continues_inline_macro(&token) => {
            self.parse_uri_scheme_macro(&token, &mut line, &mut acc)?
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
            acc.push_text_token(&token);
          }

          // if we're in a table cell, and we have a blank attr ref
          // we need to preserve a node for the crazy table cell paragraph logic
          AttrRef
            if self.ctx.table_cell_ctx != TableCellContext::None
              // we know it was blank if there's no inserted token w/ same loc
              && line.current_token().is_some_and(|t| t.loc != token.loc) =>
          {
            acc.push_node(Discarded, token.loc)
          }

          TermDelimiter
            if subs.callouts()
              && token.is_len(2)
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
              && token.is_len(2)
              && line.current_is(Whitespace)
              && line.continues_valid_callout_nums() =>
          {
            push_callout_tuck(token, &mut line, &mut acc);
          }

          CalloutNumber if subs.callouts() && line.continues_valid_callout_nums() => {
            self.recover_custom_line_comment(&mut acc);
            acc.text.trim_end();
            let loc = SourceLocation::new(acc.text.loc.end, token.loc.end, token.loc.include_depth);
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
            let mut inner = line.extract_line_before(&[Kind(GreaterThan), Kind(GreaterThan)]);
            let target = inner.consume_to_string_until(Comma, self.bump);
            self.push_xref(&target);
            let mut linktext = None;
            if !inner.is_empty() {
              inner.discard_assert(Comma);
              inner.trim_leading_whitespace();
              if !inner.is_empty() {
                linktext = Some(self.parse_inlines(&mut inner.into_lines())?);
              }
            }
            line.discard_assert(GreaterThan);
            loc.end = line.consume_current().unwrap().loc.end;
            acc.push_node(
              Macro(Xref {
                target,
                linktext,
                kind: XrefKind::Shorthand,
              }),
              loc,
            );
          }

          LessThan
            if subs.macros()
              && line.current_token().kind(UriScheme)
              && line.no_whitespace_until(GreaterThan) =>
          {
            acc.push_node(Discarded, token.loc);
            let scheme_token = line.consume_current().unwrap();
            let mut loc = scheme_token.loc;
            let line_end = line.last_loc().unwrap();
            let target = line.consume_url(Some(&scheme_token), Some(GreaterThan), self.bump);
            if target.src == scheme_token.lexeme {
              // turns out we don't have a valid uri here, backtrack
              acc.pop_node();
              if subs.special_chars() {
                acc.push_node(SpecialChar(SpecialCharKind::LessThan), token.loc);
              } else {
                acc.push_node(Text(token.lexeme), token.loc);
              }
              acc.push_text_token(&scheme_token);
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

          MaybeEmail if subs.macros() && regx::EMAIL_RE.is_match(&token.lexeme) => {
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
              && self.starts_constrained(&[Kind(Underscore)], &token, &line, lines) =>
          {
            self.ctx.inline_ctx = InlineCtx::Single([Kind(Underscore)]);
            self.parse_node(Italic, [Kind(Underscore)], &token, &mut acc, line, lines)?;
            break;
          }

          Underscore
            if subs.inline_formatting()
              && self.starts_unconstrained(&[Kind(Underscore); 2], &token, &line, lines) =>
          {
            self.ctx.inline_ctx = InlineCtx::Double([Kind(Underscore); 2]);
            self.parse_node(Italic, [Kind(Underscore); 2], &token, &mut acc, line, lines)?;
            break;
          }

          Star
            if subs.inline_formatting()
              && self.starts_constrained(&[Kind(Star)], &token, &line, lines) =>
          {
            self.ctx.inline_ctx = InlineCtx::Single([Kind(Star)]);
            self.parse_node(Bold, [Kind(Star)], &token, &mut acc, line, lines)?;
            break;
          }

          Star
            if subs.inline_formatting()
              && self.starts_unconstrained(&[Kind(Star); 2], &token, &line, lines) =>
          {
            self.ctx.inline_ctx = InlineCtx::Double([Kind(Star); 2]);
            self.parse_node(Bold, [Kind(Star); 2], &token, &mut acc, line, lines)?;
            break;
          }

          OpenBracket if subs.inline_formatting() && line.continues_formatted_text_attr_list() => {
            let mut parse_token = token.clone();
            let attr_list = self.parse_formatted_text_attr_list(&mut line)?;
            debug_assert!(line.current_is(Hash));
            line.discard_assert(Hash);
            parse_token.kind = Hash;
            let span = |inner| TextSpan(attr_list, inner);
            self.parse_node(span, [Kind(Hash)], &parse_token, &mut acc, line, lines)?;
            if let Some(InlineNode { content: TextSpan(attrs, nodes), .. }) = acc.inlines.last() {
              if let Some(id) = &attrs.id {
                self.insert_anchor(
                  id,
                  Anchor {
                    reftext: None,
                    title: nodes.clone(),
                    source_idx: self.lexer.source_idx(),
                    source_loc: Some(id.loc),
                    is_biblio: false,
                  },
                )?;
              }
            }
            break;
          }

          OpenBracket
            if line.starts_with_seq(&[Kind(OpenBracket); 2])
              && self.ctx.bibliography_ctx == BiblioContext::List
              && line.contains_seq(&[Kind(CloseBracket); 3]) =>
          {
            let second_bracket = line.consume_current().unwrap();
            let third_bracket = line.consume_current().unwrap();
            if let Some(mut anchor) = self.parse_inline_anchor(&mut line)? {
              self.insert_anchor(
                &anchor.id,
                self.anchor_from(anchor.reftext, Some(anchor.id.loc), true),
              )?;
              anchor.loc.extend(line.consume_current().unwrap().loc);
              acc.push_node(BiblioAnchor(anchor.id.src), anchor.loc);
            } else {
              acc.push_text_token(&second_bracket);
              acc.push_text_token(&third_bracket);
              acc.push_text_token(&token);
            }
          }

          OpenBracket if line.starts_with_seq(&[Kind(OpenBracket); 2]) => {
            acc.push_text_token(&token);
          }

          OpenBracket if line.continues_inline_anchor() => {
            let second_bracket = line.consume_current().unwrap();
            if let Some(anchor) = self.parse_inline_anchor(&mut line)? {
              self.insert_anchor(
                &anchor.id,
                self.anchor_from(anchor.reftext, Some(anchor.id.loc), false),
              )?;
              acc.push_node(InlineAnchor(anchor.id.src), anchor.loc);
            } else {
              acc.push_text_token(&second_bracket);
              acc.push_text_token(&token);
            }
          }

          Backtick
            if subs.inline_formatting()
              && line.starts_with_seq(&[Kind(Plus), Not(Plus)])
              && !line.starts_with_seq(&[Kind(Plus), Kind(Backtick)])
              && contains_seq(&[Len(1, Plus), Kind(Backtick)], &line, lines) =>
          {
            self.ctx.subs.remove(Subs::InlineFormatting);
            self.ctx.subs.remove(Subs::AttrRefs);
            self.parse_node(
              LitMono,
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
              && self.starts_constrained(&[Kind(Backtick)], &token, &line, lines) =>
          {
            self.ctx.inline_ctx = InlineCtx::Single([Kind(Backtick)]);
            self.parse_node(Mono, [Kind(Backtick)], &token, &mut acc, line, lines)?;
            break;
          }

          Backtick
            if subs.inline_formatting()
              && self.starts_unconstrained(&[Kind(Backtick); 2], &token, &line, lines) =>
          {
            self.parse_node(Mono, [Kind(Backtick); 2], &token, &mut acc, line, lines)?;
            break;
          }

          DoubleQuote
            if subs.inline_formatting()
              && line.current_is(Backtick)
              && self.starts_constrained(
                &[Kind(Backtick), Kind(DoubleQuote)],
                &token,
                &line,
                lines,
              ) =>
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
              && self.starts_constrained(
                &[Kind(Backtick), Kind(SingleQuote)],
                &token,
                &line,
                lines,
              ) =>
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
              && self.starts_unconstrained(&[Kind(Hash); 2], &token, &line, lines) =>
          {
            self.ctx.inline_ctx = InlineCtx::Double([Kind(Hash); 2]);
            self.parse_node(Highlight, [Kind(Hash); 2], &token, &mut acc, line, lines)?;
            break;
          }

          Hash
            if subs.inline_formatting()
              && self.starts_constrained(&[Kind(Hash)], &token, &line, lines) =>
          {
            self.ctx.inline_ctx = InlineCtx::Single([Kind(Hash)]);
            self.parse_node(Highlight, [Kind(Hash)], &token, &mut acc, line, lines)?;
            break;
          }

          // already encoded entities, eg: &#8212;
          Ampersand if line.starts_with_seq(&[Kind(Hash), Kind(Digits), Kind(SemiColon)]) => {
            acc.push_text_token(&token);
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
            if acc.text.is_empty()
              || acc
                .text
                .ends_with(|c| c.is_whitespace() || c.is_ascii_punctuation())
            {
              acc.push_text_token(&token);
            } else {
              acc.push_node(CurlyQuote(LegacyImplicitApostrophe), token.loc);
            }
          }

          Whitespace if token.lexeme.len() > 1 && subs.inline_formatting() => {
            acc.push_node(MultiCharWhitespace(token.lexeme), token.loc);
          }

          Whitespace
            if line.current_is(Plus)
              && line
                .peek_token()
                .is_none_or(|t| t.kind == TokenKind::Newline) =>
          {
            let mut loc = token.loc;
            line.discard_assert(Plus);
            loc.end += 2; // plus and newline
            acc.push_node(LineBreak, loc);
            if !line.is_empty() {
              // NB: see `break_in_table` test, we can get here only when
              // we've coalesced tokens including a break in a table cell
              loc.end = line.discard_assert(TokenKind::Newline).loc.end;
            } else {
              break;
            }
          }

          TokenKind::Newline => acc.push_node(Inline::Newline, token.loc),

          Discard | AttrRef => acc.text.loc = token.loc.clamp_end(),

          Backslash => {
            match line.current_token().map(|t| t.kind) {
              Some(Word) | None => acc.push_text_token(&token),
              _ => {
                acc.push_node(Discarded, token.loc);
                // pushing the next token as text prevents recognizing the pattern
                let next_token = line.consume_current().unwrap();
                acc.push_text_token(&next_token);
              }
            }
          }

          _ if subs.macros() && token.kind(UriScheme) => {
            let mut loc = token.loc;
            let line_end = line.last_loc().unwrap();
            let target = line.consume_url(Some(&token), None, self.bump);
            if target.src == token.lexeme {
              acc.push_text_token(&token);
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

          _ => acc.push_text_token(&token),
        }
      }
    }

    acc.commit();
    Ok(acc.trimmed_inlines())
  }

  fn push_xref(&mut self, target: &SourceString<'arena>) {
    let mut ref_id = target.src.clone();
    let mut ref_loc = target.loc;
    if ref_id.len() > 1 && ref_id.starts_with('#') {
      ref_id.drain(..1);
      ref_loc.start += 1;
    }
    self.ctx.xrefs.borrow_mut().insert(ref_id, ref_loc);
  }

  fn parse_uri_scheme_macro(
    &mut self,
    token: &Token<'arena>,
    line: &mut Line<'arena>,
    acc: &mut Accum<'arena>,
  ) -> Result<()> {
    let mut macro_loc = token.loc;
    let line_end = line.last_loc().unwrap();
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
      debug_assert!(tok.kind == spec.token_kind().unwrap());
      stop_len += tok.len();
    });
    lines.restore_if_nonempty(line);
    let inner = self.parse_inlines_until(lines, &stop_tokens)?;
    extend(&mut loc, &inner, stop_len);
    state.push_node(wrap(inner), loc);
    push_newline_if_needed(state, lines);
    self.ctx.inline_ctx = InlineCtx::None;
    Ok(())
  }

  fn should_stop_at(&self, line: &Line<'arena>) -> bool {
    // delimiter
    (line.current_is(DelimiterLine) && self.ctx.can_nest_blocks)

    // new block from attr list-ish
    || ((line.is_block_attr_list() || line.is_block_anchor()) && self.ctx.delimiter.is_none())

    // description list
    || (
      self.ctx.parsing_description_list()
      && (line.starts_description_list_item() || line.is_list_continuation())
    )

    // list continuation
    || (self.ctx.list.parsing_continuations && line.is_list_continuation())

    // special case: ending verbatim delimited block, non-matching delimiters
    // within the verbatim block are rendered as is
    || (
      self.ctx.delimiter.is_some()
      && self.ctx.delimiter == line.current_token().and_then(|t| t.to_delimiter())
    )
  }

  fn recover_custom_line_comment(&mut self, state: &mut Accum<'arena>) {
    let Some(ref comment_bytes) = self.ctx.custom_line_comment else {
      return;
    };
    let mut line_txt = state.text.str().as_bytes();
    let line_len = line_txt.len();
    let mut back = comment_bytes.len() as u32;
    if line_txt.ends_with(b" ") {
      back += 1;
      line_txt = &line_txt[..line_len - 1];
    }
    if line_txt.ends_with(comment_bytes) {
      let tuck = self.string(state.text.str().split_at(line_len - back as usize).1);
      state.text.drop_last(back);
      let tuck_loc = SourceLocation::new(
        state.text.loc.end,
        state.text.loc.end + back,
        state.text.loc.include_depth,
      );
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
    .is_some_and(|line| line.is_fully_unconsumed())
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

#[cfg(test)]
mod tests {
  use super::*;
  use test_utils::*;

  #[test]
  fn unexpected_block_macro() {
    let input = "image::foo.png[]";
    let mut parser = test_parser!(input);
    let mut block = parser.read_lines().unwrap().unwrap();
    let inlines = parser.parse_inlines(&mut block).unwrap();
    expect_eq!(inlines, nodes![node!("image::foo.png[]"; 0..16)], from: input);
  }
}
