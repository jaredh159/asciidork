use crate::internal::*;
use crate::variants::token::*;

#[derive(Debug, Clone)]
pub struct Line<'arena> {
  tokens: Deq<'arena, Token<'arena>>,
  orig_len: u32,
  term_delim: bool,
  pass_macro: bool,
  pass_pluses: u8,
}

impl<'arena> Line<'arena> {
  pub fn new(tokens: Deq<'arena, Token<'arena>>) -> Self {
    Line {
      orig_len: tokens.len() as u32,
      tokens,
      term_delim: false,
      pass_macro: false,
      pass_pluses: 0,
    }
  }

  pub fn empty(bump: &'arena Bump) -> Self {
    Line {
      orig_len: 0,
      tokens: Deq::new(bump),
      term_delim: false,
      pass_macro: false,
      pass_pluses: 0,
    }
  }

  pub fn with_capacity(capacity: usize, bump: &'arena Bump) -> Self {
    Line {
      orig_len: 0,
      tokens: Deq::with_capacity(capacity, bump),
      term_delim: false,
      pass_macro: false,
      pass_pluses: 0,
    }
  }

  pub fn may_contain_inline_pass(&self) -> bool {
    if !self.pass_macro && self.pass_pluses == 0 {
      false
    } else if self.pass_macro || self.pass_pluses > 1 {
      true
    } else {
      let last = self.last().unwrap();
      if last.is_kind_len(Plus, 1) && self.len() > 1 {
        // NB: line ending ` +` is a hardbreak, cannot be pass
        !self.nth_token(self.len() - 2).is_whitespaceish()
      } else {
        true
      }
    }
  }

  pub fn push(&mut self, token: Token<'arena>) {
    match token.kind {
      MacroName if token.lexeme == "pass:" => self.pass_macro = true,
      TermDelimiter => self.term_delim = true,
      Plus if self.tokens.last().not_kind(Backtick) && token.len() < 4 => {
        self.pass_pluses = self.pass_pluses.saturating_add(1)
      }
      _ => {}
    }
    self.tokens.push(token);
    self.orig_len += 1;
  }

  pub fn push_nonpass(&mut self, token: Token<'arena>) {
    self.tokens.push(token);
    self.orig_len += 1;
  }

  pub fn last(&self) -> Option<&Token<'arena>> {
    self.tokens.last()
  }

  pub fn pop(&mut self) -> Option<Token<'arena>> {
    self.tokens.pop()
  }

  pub fn drain_into(self, tokens: &mut Deq<'arena, Token<'arena>>) {
    tokens.extend(self.tokens.into_iter());
  }

  pub fn into_bytes(self) -> BumpVec<'arena, u8> {
    let mut bytes = BumpVec::new_in(self.tokens.bump);
    if let (Some(first), Some(last)) = (self.tokens.first(), self.tokens.last()) {
      bytes.reserve((last.loc.end - first.loc.start) as usize);
    }
    for token in self.tokens.iter() {
      bytes.extend_from_slice(token.lexeme.as_bytes());
    }
    bytes
  }

  pub const fn bump_arena(&self) -> &'arena Bump {
    self.tokens.bump
  }

  pub fn src_eq(&self, other: &Self) -> bool {
    if self.tokens.len() != other.tokens.len() {
      return false;
    }
    if self.src_len() != other.src_len() {
      return false;
    }
    for (a, b) in self.tokens.iter().zip(other.tokens.iter()) {
      if a.lexeme != b.lexeme {
        return false;
      }
    }
    true
  }

  pub fn current_token(&self) -> Option<&Token<'arena>> {
    self.tokens.get(0)
  }

  pub fn current_token_mut(&mut self) -> Option<&mut Token<'arena>> {
    self.tokens.get_mut(0)
  }

  pub fn peek_token(&self) -> Option<&Token<'arena>> {
    self.tokens.get(1)
  }

  pub fn last_token(&self) -> Option<&Token<'arena>> {
    self.tokens.last()
  }

  pub fn last_loc(&self) -> Option<SourceLocation> {
    self.last_token().map(|t| t.loc)
  }

  pub fn nth_token(&self, n: usize) -> Option<&Token<'arena>> {
    self.tokens.get(n)
  }

  pub fn num_tokens(&self) -> usize {
    self.tokens.len()
  }

  pub fn current_is(&self, kind: TokenKind) -> bool {
    self.current_token().kind(kind)
  }

  pub fn current_is_len(&self, kind: TokenKind, len: usize) -> bool {
    self.current_token().is_kind_len(kind, len)
  }

  pub fn unadjusted_heading_level(&self) -> Option<u8> {
    if self.starts_with_seq(&[Kind(EqualSigns), Kind(Whitespace)]) && self.num_tokens() > 2 {
      Some((self.current_token().unwrap().lexeme.len() - 1) as u8)
    } else {
      None
    }
  }

  pub fn is_empty(&self) -> bool {
    self.tokens.is_empty()
  }

  pub fn is_heading(&self) -> bool {
    self.unadjusted_heading_level().is_some()
  }

  pub fn is_block_macro(&self) -> bool {
    self.starts_with_seq(&[Kind(MacroName), Kind(Colon)])
      && self.current_token().can_start_block_macro()
      && self.contains(OpenBracket)
      && self.ends_with_nonescaped(CloseBracket)
  }

  pub fn is_block_attr_list(&self) -> bool {
    self.is_fully_unconsumed()
      && self.starts(OpenBracket)
      && self.ends_with_nonescaped(CloseBracket)
      && !self.peek_token().kind(OpenBracket)
  }

  pub fn is_block_anchor(&self) -> bool {
    self.starts_with_seq(&[Kind(OpenBracket); 2])
      && self.num_tokens() > 4
      && !matches!(
        self.nth_token(2).unwrap().kind,
        SingleQuote | DoubleQuote | Whitespace
      )
      && self.ends_with_nonescaped(CloseBracket)
      && self.nth_token(self.num_tokens() - 2).kind(CloseBracket)
  }

  pub fn is_chunk_title(&self) -> bool {
    // dot followed by at least one non-whitespace token
    self.starts(Dots) && self.iter().len() > 1 && self.peek_token().unwrap().not_kind(Whitespace)
  }

  pub fn is_delimiter(&self, delimiter: Delimiter) -> bool {
    self.num_tokens() == 1 && self.current_token().unwrap().to_delimeter() == Some(delimiter)
  }

  pub fn is_indented(&self) -> bool {
    self.starts(Whitespace) && self.num_tokens() > 1
  }

  pub fn discard(&mut self, n: usize) {
    for _ in 0..n {
      _ = self.consume_current();
    }
  }

  pub fn discard_assert(&mut self, kind: TokenKind) -> Token<'arena> {
    let token = self.consume_current().unwrap();
    assert!(token.kind == kind);
    token
  }

  pub fn discard_last(&mut self) -> Option<Token<'arena>> {
    self.tokens.pop()
  }

  pub fn discard_assert_last(&mut self, kind: TokenKind) {
    let token = self.discard_last();
    debug_assert!(token.unwrap().kind(kind));
  }

  pub fn contains_nonescaped(&self, token_type: TokenKind) -> bool {
    self.first_nonescaped(token_type).is_some()
  }

  pub fn ends_with_nonescaped(&self, token_type: TokenKind) -> bool {
    match self.iter().len() {
      0 => false,
      1 => self.current_is(token_type),
      n => self.last_token().kind(token_type) && self.nth_token(n - 2).not_kind(Backslash),
    }
  }

  pub fn len(&self) -> usize {
    self.tokens.len()
  }

  pub fn iter(&self) -> impl ExactSizeIterator<Item = &Token<'arena>> {
    self.tokens.iter()
  }

  pub fn iter_mut(&mut self) -> impl ExactSizeIterator<Item = &mut Token<'arena>> {
    self.tokens.iter_mut()
  }

  pub fn into_iter(self) -> impl ExactSizeIterator<Item = Token<'arena>> {
    self.tokens.into_iter()
  }

  pub fn first_nonescaped(&self, kind: TokenKind) -> Option<(&Token, usize)> {
    let mut prev: Option<TokenKind> = None;
    for (i, token) in self.iter().enumerate() {
      if token.kind(kind) && prev != Some(Backslash) {
        return Some((token, i));
      }
      prev = Some(token.kind);
    }
    None
  }

  pub fn has_seq_at(&self, specs: &[TokenSpec], offset: u32) -> bool {
    if specs.is_empty() || self.len() < offset as usize + specs.len() {
      return false;
    }
    for (i, spec) in specs.iter().enumerate() {
      let token = self.tokens.get(i + offset as usize).unwrap();
      if !token.satisfies(*spec) {
        return false;
      }
    }
    true
  }

  pub fn contains(&self, kind: TokenKind) -> bool {
    self.iter().any(|t| t.kind == kind)
  }

  pub fn contains_len(&self, kind: TokenKind, len: usize) -> bool {
    self.iter().any(|t| t.kind == kind && t.lexeme.len() == len)
  }

  pub fn starts(&self, kind: TokenKind) -> bool {
    self.current_is(kind)
  }

  pub fn starts_with(&self, predicate: impl Fn(&Token<'arena>) -> bool) -> bool {
    self.current_token().map(predicate).unwrap_or(false)
  }

  pub fn is_comment(&self) -> bool {
    self.is_fully_unconsumed() && self.current_is_len(ForwardSlashes, 2)
  }

  pub fn ends(&self, kind: TokenKind) -> bool {
    self.last_token().kind(kind)
  }

  pub fn starts_with_seq(&self, tokens: &[TokenSpec]) -> bool {
    self.has_seq_at(tokens, 0)
  }

  pub fn contains_seq(&self, specs: &[TokenSpec]) -> bool {
    self.index_of_seq(specs).is_some()
  }

  pub fn index_of_kind(&self, kind: TokenKind) -> Option<usize> {
    self.iter().position(|t| t.kind(kind))
  }

  pub fn index_of_seq(&self, specs: &[TokenSpec]) -> Option<usize> {
    assert!(!specs.is_empty());
    if self.len() < specs.len() {
      return None;
    }
    let first_spec = specs.first().unwrap();
    'outer: for (i, token) in self.iter().enumerate() {
      if token.satisfies(*first_spec) {
        if self.len() - i < specs.len() {
          return None;
        }
        for (j, spec) in specs.iter().skip(1).enumerate() {
          if !self.tokens.get(i + j + 1).unwrap().satisfies(*spec) {
            continue 'outer;
          }
        }
        return Some(i);
      }
    }
    None
  }

  pub fn continues_valid_callout_nums(&self) -> bool {
    for token in self.iter() {
      if token.kind(Whitespace) || token.kind(CalloutNumber) {
        continue;
      } else {
        return false;
      }
    }
    true
  }

  pub fn continues_inline_macro(&self, prev: &Token) -> bool {
    if self.current_is(Whitespace) {
      return false;
    }
    let Some(open_idx) = self.index_of_kind(OpenBracket) else {
      return false;
    };
    let Some(end_idx) = self.index_of_seq(&[Not(Backslash), Kind(CloseBracket)]) else {
      return false;
    };
    if end_idx < open_idx {
      false
    } else {
      !self.current_is(Colon) || prev.lexeme.as_str() == "xref:"
    }
  }

  pub fn continues_xref_shorthand(&self) -> bool {
    self.current_is(LessThan)
      && self.num_tokens() > 3
      && self.contains_seq(&[Kind(GreaterThan), Kind(GreaterThan)])
      && self.nth_token(1).not_kind(GreaterThan)
      && self.nth_token(1).not_kind(LessThan)
      && self.nth_token(1).not_kind(Whitespace)
  }

  /// `true` if no whitespace until token type *and* token type is found
  pub fn no_whitespace_until(&self, kind: TokenKind) -> bool {
    for token in self.iter() {
      if token.kind(kind) {
        return true;
      } else if token.kind(Whitespace) {
        return false;
      } else {
        continue;
      }
    }
    false
  }

  pub fn terminates_constrained(&self, stop_tokens: &[TokenSpec], ctx: &InlineCtx) -> bool {
    self.terminates_constrained_in(stop_tokens, ctx).is_some()
  }

  pub fn terminates_constrained_in(
    &self,
    stop_tokens: &[TokenSpec],
    ctx: &InlineCtx,
  ) -> Option<usize> {
    match self.index_of_seq(stop_tokens) {
      // constrained sequences can't immediately terminate
      // or else `foo __bar` would include an empty italic node
      // TODO: maybe that's only true for _single_ tok sequences?
      Some(0) => None,
      Some(n) if self.nth_token(n + 1).not_kind(Word) => match ctx.specs() {
        Some(specs) => self
          .index_of_seq(specs)
          .map_or(Some(n), |m| if m < n { None } else { Some(n) }),
        None => Some(n),
      },
      _ => None,
    }
  }

  #[must_use]
  pub fn consume_to_string_until(
    &mut self,
    kind: TokenKind,
    bump: &'arena Bump,
  ) -> SourceString<'arena> {
    let mut loc = self.loc().expect("no tokens to consume");
    let mut s = BumpString::new_in(bump);
    while let Some(token) = self.consume_if_not(kind) {
      if token.kind != AttrRef && token.kind != Backslash {
        s.push_str(&token.lexeme);
      }
      loc.extend(token.loc);
    }
    SourceString::new(s, loc)
  }

  #[must_use]
  pub fn consume_to_string_until_one_of(
    &mut self,
    spec: &[TokenSpec],
    bump: &'arena Bump,
  ) -> SourceString<'arena> {
    let mut loc = self.loc().expect("no tokens to consume");
    let mut s = BumpString::new_in(bump);
    loop {
      let Some(peek) = self.current_token() else {
        break;
      };
      if peek.satisfies_any(spec) {
        break;
      }
      let token = self.consume_current().unwrap();
      s.push_str(&token.lexeme);
      loc.extend(token.loc);
    }
    SourceString::new(s, loc)
  }

  #[must_use]
  pub fn consume_to_string(&mut self, bump: &'arena Bump) -> SourceString<'arena> {
    let mut loc = self.loc().expect("no tokens to consume");
    let mut s = BumpString::new_in(bump);
    while let Some(token) = self.consume_current() {
      s.push_str(&token.lexeme);
      loc.extend(token.loc);
    }
    SourceString::new(s, loc)
  }

  pub fn consume_if_not(&mut self, kind: TokenKind) -> Option<Token> {
    match self.current_token() {
      Some(token) if !token.kind(kind) => self.consume_current(),
      _ => None,
    }
  }

  #[must_use]
  pub fn consume_macro_target(&mut self, bump: &'arena Bump) -> SourceString<'arena> {
    let target = self.consume_to_string_until(OpenBracket, bump);
    self.discard_assert(OpenBracket);
    target
  }

  #[must_use]
  pub fn consume_optional_macro_target(
    &mut self,
    bump: &'arena Bump,
  ) -> Option<SourceString<'arena>> {
    let target = match self.current_is(OpenBracket) {
      true => None,
      false => Some(self.consume_to_string_until(OpenBracket, bump)),
    };
    self.discard_assert(OpenBracket);
    target
  }

  #[must_use]
  pub fn consume_url(
    &mut self,
    start: Option<&Token>,
    stop: Option<TokenKind>,
    bump: &'arena Bump,
  ) -> SourceString<'arena> {
    let mut loc = start.map_or_else(|| self.loc().unwrap(), |t| t.loc);
    let mut num_tokens = 0;

    if let Some(stop) = stop {
      num_tokens = self.index_of_kind(stop).unwrap_or(0);
    } else {
      for token in self.iter() {
        match token.kind {
          Whitespace | GreaterThan | OpenBracket | OpenParens | CloseParens | Bang | SemiColon
          | Colon | Star | QuestionMark => break,
          _ => num_tokens += 1,
        }
      }
    }

    if stop.is_none() && num_tokens > 0 && self.tokens.get(num_tokens - 1).kind(Dots) {
      num_tokens -= 1;
    }

    let mut s = BumpString::new_in(bump);
    if let Some(start) = start {
      s.push_str(&start.lexeme);
      loc.extend(start.loc);
    }
    for _ in 0..num_tokens {
      let token = self.consume_current().unwrap();
      s.push_str(&token.lexeme);
      loc.extend(token.loc);
    }
    SourceString::new(s, loc)
  }

  #[must_use]
  pub fn consume_current(&mut self) -> Option<Token<'arena>> {
    self.tokens.pop_front()
  }

  pub fn into_lines(self) -> ContiguousLines<'arena> {
    let mut lines = Deq::with_capacity(1, self.tokens.bump);
    lines.push(self);
    ContiguousLines::new(lines)
  }

  pub fn loc(&self) -> Option<SourceLocation> {
    self.current_token().map(|t| t.loc)
  }

  pub fn last_location(&self) -> Option<SourceLocation> {
    self.last_token().map(|t| t.loc)
  }

  pub fn src_len(&self) -> usize {
    if self.tokens.is_empty() {
      0
    } else {
      self.tokens.iter().map(|token| token.lexeme.len()).sum()
    }
  }

  pub fn reassemble_src(&self) -> BumpString<'arena> {
    let mut src = BumpString::with_capacity_in(self.src_len(), self.tokens.bump);
    for token in self
      .tokens
      .iter()
      .filter(|t| !matches!(t.kind, AttrRef | Discard))
    {
      src.push_str(&token.lexeme);
    }
    src
  }

  pub fn list_marker(&self) -> Option<ListMarker> {
    if self.is_comment() {
      return None;
    }

    let mut offset = 0;
    if self.current_token().kind(Whitespace) {
      offset += 1;
    }
    let token = self.nth_token(offset)?;
    let second = self.nth_token(offset + 1);
    let third = self.nth_token(offset + 2);

    match token.kind {
      Star if second.kind(Whitespace) && third.is_some() => Some(ListMarker::Star(1)),
      Dots if second.kind(Whitespace) && third.is_some() => {
        Some(ListMarker::Dot(token.len() as u8))
      }
      Dashes if second.kind(Whitespace) && token.len() == 1 && third.is_some() => {
        Some(ListMarker::Dash)
      }
      Star if second.kind(Star) => {
        let src = self.reassemble_src();
        let captures = regx::REPEAT_STAR_LI_START.captures(&src)?;
        Some(ListMarker::Star(captures.get(1).unwrap().len() as u8))
      }
      CalloutNumber if token.lexeme.as_bytes()[1] != b'!' => {
        Some(ListMarker::Callout(token.parse_callout_num()))
      }
      Digits if second.kind(Dots) && third.kind(Whitespace) => {
        Some(ListMarker::Digits(token.lexeme.parse().unwrap()))
      }
      TermDelimiter => None, // we need to see a non-whitespace token first
      _ if self.term_delim => {
        for token in self.iter().skip(offset) {
          if token.kind(TermDelimiter) {
            return match token.lexeme.as_str() {
              "::" => Some(ListMarker::Colons(2)),
              ":::" => Some(ListMarker::Colons(3)),
              "::::" => Some(ListMarker::Colons(4)),
              ";;" => Some(ListMarker::SemiColons),
              _ => unreachable!(),
            };
          }
        }
        None
      }
      _ => None,
    }
  }

  pub fn starts_list_item(&self) -> bool {
    self.list_marker().is_some()
  }

  pub fn starts_description_list_item(&self) -> bool {
    self
      .list_marker()
      .map(|marker| marker.is_description())
      .unwrap_or(false)
  }

  pub fn continues_list_item_principle(&self) -> bool {
    if self.is_comment() {
      return false;
    }
    match self.current_token().map(|t| t.kind) {
      Some(OpenBracket) => !self.is_block_attr_list(),
      Some(Plus) | None => false,
      _ => !self.starts_list_item(),
    }
  }

  pub fn is_list_continuation(&self) -> bool {
    self.num_tokens() == 1 && self.starts(Plus)
  }

  pub fn trim_leading_whitespace(&mut self) {
    while self.current_is(Whitespace) {
      self.discard(1);
    }
  }

  pub fn discard_leading_whitespace(&mut self) {
    if self.current_is(Whitespace) {
      self.tokens.get_mut(0).unwrap().kind = Discard;
    }
  }

  pub fn drop_leading_bytes(&mut self, n: u32) {
    debug_assert!(n as usize <= self.current_token().unwrap().lexeme.len());
    if n > 0 {
      self.tokens.get_mut(0).unwrap().drop_leading_bytes(n);
    }
  }

  pub fn starts_nested_list(&self, stack: &ListStack) -> bool {
    self
      .list_marker()
      .map(|marker| stack.starts_nested_list(marker))
      .unwrap_or(false)
  }

  pub fn consume_checklist_item(
    &mut self,
    bump: &'arena Bump,
  ) -> Option<(bool, SourceString<'arena>)> {
    if !self.starts(OpenBracket) || !self.has_seq_at(&[Kind(CloseBracket), Kind(Whitespace)], 2) {
      return None;
    }
    let inside = self.nth_token(1).unwrap();
    let (src, checked) = match inside {
      Token { kind: Star, .. } => ("[*]", true),
      Token { kind: Whitespace, .. } => ("[ ]", false),
      Token { kind: Word, lexeme, .. } if *lexeme == "x" => ("[x]", true),
      _ => return None,
    };
    let mut loc = self.loc().unwrap();
    loc.end += 2;
    self.discard(3);
    let src = BumpString::from_str_in(src, bump);
    Some((checked, SourceString::new(src, loc)))
  }

  pub fn extract_line_before(&mut self, seq: &[TokenSpec]) -> Line<'arena> {
    let mut line = Line::with_capacity(self.num_tokens(), self.tokens.bump);
    while !self.starts_with_seq(seq) {
      line.push(self.consume_current().unwrap());
    }
    line
  }

  pub fn is_partially_consumed(&self) -> bool {
    self.tokens.len() < self.orig_len as usize
  }

  pub fn is_fully_unconsumed(&self) -> bool {
    self.tokens.len() == self.orig_len as usize
  }

  pub fn trim_for_cell(&mut self, style: CellContentStyle) {
    // literal cell should preserve only leading spaces
    if matches!(style, CellContentStyle::Literal) {
      while self.current_is(Newline) {
        self.discard(1);
      }
    }
    while self.last_token().is_whitespaceish() {
      self.discard_last();
    }
  }

  // for asciidoc cells, we get a new document context
  // so we need to throw away the resolved attr refs and re-resolve
  // instead of dropping the tokens, we can blank out the lexeme
  // because we create a new lexer from the reassembled lexemes as src
  pub fn remove_resolved_attr_refs(&mut self) {
    let mut attr_loc: Option<SourceLocation> = None;
    let bump = self.tokens.bump;
    for token in self.iter_mut() {
      if token.kind(AttrRef) {
        attr_loc = Some(token.loc);
      } else if attr_loc.is_some_and(|loc| loc == token.loc) {
        token.lexeme = BumpString::from_str_in("", bump);
      } else {
        attr_loc = None;
      }
    }
  }

  pub fn get_indentation(&self) -> usize {
    self
      .current_token()
      .filter(|t| t.kind(Whitespace))
      .map_or(0, |t| t.lexeme.len())
  }

  pub fn set_indentation(&mut self, indent: usize) {
    let Some(token) = self.current_token_mut() else {
      return;
    };
    let token_len = token.len();
    if token.kind(Whitespace) && token_len > indent {
      let delta = token_len - indent;
      if delta == token_len {
        self.tokens.remove_first();
      } else {
        token.loc.start += delta as u32;
        for _ in 0..delta {
          token.lexeme.pop();
        }
      }
    } else if token.kind(Whitespace) && token_len < indent {
      // this is a little hinky, as the source locations are lying now
      // maybe would be better to have an explicit token type to formalize
      // a indentation created out of thin air with no source location ?
      token.lexeme.push_str(&" ".repeat(indent - token_len));
    } else if !token.kind(Whitespace) && indent != 0 {
      let loc = token.loc.clamp_start();
      // ... also hinky
      self.tokens.slowly_push_front(Token::new(
        Whitespace,
        loc,
        BumpString::from_str_in(&" ".repeat(indent), self.tokens.bump),
      ));
    }
  }

  pub fn is_attr_decl(&self) -> bool {
    if !self.current_is(TokenKind::Colon) || self.num_tokens() < 2 {
      return false;
    }
    let src = self.reassemble_src();
    regx::ATTR_DECL.is_match(&src)
  }

  pub fn is_directive_endif(&self) -> bool {
    self.directive_endif_target().is_some()
  }

  pub fn directive_endif_target(&self) -> Option<BumpString<'arena>> {
    if !self
      .current_token()
      .matches(TokenKind::Directive, "endif::")
      || self.num_tokens() < 3
    {
      return None;
    }
    let src = self.reassemble_src();
    regx::DIRECTIVE_ENDIF.captures(&src).map(|captures| {
      let attrs = captures.get(1).map_or("", |c| c.as_str());
      BumpString::from_str_in(attrs, self.tokens.bump)
    })
  }
}

#[cfg(test)]
mod tests {
  use crate::internal::*;
  use crate::token::{TokenKind::*, TokenSpec::*, *};
  use test_utils::*;

  #[test]
  fn set_indentation() {
    let cases = vec![
      ("    foo", 2, "  foo", 2..4),
      ("    foo", 1, " foo", 3..4),
      ("  foo", 2, "  foo", 0..2),
      ("  foo", 0, "foo", 2..5),
      ("foo  ", 2, "  foo  ", 0..0),
      ("  foo", 4, "    foo", 0..2),
      ("foo", 4, "    foo", 0..0),
    ];
    for (input, indent, expected, range) in cases {
      let mut line = read_line!(input);
      line.set_indentation(indent);
      expect_eq!(line.reassemble_src(), expected, from: input);
      expect_eq!(line.current_token().unwrap().loc, range.into(), from: input);
    }
  }

  #[test]
  fn test_continues_list_item_principle() {
    let cases = vec![
      ("foo", true),
      (" foo", true),
      ("      foo", true),
      ("* foo", false),
      ("  * foo", false),
      ("- foo", false),
      ("// foo", false),
      ("[circles]", false),
      ("term::", false),
      ("term:: desc", false),
    ];
    for (input, expected) in cases {
      let line = read_line!(input);
      expect_eq!(line.continues_list_item_principle(), expected, from: input);
    }
  }

  #[test]
  fn test_starts_nested_list() {
    use ListMarker::*;
    let cases: Vec<(&str, &[ListMarker], bool)> = vec![
      ("* foo", &[Star(1)], false),
      ("** foo", &[Star(1)], true),
      ("* foo", &[Star(2)], true),
      (". foo", &[Star(2), Star(1)], true),
      ("2. foo", &[Digits(1)], false),
      ("<2> bar", &[Callout(Some(1))], false),
    ];
    for (input, markers, expected) in cases {
      let mut stack = ListStack::default();
      for marker in markers {
        stack.push(*marker);
      }
      let line = read_line!(input);
      expect_eq!(line.starts_nested_list(&stack), expected, from: input);
    }
  }

  #[test]
  fn test_list_marker() {
    use ListMarker::*;
    let cases = vec![
      ("* foo", Some(Star(1))),
      ("** foo", Some(Star(2))),
      (". foo", Some(Dot(1))),
      (".. foo", Some(Dot(2))),
      ("... foo", Some(Dot(3))),
      ("- foo", Some(Dash)),
      ("1. foo", Some(Digits(1))),
      ("999. foo", Some(Digits(999))),
      ("2. foo", Some(Digits(2))),
      ("--- foo", None),
      ("33.44. foo", None),
      (":: bar", None),
      ("* ", None),
      ("** ", None),
      ("*** ", None),
      (" ", None),
      (". ", None),
      (".. ", None),
      ("... ", None),
      ("- ", None),
      ("foo:: bar", Some(Colons(2))),
      ("foo::", Some(Colons(2))),
      ("image:: baz", Some(Colons(2))),
      ("image::cat.png[]", None),
      ("foo::: bar", Some(Colons(3))),
      ("foo:::: bar", Some(Colons(4))),
      ("foo;; bar", Some(SemiColons)),
      ("_foo_::", Some(Colons(2))),
      ("foo bar:: baz", Some(Colons(2))),
      ("<1> foo", Some(Callout(Some(1)))),
      ("<.> foo", Some(Callout(None))),
      ("<!--3--> foo", None), // CalloutNumber token, but not a list marker
      ("<255> foo", Some(Callout(Some(255)))),
    ];
    for (input, marker) in cases {
      let line = read_line!(input);
      expect_eq!(line.list_marker(), marker, from: input);
    }
  }

  #[test]
  fn test_starts_list_item() {
    let cases = vec![
      ("* foo", true),
      ("foo", false),
      ("- foo", true),
      ("-- foo", false),
      ("   - foo", true),
      (". foo", true),
      ("**** foo", true),
      ("1. foo", true),
      ("999. foo", true),
      (" * foo", true),
      ("   * foo", true),
      ("* {foo}", true),
      (". {foo}", true),
      ("*foo", false),
      (".foo", false),
      ("-foo", false),
      (" ::", false),
      ("//foo:: bar", false),
    ];
    for (input, expected) in cases {
      let line = read_line!(input);
      expect_eq!(line.starts_list_item(), expected, from: input);
    }
  }

  #[test]
  fn test_discard() {
    let mut line = read_line!("foo bar\nso baz\n");
    expect_eq!(line.reassemble_src(), "foo bar");
    expect_eq!(line.num_tokens(), 3);
    line.discard(1);
    expect_eq!(line.reassemble_src(), " bar");
    expect_eq!(line.num_tokens(), 2);
    line.discard(2);
    expect_eq!(line.reassemble_src(), "");
    expect_eq!(line.num_tokens(), 0);
  }

  #[test]
  fn test_discard_last() {
    let mut line = read_line!("'foo'");
    expect_eq!(line.reassemble_src(), "'foo'");
    line.discard_last();
    expect_eq!(line.reassemble_src(), "'foo");
    line.discard_last();
    expect_eq!(line.reassemble_src(), "'");
  }

  #[test]
  fn test_line_has_seq_at() {
    let cases: Vec<(&str, &[TokenSpec], u32, bool)> = vec![
      ("foo bar_:", &[Kind(Word), Kind(Whitespace)], 0, true),
      ("foo bar_:", &[Kind(Word), Kind(Whitespace)], 1, false),
      ("foo bar", &[Kind(Whitespace), Kind(Word)], 1, true),
      (
        "foo bar_:",
        &[Kind(Word), Kind(Underscore), Kind(Colon)],
        2,
        true,
      ),
      (
        "foo bar_:",
        &[Kind(Word), Kind(Underscore), Kind(Colon)],
        0,
        false,
      ),
      ("#", &[Kind(Hash)], 0, true),
    ];
    for (input, token_types, pos, expected) in cases {
      let line = read_line!(input);
      expect_eq!(line.has_seq_at(token_types, pos), expected);
    }

    // test that it works after shifting elements off of the front
    let mut line = read_line!("foo_#");
    line.discard(2); // `foo` and `_`
    assert!(line.has_seq_at(&[Kind(Hash)], 0));
  }

  #[test]
  fn test_ends_nonescaped() {
    let cases: Vec<(&str, TokenKind, bool)> = vec![
      ("x", CloseBracket, false),
      ("]", CloseBracket, true),
      ("\\]", CloseBracket, false),
      ("l]", CloseBracket, true),
    ];
    for (input, token_type, expected) in cases {
      let line = read_line!(input);
      expect_eq!(line.ends_with_nonescaped(token_type), expected);
    }
  }

  #[test]
  fn test_line_terminates_constrained_in() {
    let cases: Vec<(&str, &[TokenSpec], Option<usize>)> = vec![
      ("foo_ bar", &[Kind(Underscore)], Some(1)),
      ("foo_bar bar", &[Kind(Underscore)], None),
    ];
    for (input, specs, expected) in cases {
      let line = read_line!(input);
      expect_eq!(line.terminates_constrained_in(specs, &InlineCtx::None), expected, from: input);
    }
  }

  #[test]
  fn test_line_contains_seq() {
    let cases: Vec<(&str, &[TokenSpec], bool)> = vec![
      ("_bar__r", &[Kind(Underscore), Kind(Underscore)], true),
      ("foo bar_:", &[Kind(Word), Kind(Whitespace)], true),
      (
        "foo bar_:",
        &[Kind(Word), Kind(Whitespace), Kind(Word)],
        true,
      ),
      ("foo bar_:", &[Kind(Word)], true),
      ("foo bar_:", &[Kind(Underscore), Kind(Colon)], true),
      ("foo bar_:", &[Kind(Underscore), Kind(Word)], false),
      (
        "foo bar_:",
        &[Kind(Whitespace), Kind(Word), Kind(Underscore)],
        true,
      ),
      (
        "foo ",
        &[Kind(Word), Kind(Whitespace), Kind(Underscore), Kind(Colon)],
        false,
      ),
    ];
    for (input, token_types, expected) in cases {
      let line = read_line!(input);
      expect_eq!(line.contains_seq(token_types), expected, from: input);
    }
  }
}
