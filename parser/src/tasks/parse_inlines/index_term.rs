use crate::internal::*;
use crate::tasks::parse_inlines::inline_utils::*;
use crate::variants::token::*;

impl<'arena> Parser<'arena> {
  pub(crate) fn parse_index_term_macro(
    &mut self,
    visible: bool,
    token: &Token,
    line: &mut Line<'arena>,
  ) -> Result<(Inline<'arena>, SourceLocation)> {
    let mut loc = token.loc;
    line.discard_assert(TokenKind::OpenBracket);
    let mut attr_list = self.parse_inline_attr_list(line)?;
    loc.end = attr_list.loc.end;
    let Some(term) = attr_list.positional.get_mut(0).and_then(Option::take) else {
      self.err_at("invalid index term macro", attr_list.loc)?;
      let invalid_macro = self.lexer.str_from_loc(loc);
      return Ok((Inline::Text(self.string(invalid_macro)), loc));
    };

    let term_type = if visible {
      IndexTermType::Visible { term }
    } else {
      let secondary = attr_list.positional.get_mut(1).and_then(Option::take);
      let tertiary = attr_list.positional.get_mut(2).and_then(Option::take);
      IndexTermType::Concealed { primary: term, secondary, tertiary }
    };

    let mut term_ref = IndexTermReference::None;
    if let Some(see_term) = attr_list.named("see") {
      term_ref = IndexTermReference::See(self.string(see_term));
    } else if let Some(see_also_terms) = attr_list.named("see-also") {
      let terms = see_also_terms
        .split(',')
        .map(|s| self.string(s.trim()))
        .collect();
      term_ref = IndexTermReference::SeeAlso(terms);
    }

    Ok((Inline::IndexTerm(IndexTerm { term_type, term_ref }), loc))
  }

  pub(crate) fn parse_index_term_shorthand(
    &mut self,
    end_len: usize,
    open_parens_token: &Token<'arena>,
    mut line: Line<'arena>,
    lines: &mut ContiguousLines<'arena>,
    acc: &mut Accum<'arena>,
  ) -> Result<(Inline<'arena>, SourceLocation)> {
    match (open_parens_token.len(), end_len) {
      (2, end) => self.parse_index_term_shorthand_visible(end, open_parens_token, line, lines),
      (start, 2) => {
        let mut single = open_parens_token.clone();
        single.lexeme.truncate(start - 2);
        single.loc.end -= 2;
        acc.push_text_token(&single);
        self.parse_index_term_shorthand_visible(2, open_parens_token, line, lines)
      }
      (3, 3) => self.parse_index_term_concealed(3, open_parens_token, line, lines),
      (start, end) => {
        if start > end || start > 3 {
          let mut put_back = open_parens_token.clone();
          put_back.lexeme.truncate(start - 3);
          put_back.loc.start += 3;
          line.restore_front(put_back);
        }
        self.parse_index_term_concealed(end, open_parens_token, line, lines)
      }
    }
  }

  fn parse_index_term_shorthand_visible(
    &mut self,
    end_len: usize,
    open_parens_token: &Token<'arena>,
    line: Line<'arena>,
    lines: &mut ContiguousLines<'arena>,
  ) -> Result<(Inline<'arena>, SourceLocation)> {
    let mut node_loc = open_parens_token.loc;
    lines.restore_if_nonempty(line);
    while lines.current_token().is_whitespaceish() {
      lines.consume_current_token();
    }
    let mut term = self.parse_inlines_until(lines, &[Len(end_len as u8, CloseParens)])?;
    term.trim_trailing_whitespace();
    node_loc.end = term.loc().map(|ml| ml.end_pos).unwrap_or(node_loc.end);

    if end_len > 2 {
      let mut trailing_parens_loc = node_loc.clamp_end();
      trailing_parens_loc.end += (end_len - 2) as u32;
      let trailing_parens = self.lexer.src_string_from_loc(trailing_parens_loc);
      if let Some(InlineNode { content: Inline::Text(txt), loc }) = term.last_mut() {
        txt.push_str(&trailing_parens.src);
        loc.extend(trailing_parens_loc);
      } else {
        term.push(InlineNode {
          content: Inline::Text(trailing_parens.src),
          loc: trailing_parens_loc,
        });
      }
      node_loc.end += (end_len as u32) - 2;
    }

    node_loc.end += 2;
    Ok((
      Inline::IndexTerm(ast::IndexTerm {
        term_type: IndexTermType::Visible { term },
        term_ref: IndexTermReference::None,
      }),
      node_loc,
    ))
  }

  fn parse_index_term_concealed(
    &mut self,
    end_len: usize,
    open_parens_token: &Token<'arena>,
    line: Line<'arena>,
    lines: &mut ContiguousLines<'arena>,
  ) -> Result<(Inline<'arena>, SourceLocation)> {
    lines.restore_if_nonempty(line);
    let mut terms = lines.consume_splitting_csv_until(Len(end_len as u8, CloseParens), self.bump);
    if terms.len() > 3 {
      self.err_at(
        "too many terms in concealed indexterm, max 3",
        open_parens_token.loc,
      )?;
      terms.truncate(3);
    }

    let close_parens = lines.consume_current_token().unwrap();
    debug_assert!(close_parens.kind == CloseParens);
    let loc = open_parens_token.loc.setting_end(close_parens.loc.end);

    if close_parens.len() > 3 {
      let mut put_back = close_parens.clone();
      put_back.lexeme.truncate(close_parens.len() - 3);
      put_back.loc.end -= 3;
      if let Some(last_term) = terms.last_mut() {
        last_term.push(put_back);
      }
    };

    let mut primary = InlineNodes::new(self.bump);
    let mut secondary = None;
    let mut tertiary = None;
    for (i, term_tokens) in terms.into_iter().enumerate() {
      let mut lines = Line::new(term_tokens).into_lines();
      let term = self.parse_inlines(&mut lines)?;
      match i {
        0 => primary = term,
        1 => secondary = Some(term),
        _ => tertiary = Some(term),
      }
    }
    Ok((
      Inline::IndexTerm(ast::IndexTerm {
        term_type: IndexTermType::Concealed { primary, secondary, tertiary },
        term_ref: IndexTermReference::None,
      }),
      loc,
    ))
  }

  pub(crate) fn parse_escaped_index_term_shorthand(
    &mut self,
    token: &Token<'arena>,
    mut line: Line<'arena>,
    lines: &mut ContiguousLines<'arena>,
    acc: &mut Accum<'arena>,
  ) -> Result<()> {
    let mut single_paren = token.clone();
    single_paren.loc = single_paren.loc.incr();
    single_paren.lexeme = self.string("(");
    let mut parens = line.consume_current().unwrap();
    parens.loc.start += 1;
    parens.lexeme.pop();
    acc.push_node(Inline::Discarded, token.loc);
    acc.push_text_token(&single_paren);
    let end_len = terminates(&line, lines).unwrap();
    let (node, loc) = self.parse_index_term_shorthand(end_len, &parens, line, lines, acc)?;
    acc.push_node(node, loc);
    Ok(())
  }
}

impl<'arena> ContiguousLines<'arena> {
  fn consume_splitting_csv_until(
    &mut self,
    stop: TokenSpec,
    bump: &'arena Bump,
  ) -> BumpVec<'arena, Deq<'arena, Token<'arena>>> {
    let mut items = BumpVec::with_capacity_in(3, bump);
    let mut tokens = Deq::with_capacity(4, bump);
    let mut in_quotes = false;
    loop {
      let Some(current) = self.current_token() else {
        break;
      };
      if current.satisfies(stop) {
        break;
      }
      match (current.kind, current.len(), in_quotes) {
        (Comma, _, false) => {
          while tokens.last().kind(Whitespace) {
            tokens.pop();
          }
          if !tokens.is_empty() {
            let mut item = Deq::with_capacity(tokens.len(), bump);
            std::mem::swap(&mut item, &mut tokens);
            items.push(item);
          }
          self.consume_current_token();
          while self.current_token().kind(Whitespace) {
            self.consume_current_token();
          }
        }
        (DoubleQuote, _, false) => {
          in_quotes = true;
          self.consume_current_token();
        }
        (DoubleQuote, _, true) => {
          in_quotes = false;
          self.consume_current_token();
        }
        _ => {
          tokens.push(self.consume_current_token().unwrap());
        }
      }
    }
    while tokens.last().kind(Whitespace) {
      tokens.pop();
    }
    if !tokens.is_empty() {
      items.push(tokens);
    }
    items
  }
}

pub fn terminates(line: &Line, lines: &ContiguousLines) -> Option<usize> {
  if line.starts(CloseParens) {
    None
  } else {
    line
      .terminates_index_term()
      .or_else(|| lines.terminates_index_term())
  }
}
