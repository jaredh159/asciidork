use std::fmt::Write;

use crate::internal::*;

impl<'arena> Parser<'arena> {
  pub(super) fn replace_inline_pass(
    &mut self,
    line: &mut Line<'arena>,
    lines: &mut ContiguousLines<'arena>,
  ) -> Result<()> {
    let mut replaced = Line::with_capacity(line.num_tokens(), self.bump);
    let mut prev_kind = None;
    while let Some(token) = line.consume_current() {
      let kind = token.kind;
      match token.kind {
        TokenKind::MacroName if token.lexeme == "pass:" => {
          if let Some((n, subs)) = self.terminates_valid_pass_macro(line, lines) {
            let placeholder = self.pass_placeholder(&token, line, lines, n, subs)?;
            replaced.push_nonpass(placeholder);
          } else {
            replaced.push_nonpass(token);
          };
        }
        TokenKind::Plus if prev_kind != Some(TokenKind::Backtick) && token.len() < 4 => {
          if let Some(n) = terminates_plus(token.len() as u8, line, lines) {
            let subs = Substitutions::from_pass_plus_token(&token);
            let placeholder = self.pass_placeholder(&token, line, lines, n, subs)?;
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
    line: &mut Line<'arena>,
    lines: &mut ContiguousLines<'arena>,
    num_passthru_tokens: usize,
    subs: Substitutions,
  ) -> Result<Token<'arena>> {
    let mut loc = start_token.loc;
    let mut passlines = self.passthru_parse_lines(num_passthru_tokens, line, lines);
    let extend = match start_token.len() as u32 {
      5 => 1, // `pass:` ends with `]`
      n => n, // plus ends with same-len plus
    };
    loc.end = passlines.last_loc().unwrap().end + extend;
    let restore_subs = self.ctx.subs;
    self.ctx.subs = subs;
    let nodes = self.parse_inlines(&mut passlines)?;
    self.ctx.subs = restore_subs;
    self.ctx.passthrus.push(Some(nodes));
    let index = self.ctx.passthrus.len() - 1;
    let mut lexeme = BumpString::with_capacity_in(6, self.bump);
    write!(lexeme, "^{:05}", index).unwrap();
    Ok(Token::new(TokenKind::PreprocPassthru, loc, lexeme))
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
        passthru.push(passthru_line);
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
    let mut num_target_tokens = 0;
    while num_target_tokens < line.num_tokens() {
      let token = line.nth_token(num_target_tokens).unwrap();
      match token.kind {
        TokenKind::OpenBracket => break,
        TokenKind::Word | TokenKind::Comma => num_target_tokens += 1,
        _ => return None,
      }
    }

    let mut n = 1;
    let mut last = TokenKind::OpenBracket;
    while num_target_tokens + n < line.num_tokens() {
      let token = line.nth_token(num_target_tokens + n).unwrap();
      if token.kind == TokenKind::CloseBracket && last != TokenKind::Backslash {
        let subs = pass_macro_subs(line, num_target_tokens, self.bump);
        return Some((n - 1, subs));
      }
      last = token.kind;
      n += 1;
    }

    // search rest of paragraph
    for next_line in lines.iter() {
      for token in next_line.iter() {
        if token.kind == TokenKind::CloseBracket && last != TokenKind::Backslash {
          let subs = pass_macro_subs(line, num_target_tokens, self.bump);
          return Some((n - 1, subs));
        }
        last = token.kind;
        n += 1;
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
  line.discard_assert(TokenKind::OpenBracket);
  Substitutions::from_pass_macro_target(target)
}

fn terminates_plus(len: u8, line: &Line, lines: &ContiguousLines) -> Option<usize> {
  if len == 1 {
    return terminates_constrained_plus(line, lines);
  }

  let spec = TokenSpec::Len(len, TokenKind::Plus);
  let mut n = 0;
  for token in line.iter() {
    if token.satisfies(spec) {
      return Some(n);
    } else {
      n += 1;
    }
  }
  // search rest of paragraph
  for line in lines.iter() {
    if let Some(m) = line.index_of_seq(&[spec]) {
      return Some(n + m);
    } else {
      n += line.num_tokens();
    }
  }
  None
}

fn terminates_constrained_plus(line: &Line, lines: &ContiguousLines) -> Option<usize> {
  let stop = &[TokenSpec::Len(1, TokenKind::Plus)];
  if let Some(n) = line.terminates_constrained_in(stop) {
    return Some(n);
  }
  let mut n = line.num_tokens();
  for line in lines.iter() {
    if let Some(m) = line.terminates_constrained_in(stop) {
      return Some(n + m);
    } else {
      n += line.num_tokens();
    }
  }
  None
}

#[cfg(test)]
mod tests {
  use super::*;
  use test_utils::*;

  #[test]
  fn test_replace_inline_pass() {
    let mut parser = Parser::from_str("foo +bar+ baz", leaked_bump());
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
        Token::new(TokenKind::Word, 0..3, bstr!("foo")),
        Token::new(TokenKind::Whitespace, 3..4, bstr!(" ")),
        Token::new(TokenKind::PreprocPassthru, 4..9, bstr!("^00000")),
        Token::new(TokenKind::Whitespace, 9..10, bstr!(" ")),
        Token::new(TokenKind::Word, 10..13, bstr!("baz")),
      ]
    );
  }
}
