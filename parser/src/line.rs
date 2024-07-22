use lazy_static::lazy_static;
use regex::Regex;

use crate::internal::*;
use crate::variants::token::*;

#[derive(Debug, Clone)]
pub struct Line<'arena> {
  tokens: Deq<'arena, Token<'arena>>,
  orig_len: usize,
}

impl<'arena> Line<'arena> {
  pub fn new(tokens: Deq<'arena, Token<'arena>>) -> Self {
    Line { orig_len: tokens.len(), tokens }
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
    self.current_token().is(kind)
  }

  pub fn current_is_len(&self, kind: TokenKind, len: usize) -> bool {
    self.current_token().is_len(kind, len)
  }

  pub fn heading_level(&self) -> Option<u8> {
    if self.starts_with_seq(&[EqualSigns, Whitespace]) && self.num_tokens() > 2 {
      Some((self.current_token().unwrap().lexeme.len() - 1) as u8)
    } else {
      None
    }
  }

  pub fn is_empty(&self) -> bool {
    self.tokens.is_empty()
  }

  pub fn is_heading(&self) -> bool {
    self.heading_level().is_some()
  }

  pub fn is_heading_level(&self, level: u8) -> bool {
    self.heading_level() == Some(level)
  }

  pub fn is_block_macro(&self) -> bool {
    self.starts_with_seq(&[MacroName, Colon])
      && self.contains(OpenBracket)
      && self.ends_with_nonescaped(CloseBracket)
  }

  pub fn is_attr_list(&self) -> bool {
    if !self.starts(OpenBracket) || !self.ends_with_nonescaped(CloseBracket) {
      false
    // support legacy [[id,pos]] anchor syntax
    } else if self.starts_with_seq(&[OpenBracket, OpenBracket]) {
      self.ends_with_nonescaped(CloseBracket)
        && self.nth_token(self.num_tokens() - 2).is(CloseBracket)
    } else {
      true
    }
  }

  pub fn is_chunk_title(&self) -> bool {
    // dot followed by at least one non-whitespace token
    self.starts(Dots) && self.tokens().len() > 1 && self.peek_token().unwrap().is_not(Whitespace)
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

  pub fn discard_assert(&mut self, kind: TokenKind) {
    let token = self.consume_current();
    debug_assert!(token.unwrap().is(kind));
  }

  pub fn discard_last(&mut self) -> Option<Token<'arena>> {
    self.tokens.pop()
  }

  pub fn discard_assert_last(&mut self, kind: TokenKind) {
    let token = self.discard_last();
    debug_assert!(token.unwrap().is(kind));
  }

  pub fn contains_nonescaped(&self, token_type: TokenKind) -> bool {
    self.first_nonescaped(token_type).is_some()
  }

  pub fn ends_with_nonescaped(&self, token_type: TokenKind) -> bool {
    match self.tokens().len() {
      0 => false,
      1 => self.current_is(token_type),
      n => self.last_token().is(token_type) && self.nth_token(n - 2).is_not(Backslash),
    }
  }

  fn tokens(&self) -> impl ExactSizeIterator<Item = &Token<'arena>> {
    self.tokens.iter()
  }

  fn tokens_mut(&mut self) -> impl ExactSizeIterator<Item = &mut Token<'arena>> {
    self.tokens.iter_mut()
  }

  pub fn first_nonescaped(&self, kind: TokenKind) -> Option<&Token> {
    let mut prev: Option<TokenKind> = None;
    for token in self.tokens() {
      if token.is(kind) && prev.map_or(true, |k| k != Backslash) {
        return Some(token);
      }
      prev = Some(token.kind);
    }
    None
  }

  pub fn has_seq_at(&self, kinds: &[TokenKind], offset: u32) -> bool {
    if kinds.is_empty() || self.tokens().len() < offset as usize + kinds.len() {
      return false;
    }
    for (i, token_type) in kinds.iter().enumerate() {
      if self.tokens.get(i + offset as usize).unwrap().kind != *token_type {
        return false;
      }
    }
    true
  }

  pub fn contains(&self, kind: TokenKind) -> bool {
    self.tokens().any(|t| t.kind == kind)
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
    self.last_token().is(kind)
  }

  pub fn starts_with_seq(&self, kinds: &[TokenKind]) -> bool {
    self.has_seq_at(kinds, 0)
  }

  pub fn contains_seq(&self, kind: &[TokenKind]) -> bool {
    self.index_of_seq(kind).is_some()
  }

  pub fn index_of_seq(&self, kinds: &[TokenKind]) -> Option<usize> {
    if self.tokens().len() < kinds.len() {
      return None;
    }
    let Some(first_kind) = kinds.first() else {
      return None;
    };
    'outer: for (i, token) in self.tokens().enumerate() {
      if token.kind == *first_kind {
        if self.tokens().len() - i < kinds.len() {
          return None;
        }
        for (j, kind) in kinds.iter().skip(1).enumerate() {
          if self.tokens.get(i + j + 1).unwrap().kind != *kind {
            continue 'outer;
          }
        }
        return Some(i);
      }
    }
    None
  }

  pub fn continues_valid_callout_nums(&self) -> bool {
    for token in self.tokens() {
      if token.is(Whitespace) || token.is(CalloutNumber) {
        continue;
      } else {
        return false;
      }
    }
    true
  }

  pub fn continues_inline_macro(&self) -> bool {
    !self.current_is(Colon)
      && self.is_continuous_thru(OpenBracket)
      && self.contains_nonescaped(CloseBracket)
  }

  pub fn continues_xref_shorthand(&self) -> bool {
    self.current_is(LessThan)
      && self.num_tokens() > 3
      && self.contains_seq(&[GreaterThan, GreaterThan])
      && self.nth_token(1).is_not(GreaterThan)
      && self.nth_token(1).is_not(LessThan)
      && self.nth_token(1).is_not(Whitespace)
  }

  /// true if there is no whitespace until token type, and token type is found
  pub fn is_continuous_thru(&self, kind: TokenKind) -> bool {
    for token in self.tokens() {
      if token.is(kind) {
        return true;
      } else if token.is(Whitespace) {
        return false;
      } else {
        continue;
      }
    }
    false
  }

  pub fn terminates_constrained(&self, stop_tokens: &[TokenKind]) -> bool {
    match self.index_of_seq(stop_tokens) {
      // constrained sequences can't immediately terminate
      // or else `foo __bar` would include an empty italic node
      // TODO: maybe that's only true for _single_ tok sequences?
      Some(0) => false,
      Some(n) if self.nth_token(n).is_not(Word) => true,
      _ => false,
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
      Some(token) if !token.is(kind) => self.consume_current(),
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
  pub fn consume_url(&mut self, start: Option<&Token>, bump: &'arena Bump) -> SourceString<'arena> {
    let mut loc = start.map_or_else(|| self.loc().unwrap(), |t| t.loc);
    let mut num_tokens = 0;

    for token in self.tokens() {
      match token.kind {
        Whitespace | GreaterThan | OpenBracket => break,
        _ => num_tokens += 1,
      }
    }

    if num_tokens > 0 && self.tokens.get(num_tokens - 1).is(Dots) {
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
    let mut lines = Deq::with_capacity(self.tokens.bump, 1);
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
    for token in self.tokens.iter() {
      src.push_str(&token.lexeme);
    }
    src
  }

  pub fn list_marker(&self) -> Option<ListMarker> {
    // PERF: checking for list markers seems sort of sad, wonder if the
    // Line could be created with some markers to speed these tests up
    let mut offset = 0;
    if self.current_token().is(Whitespace) {
      offset += 1;
    }
    let Some(token) = self.nth_token(offset) else {
      return None;
    };
    let second = self.nth_token(offset + 1);
    let third = self.nth_token(offset + 2);

    match token.kind {
      Star if second.is(Whitespace) && third.is_some() => Some(ListMarker::Star(1)),
      Dots if second.is(Whitespace) && third.is_some() => Some(ListMarker::Dot(token.len() as u8)),
      Dashes if second.is(Whitespace) && token.len() == 1 && third.is_some() => {
        Some(ListMarker::Dash)
      }
      Star if second.is(Star) => {
        let src = self.reassemble_src();
        let Some(captures) = REPEAT_STAR_LI_START.captures(&src) else {
          return None;
        };
        Some(ListMarker::Star(captures.get(1).unwrap().len() as u8))
      }
      CalloutNumber if token.lexeme.as_bytes()[1] != b'!' => {
        Some(ListMarker::Callout(token.parse_callout_num()))
      }
      Digits if second.is(Dots) && third.is(Whitespace) => {
        Some(ListMarker::Digits(token.lexeme.parse().unwrap()))
      }
      _ => {
        for token in self.tokens().skip(offset) {
          if token.is(TermDelimiter) {
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
      Some(OpenBracket) => !self.is_attr_list(),
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
    if !self.starts(OpenBracket) || !self.has_seq_at(&[CloseBracket, Whitespace], 2) {
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

  pub fn extract_line_before(&mut self, seq: &[TokenKind]) -> Line<'arena> {
    let mut tokens = Deq::with_capacity(self.tokens.bump, self.num_tokens());
    while !self.starts_with_seq(seq) {
      tokens.push(self.consume_current().unwrap());
    }
    Line::new(tokens)
  }

  pub fn is_partially_consumed(&self) -> bool {
    self.tokens.len() < self.orig_len
  }

  pub fn is_fully_unconsumed(&self) -> bool {
    self.tokens.len() == self.orig_len
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
}

lazy_static! {
  pub static ref REPEAT_STAR_LI_START: Regex = Regex::new(r#"^\s?(\*+)\s+.+"#).unwrap();
}

#[cfg(test)]
mod tests {
  use crate::internal::*;
  use crate::token::{TokenKind::*, *};
  use bumpalo::Bump;
  use test_utils::*;

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
    let bump = &Bump::new();
    for (input, expected) in cases {
      let mut lexer = Lexer::from_str(bump, input);
      let line = lexer.consume_line().unwrap();
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
    let bump = &Bump::new();
    for (input, markers, expected) in cases {
      let mut stack = ListStack::default();
      for marker in markers {
        stack.push(*marker);
      }
      let mut lexer = Lexer::from_str(bump, input);
      let line = lexer.consume_line().unwrap();
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
    let bump = &Bump::new();
    for (input, marker) in cases {
      let mut lexer = Lexer::from_str(bump, input);
      let line = lexer.consume_line().unwrap();
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
    ];
    let bump = &Bump::new();
    for (input, expected) in cases {
      let mut lexer = Lexer::from_str(bump, input);
      let line = lexer.consume_line().unwrap();
      expect_eq!(line.starts_list_item(), expected, from: input);
    }
  }

  #[test]
  fn test_discard() {
    let bump = &Bump::new();
    let mut lexer = Lexer::from_str(bump, "foo bar\nso baz\n");
    let mut line = lexer.consume_line().unwrap();
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
    let bump = &Bump::new();
    let mut lexer = Lexer::from_str(bump, "'foo'");
    let mut line = lexer.consume_line().unwrap();
    expect_eq!(line.reassemble_src(), "'foo'");
    line.discard_last();
    expect_eq!(line.reassemble_src(), "'foo");
    line.discard_last();
    expect_eq!(line.reassemble_src(), "'");
  }

  #[test]
  fn test_line_has_seq_at() {
    let cases: Vec<(&str, &[TokenKind], u32, bool)> = vec![
      ("foo bar_:", &[Word, Whitespace], 0, true),
      ("foo bar_:", &[Word, Whitespace], 1, false),
      ("foo bar", &[Whitespace, Word], 1, true),
      ("foo bar_:", &[Word, Underscore, Colon], 2, true),
      ("foo bar_:", &[Word, Underscore, Colon], 0, false),
      ("#", &[Hash], 0, true),
    ];
    let bump = &Bump::new();
    for (input, token_types, pos, expected) in cases {
      let mut lexer = Lexer::from_str(bump, input);
      let line = lexer.consume_line().unwrap();
      expect_eq!(line.has_seq_at(token_types, pos), expected);
    }

    // test that it works after shifting elements off of the front
    let mut lexer = Lexer::from_str(bump, "foo_#");
    let mut line = lexer.consume_line().unwrap();
    line.discard(2); // `foo` and `_`
    assert!(line.has_seq_at(&[Hash], 0));
  }

  #[test]
  fn test_ends_nonescaped() {
    let cases: Vec<(&str, TokenKind, bool)> = vec![
      ("x", CloseBracket, false),
      ("]", CloseBracket, true),
      ("\\]", CloseBracket, false),
      ("l]", CloseBracket, true),
    ];
    let bump = &Bump::new();
    for (input, token_type, expected) in cases {
      let mut lexer = Lexer::from_str(bump, input);
      let line = lexer.consume_line().unwrap();
      expect_eq!(line.ends_with_nonescaped(token_type), expected);
    }
  }

  #[test]
  fn test_line_contains_seq() {
    let cases: Vec<(&str, &[TokenKind], bool)> = vec![
      ("_bar__r", &[Underscore, Underscore], true),
      ("foo bar_:", &[Word, Whitespace], true),
      ("foo bar_:", &[Word, Whitespace, Word], true),
      ("foo bar_:", &[Word], true),
      ("foo bar_:", &[], false),
      ("foo bar_:", &[Underscore, Colon], true),
      ("foo bar_:", &[Underscore, Word], false),
      ("foo bar_:", &[Whitespace, Word, Underscore], true),
      ("foo ", &[Word, Whitespace, Underscore, Colon], false),
    ];
    let bump = &Bump::new();
    for (input, token_types, expected) in cases {
      let mut lexer = Lexer::from_str(bump, input);
      let line = lexer.consume_line().unwrap();
      expect_eq!(line.contains_seq(token_types), expected);
    }
  }
}
