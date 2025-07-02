use std::fmt::Write;

use crate::internal::*;
use crate::variants::token::*;

impl<'arena> Parser<'arena> {
  pub(super) fn replace_inline_pass(
    &mut self,
    line: &mut Line<'arena>,
    lines: &mut ContiguousLines<'arena>,
  ) -> Result<()> {
    let mut replaced = Line::with_capacity(line.num_tokens(), self.bump);
    let mut prev_kind = None;
    let mut skip_tokens = 0;
    while let Some(token) = line.consume_current() {
      if skip_tokens > 0 {
        replaced.push_nonpass(token);
        skip_tokens -= 1;
        continue;
      }
      let kind = token.kind;
      match token.kind {
        MacroName if token.lexeme == "pass:" => {
          if let Some((n, subs)) = self.terminates_valid_pass_macro(line, lines) {
            let placeholder = self.pass_placeholder(&token, 0, line, lines, n, subs)?;
            replaced.push_nonpass(placeholder);
          } else {
            replaced.push_nonpass(token);
          };
        }
        Plus if could_be_plus_passthru(prev_kind, line) => {
          let count = line.iter().take_while(|t| t.kind(Plus)).count() + 1;
          if let Some(n) = self.terminates_plus(count, line, lines) {
            let subs = Substitutions::from_pass_plus_len(count);
            let placeholder = self.pass_placeholder(&token, count, line, lines, n, subs)?;
            skip_tokens = count;
            replaced.push_nonpass(placeholder);
          } else {
            replaced.push_nonpass(token);
          }
        }
        _ => replaced.push_nonpass(token),
      }
      prev_kind = Some(kind);
    }
    lines.restore_if_nonempty(replaced);
    Ok(())
  }

  fn pass_placeholder(
    &mut self,
    start_token: &Token<'arena>,
    plus_count: usize,
    line: &mut Line<'arena>,
    lines: &mut ContiguousLines<'arena>,
    mut num_passthru_tokens: usize,
    subs: Substitutions,
  ) -> Result<Token<'arena>> {
    let mut loc = start_token.loc;
    if plus_count > 1 {
      line.discard(plus_count - 1);
      num_passthru_tokens -= plus_count - 1;
    }
    let mut passlines = self.passthru_parse_lines(num_passthru_tokens, line, lines);
    if plus_count > 1 {
      line.discard(plus_count - 1);
    }
    let extend = if start_token.kind == Plus { plus_count as u32 } else { 1 };
    loc.end = passlines.last_loc().unwrap().end + extend;
    let restore_subs = self.ctx.subs;
    self.ctx.subs = subs;
    let nodes = self.parse_inlines(&mut passlines)?;
    self.ctx.subs = restore_subs;
    self.ctx.passthrus.push(Some(nodes));
    let index = self.ctx.passthrus.len() - 1;
    let mut lexeme = BumpString::with_capacity_in(6, self.bump);
    write!(lexeme, "^{index:05}").unwrap();
    Ok(Token::new(PreprocPassthru, loc, lexeme))
  }

  fn passthru_parse_lines(
    &self,
    num_tokens: usize,
    line: &mut Line<'arena>,
    lines: &mut ContiguousLines<'arena>,
  ) -> ContiguousLines<'arena> {
    let passthru = ContiguousLines::with_capacity(1, self.bump);
    self.accum_passthru(num_tokens, line, lines, passthru)
  }

  fn accum_passthru(
    &self,
    mut num_tokens: usize,
    line: &mut Line<'arena>,
    source: &mut ContiguousLines<'arena>,
    mut passthru: ContiguousLines<'arena>,
  ) -> ContiguousLines<'arena> {
    let mut passthru_line = Line::with_capacity(num_tokens, self.bump);
    loop {
      if num_tokens == 0 {
        line.discard(1); // discard passthru end delimiter
        passthru.push(passthru_line);
        return passthru;
      }
      if let Some(token) = line.consume_current() {
        passthru_line.push_nonpass(token);
        num_tokens -= 1;
      } else {
        if !passthru_line.is_empty() {
          passthru.push(passthru_line);
        }
        // NB: we know there is a next line because we found the end further on
        let mut next_line = source.consume_current().unwrap();
        std::mem::swap(line, &mut next_line);
        return self.accum_passthru(num_tokens, line, source, passthru);
      }
    }
  }

  fn terminates_valid_pass_macro(
    &self,
    line: &mut Line<'arena>,
    lines: &mut ContiguousLines<'arena>,
  ) -> Option<(usize, Substitutions)> {
    if line.is_empty() {
      return None;
    }
    let mut num_target_tokens = 0;
    while num_target_tokens < line.num_tokens() {
      let token = line.nth_token(num_target_tokens).unwrap();
      match token.kind {
        OpenBracket => break,
        Word | Comma => num_target_tokens += 1,
        _ => return None,
      }
    }

    let mut n = 1;
    let mut last = OpenBracket;
    while num_target_tokens + n < line.num_tokens() {
      let token = line.nth_token(num_target_tokens + n).unwrap();
      if token.kind == CloseBracket && last != Backslash {
        let subs = pass_macro_subs(line, num_target_tokens, self.bump);
        return Some((n - 1, subs));
      }
      last = token.kind;
      n += 1;
    }

    // search rest of paragraph
    for next_line in lines.iter() {
      for token in next_line.iter() {
        if token.kind == CloseBracket && last != Backslash {
          let subs = pass_macro_subs(line, num_target_tokens, self.bump);
          return Some((n - 1, subs));
        }
        last = token.kind;
        n += 1;
      }
    }

    None
  }

  fn terminates_plus(
    &self,
    plus_count: usize,
    line: &Line,
    lines: &ContiguousLines,
  ) -> Option<usize> {
    if plus_count == 1 {
      return self.terminates_constrained_plus(line, lines);
    }

    let spec = &[TokenSpec::Kind(Plus); 3][..plus_count];

    if let Some(n) = line.index_of_seq(spec) {
      return Some(n);
    }

    // search rest of paragraph
    let orig_n = line.num_tokens();
    let mut n = orig_n;
    for line in lines.iter() {
      if self.ctx.delimiter.is_some_and(|d| line.is_delimiter(d)) {
        return None; // can't span over pending delimiter
      }
      if let Some(m) = line.index_of_seq(spec) {
        if m == 0 && n == orig_n {
          return None; // empty multiline, e.g. "++\n++"
        } else {
          return Some(n + m);
        }
      } else {
        n += line.num_tokens();
      }
    }
    None
  }

  fn terminates_constrained_plus(&self, line: &Line, lines: &ContiguousLines) -> Option<usize> {
    if line.current_is(Newline) {
      return None;
    }
    let stop = &[TokenSpec::Len(1, Plus)];
    if let Some(n) = line.terminates_constrained_in(stop, &InlineCtx::None) {
      return Some(n);
    }
    let mut n = line.num_tokens();
    for line in lines.iter() {
      if self.ctx.delimiter.is_some_and(|d| line.is_delimiter(d)) {
        return None; // can't span over pending delimiter
      }
      if let Some(m) = line.terminates_constrained_in(stop, &InlineCtx::None) {
        return Some(n + m);
      } else {
        n += line.num_tokens();
      }
    }
    None
  }
}

fn pass_macro_subs<'arena>(
  line: &mut Line<'arena>,
  n_target_tokens: usize,
  bump: &'arena Bump,
) -> Substitutions {
  let mut target = BumpString::with_capacity_in(8, bump);
  for _ in 0..n_target_tokens {
    target.push_str(&line.consume_current().unwrap().lexeme);
  }
  line.discard_assert(OpenBracket);
  Substitutions::from_pass_macro_target(target)
}

#[inline(always)]
fn could_be_plus_passthru(prev_kind: Option<TokenKind>, line: &Line) -> bool {
  if prev_kind == Some(Backtick) || line.starts_with_seq(&[TokenSpec::Kind(Plus); 3]) {
    false
  } else if !line.starts(Plus) {
    prev_kind.is_none() || matches!(prev_kind, Some(Whitespace | Plus))
  } else {
    true
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use test_utils::*;

  #[test]
  fn test_replace_inline_pass() {
    let mut parser = test_parser!("foo +bar+ baz");
    let mut lines = parser.read_lines().unwrap().unwrap();
    let mut line = lines.consume_current().unwrap();
    assert!(line.may_contain_inline_pass());
    assert!(lines.is_empty());

    parser.replace_inline_pass(&mut line, &mut lines).unwrap();
    let mut replaced = lines.consume_current().unwrap();
    assert!(!replaced.may_contain_inline_pass());
    assert!(lines.is_empty());

    assert_eq!(
      std::array::from_fn(|_| replaced.consume_current().unwrap()),
      [
        Token::new(Word, loc!(0..3), bstr!("foo")),
        Token::new(Whitespace, loc!(3..4), bstr!(" ")),
        Token::new(PreprocPassthru, loc!(4..9), bstr!("^00000")),
        Token::new(Whitespace, loc!(9..10), bstr!(" ")),
        Token::new(Word, loc!(10..13), bstr!("baz")),
      ]
    );
  }
}
