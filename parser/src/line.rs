use lazy_static::lazy_static;
use regex::Regex;

use crate::internal::*;
use crate::variants::token::*;

#[derive(Debug, Clone)]
pub struct Line<'bmp, 'src> {
  pub src: &'src str,
  all_tokens: BumpVec<'bmp, Token<'src>>,
  pos: usize,
}

impl<'bmp, 'src> Line<'bmp, 'src> {
  pub fn new(tokens: BumpVec<'bmp, Token<'src>>, src: &'src str) -> Self {
    Line { all_tokens: tokens, src, pos: 0 }
  }

  pub fn current_token(&self) -> Option<&Token<'src>> {
    self.all_tokens.get(self.pos)
  }

  pub fn peek_token(&self) -> Option<&Token<'src>> {
    self.nth_token(1)
  }

  pub fn last_token(&self) -> Option<&Token<'src>> {
    if self.is_empty() {
      return None;
    }
    self.all_tokens.last()
  }

  pub fn nth_token(&self, n: usize) -> Option<&Token<'src>> {
    self.all_tokens.get(self.pos + n)
  }

  pub fn num_tokens(&self) -> usize {
    self.all_tokens.len() - self.pos
  }

  pub fn current_is(&self, kind: TokenKind) -> bool {
    self.current_token().map_or(false, |t| t.kind == kind)
  }

  pub fn is_empty(&self) -> bool {
    self.pos >= self.all_tokens.len()
  }

  pub fn is_header(&self, level: u8) -> bool {
    self.header_level() == Some(level)
  }

  pub fn header_level(&self) -> Option<u8> {
    if self.starts_with_seq(&[EqualSigns, Whitespace]) {
      Some((self.current_token().unwrap().lexeme.len() - 1) as u8)
    } else {
      None
    }
  }

  pub fn is_block_macro(&self) -> bool {
    self.starts_with_seq(&[MacroName, Colon])
      && self.contains(OpenBracket)
      && self.ends_with_nonescaped(CloseBracket)
  }

  pub fn is_attr_list(&self) -> bool {
    self.starts(OpenBracket) && self.ends_with_nonescaped(CloseBracket)
  }

  pub fn is_block_title(&self) -> bool {
    // dot followed by at least one non-whitespace token
    self.starts(Dots) && self.tokens().len() > 1 && self.peek_token().unwrap().is_not(Whitespace)
  }

  pub fn is_delimiter(&self, delimiter: Delimiter) -> bool {
    self.num_tokens() == 1 && self.current_token().unwrap().to_delimeter() == Some(delimiter)
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

  pub fn discard_last(&mut self) -> Option<Token<'src>> {
    let Some(token) = self.all_tokens.pop() else {
      return None;
    };
    self.src = &self.src[..self.src.len() - token.lexeme.len()];
    Some(token)
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

  fn tokens(&self) -> impl ExactSizeIterator<Item = &Token<'src>> {
    self.all_tokens.iter().skip(self.pos)
  }

  fn tokens_mut(&mut self) -> impl ExactSizeIterator<Item = &mut Token<'src>> {
    self.all_tokens.iter_mut().skip(self.pos)
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

  pub fn has_seq_at(&self, kinds: &[TokenKind], offset: usize) -> bool {
    if kinds.is_empty() || self.tokens().len() < offset + kinds.len() {
      return false;
    }
    for (i, token_type) in kinds.iter().enumerate() {
      if self.all_tokens[i + self.pos + offset].kind != *token_type {
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
          if self.all_tokens[self.pos + i + j + 1].kind != *kind {
            continue 'outer;
          }
        }
        return Some(i);
      }
    }
    None
  }

  pub fn continues_inline_macro(&self) -> bool {
    !self.current_is(Colon)
      && self.is_continuous_thru(OpenBracket)
      && self.contains_nonescaped(CloseBracket)
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
    bump: &'bmp Bump,
  ) -> SourceString<'bmp> {
    let mut loc = self.loc().expect("no tokens to consume");
    let mut s = BumpString::new_in(bump);
    while let Some(token) = self.consume_if_not(kind) {
      s.push_str(token.lexeme);
      loc.extend(token.loc);
    }
    SourceString::new(s, loc)
  }

  #[must_use]
  pub fn consume_to_string(&mut self, bump: &'bmp Bump) -> SourceString<'bmp> {
    let mut loc = self.loc().expect("no tokens to consume");
    let mut s = BumpString::new_in(bump);
    while let Some(token) = self.consume_current() {
      s.push_str(token.lexeme);
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
  pub fn consume_macro_target(&mut self, bump: &'bmp Bump) -> SourceString<'bmp> {
    let target = self.consume_to_string_until(OpenBracket, bump);
    debug_assert!(self.current_is(OpenBracket));
    self.discard(1); // `[`
    target
  }

  #[must_use]
  pub fn consume_optional_macro_target(&mut self, bump: &'bmp Bump) -> Option<SourceString<'bmp>> {
    let target = match self.current_is(OpenBracket) {
      true => None,
      false => Some(self.consume_to_string_until(OpenBracket, bump)),
    };
    debug_assert!(self.current_is(OpenBracket));
    self.discard(1); // `[`
    target
  }

  #[must_use]
  pub fn consume_url(&mut self, start: Option<&Token>, bump: &'bmp Bump) -> SourceString<'bmp> {
    let mut loc = start.map_or_else(|| self.loc().unwrap(), |t| t.loc);
    let mut num_tokens = 0;

    for token in self.tokens() {
      match token.kind {
        Whitespace | GreaterThan | OpenBracket => break,
        _ => num_tokens += 1,
      }
    }

    if num_tokens > 0 && self.all_tokens.get(self.pos + num_tokens - 1).is(Dots) {
      num_tokens -= 1;
    }

    let mut s = BumpString::new_in(bump);
    if let Some(start) = start {
      s.push_str(start.lexeme);
      loc.extend(start.loc);
    }
    for _ in 0..num_tokens {
      let token = self.consume_current().unwrap();
      s.push_str(token.lexeme);
      loc.extend(token.loc);
    }
    SourceString::new(s, loc)
  }

  #[must_use]
  pub fn consume_current(&mut self) -> Option<Token<'src>> {
    if self.is_empty() {
      return None;
    }
    let token = std::mem::take(&mut self.all_tokens[self.pos]);
    self.pos += 1;
    self.src = &self.src[token.lexeme.len()..];
    Some(token)
  }

  pub fn into_lines_in(self, bump: &'bmp Bump) -> ContiguousLines<'bmp, 'src> {
    ContiguousLines::new(bvec![in bump; self])
  }

  pub fn loc(&self) -> Option<SourceLocation> {
    self.current_token().map(|t| t.loc)
  }

  pub fn last_location(&self) -> Option<SourceLocation> {
    self.last_token().map(|t| t.loc)
  }

  pub fn list_marker(&self) -> Option<ListMarker> {
    // PERF: checking for list markers seems sort of sad, wonder if the
    // Line could be created with some markers to speed these tests up
    let mut offset = 0;
    if self.current_token().is(Whitespace) {
      offset += 1;
    }
    let token = self.nth_token(offset).unwrap();
    let next = self.nth_token(offset + 1);

    match token.kind {
      Star if next.is(Whitespace) => Some(ListMarker::Star(1)),
      Dots if next.is(Whitespace) => Some(ListMarker::Dot(token.len() as u8)),
      Dashes if next.is(Whitespace) && token.len() == 1 => Some(ListMarker::Dash),
      Star if next.is(Star) => {
        let Some(captures) = REPEAT_STAR_LI_START.captures(self.src) else {
          return None;
        };
        Some(ListMarker::Star(captures.get(1).unwrap().len() as u8))
      }
      Digits if next.is(Dots) && self.nth_token(offset + 2).is(Whitespace) => {
        Some(ListMarker::Digits(token.lexeme.parse().unwrap()))
      }
      _ => {
        for token in self.tokens().skip(offset) {
          if token.is(TermDelimiter) {
            return match token.lexeme {
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
    match self.current_token().map(|t| t.kind) {
      Some(OpenBracket) => !self.is_attr_list(),
      Some(Plus) | Some(CommentLine) => false,
      None => false,
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
      self.all_tokens[self.pos].kind = Discard;
    }
  }

  pub fn starts_nested_list(&self, stack: &ListStack) -> bool {
    self
      .list_marker()
      .map(|marker| stack.starts_nested_list(marker))
      .unwrap_or(false)
  }

  pub fn consume_checklist_item(&mut self, bump: &'bmp Bump) -> Option<(bool, SourceString<'bmp>)> {
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

  pub fn extract_line_before(&mut self, kind: TokenKind, bump: &'bmp Bump) -> Line<'bmp, 'src> {
    let mut tokens = BumpVec::with_capacity_in(self.num_tokens(), bump);
    let orig_src = self.src;
    let mut src_len = 0;
    while self.current_token().is_not(kind) {
      let token = self.consume_current().unwrap();
      src_len += token.lexeme.len();
      tokens.push(token);
    }
    Line::new(tokens, &orig_src[..src_len])
  }
}

lazy_static! {
  pub static ref REPEAT_STAR_LI_START: Regex = Regex::new(r#"^\s?(\*+)\s+.+"#).unwrap();
}

#[cfg(test)]
mod tests {
  use crate::internal::*;
  use crate::lexer::Lexer;
  use crate::token::{TokenKind::*, *};
  use bumpalo::Bump;
  use test_utils::assert_eq;

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
      let mut lexer = Lexer::new(input);
      let line = lexer.consume_line(bump).unwrap();
      assert_eq!(line.continues_list_item_principle(), expected, from: input);
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
    ];
    let bump = &Bump::new();
    for (input, markers, expected) in cases {
      let mut stack = ListStack::default();
      for marker in markers {
        stack.push(*marker);
      }
      let mut lexer = Lexer::new(input);
      let line = lexer.consume_line(bump).unwrap();
      assert_eq!(line.starts_nested_list(&stack), expected, from: input);
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
      ("foo:: bar", Some(Colons(2))),
      ("foo::", Some(Colons(2))),
      ("image:: baz", Some(Colons(2))),
      ("image::cat.png[]", None),
      ("foo::: bar", Some(Colons(3))),
      ("foo:::: bar", Some(Colons(4))),
      ("foo;; bar", Some(SemiColons)),
      ("_foo_::", Some(Colons(2))),
      ("foo bar:: baz", Some(Colons(2))),
    ];
    let bump = &Bump::new();
    for (input, marker) in cases {
      let mut lexer = Lexer::new(input);
      let line = lexer.consume_line(bump).unwrap();
      assert_eq!(line.list_marker(), marker, from: input);
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
      let mut lexer = Lexer::new(input);
      let line = lexer.consume_line(bump).unwrap();
      assert_eq!(line.starts_list_item(), expected, from: input);
    }
  }

  #[test]
  fn test_discard() {
    let bump = &Bump::new();
    let mut lexer = Lexer::new("foo bar\nso baz\n");
    let mut line = lexer.consume_line(bump).unwrap();
    assert_eq!(line.src, "foo bar");
    assert_eq!(line.num_tokens(), 3);
    line.discard(1);
    assert_eq!(line.src, " bar");
    assert_eq!(line.num_tokens(), 2);
    line.discard(2);
    assert_eq!(line.src, "");
    assert_eq!(line.num_tokens(), 0);
  }

  #[test]
  fn test_discard_last() {
    let bump = &Bump::new();
    let mut lexer = Lexer::new("'foo'");
    let mut line = lexer.consume_line(bump).unwrap();
    assert_eq!(line.src, "'foo'");
    line.discard_last();
    assert_eq!(line.src, "'foo");
    line.discard_last();
    assert_eq!(line.src, "'");
  }

  #[test]
  fn test_line_has_seq_at() {
    let cases: Vec<(&str, &[TokenKind], usize, bool)> = vec![
      ("foo bar_:", &[Word, Whitespace], 0, true),
      ("foo bar_:", &[Word, Whitespace], 1, false),
      ("foo bar", &[Whitespace, Word], 1, true),
      ("foo bar_:", &[Word, Underscore, Colon], 2, true),
      ("foo bar_:", &[Word, Underscore, Colon], 0, false),
      ("#", &[Hash], 0, true),
    ];
    let bump = &Bump::new();
    for (input, token_types, pos, expected) in cases {
      let mut lexer = Lexer::new(input);
      let line = lexer.consume_line(bump).unwrap();
      assert_eq!(line.has_seq_at(token_types, pos), expected);
    }

    // test that it works after shifting elements off of the front
    let mut lexer = Lexer::new("foo_#");
    let mut line = lexer.consume_line(bump).unwrap();
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
      let mut lexer = Lexer::new(input);
      let line = lexer.consume_line(bump).unwrap();
      assert_eq!(line.ends_with_nonescaped(token_type), expected);
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
      let mut lexer = Lexer::new(input);
      let line = lexer.consume_line(bump).unwrap();
      assert_eq!(line.contains_seq(token_types), expected);
    }
  }
}
