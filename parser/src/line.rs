use bumpalo::collections::{String, Vec};
use bumpalo::Bump;

use crate::token::{Token, TokenIs, TokenKind, TokenKind::*};

#[derive(Debug)]
pub struct Line<'alloc, 'src> {
  pub src: &'src str,
  all_tokens: Vec<'alloc, Token<'src>>,
  pos: usize,
}

impl<'alloc, 'src> Line<'alloc, 'src> {
  pub fn new(tokens: Vec<'alloc, Token<'src>>, src: &'src str) -> Self {
    Line { all_tokens: tokens, src, pos: 0 }
  }

  pub fn current_token(&self) -> Option<&Token<'src>> {
    self.all_tokens.get(self.pos)
  }

  pub fn last_token(&self) -> Option<&Token<'src>> {
    self.all_tokens.last()
  }

  pub fn nth_token(&self, n: usize) -> Option<&Token> {
    self.all_tokens.get(self.pos + n)
  }

  pub fn current_is(&self, kind: TokenKind) -> bool {
    self.current_token().map_or(false, |t| t.kind == kind)
  }

  pub fn is_empty(&self) -> bool {
    self.pos >= self.all_tokens.len()
  }

  pub fn discard(&mut self, n: usize) {
    for _ in 0..n {
      _ = self.consume_current();
    }
  }

  pub fn contains_nonescaped(&self, token_type: TokenKind) -> bool {
    self.first_nonescaped(token_type).is_some()
  }

  fn tokens(&self) -> impl ExactSizeIterator<Item = &Token<'src>> {
    self.all_tokens.iter().skip(self.pos)
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
    self.is_continuous_thru(OpenBracket) && self.contains_nonescaped(CloseBracket)
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
      Some(n) if n == 0 => false,
      Some(n) if self.nth_token(n).is_not(Word) => true,
      _ => false,
    }
  }

  pub fn consume_to_string_until(
    &mut self,
    kind: TokenKind,
    allocator: &'alloc Bump,
  ) -> String<'alloc> {
    let mut s = String::new_in(allocator);
    while let Some(token) = self.consume_if_not(kind) {
      s.push_str(token.lexeme);
    }
    s
  }

  pub fn consume_if_not(&mut self, kind: TokenKind) -> Option<Token> {
    match self.current_token() {
      Some(token) if !token.is(kind) => self.consume_current(),
      _ => None,
    }
  }

  pub fn consume_macro_target(&mut self, allocator: &'alloc Bump) -> String<'alloc> {
    let target = self.consume_to_string_until(OpenBracket, allocator);
    self.discard(1); // `[`
    target
  }

  pub fn consume_optional_macro_target(
    &mut self,
    allocator: &'alloc Bump,
  ) -> Option<String<'alloc>> {
    let target = match self.current_is(OpenBracket) {
      true => None,
      false => Some(self.consume_to_string_until(CloseBracket, allocator)),
    };
    self.discard(1); // `[`
    target
  }

  #[must_use]
  pub fn consume_url(&mut self, start: Option<&Token>, allocator: &'alloc Bump) -> String<'alloc> {
    let mut num_tokens = 0;

    for token in self.tokens() {
      match token.kind {
        Whitespace => break,
        GreaterThan => break,
        _ => num_tokens += 1,
      }
    }

    if num_tokens > 0 && self.all_tokens.get(self.pos + num_tokens - 1).is(Dot) {
      num_tokens -= 1;
    }

    let mut s = String::new_in(allocator);
    if let Some(start) = start {
      s.push_str(start.lexeme);
    }
    for _ in 0..num_tokens {
      s.push_str(self.consume_current().unwrap().lexeme);
    }
    s
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
}

#[cfg(test)]
mod tests {
  use crate::lexer::Lexer;
  use crate::token::{TokenKind::*, *};
  use bumpalo::Bump;

  #[test]
  fn test_discard() {
    let bump = &Bump::new();
    let mut lexer = Lexer::new("foo bar\nso baz\n");
    let mut line = lexer.consume_line(bump).unwrap();
    assert_eq!(line.src, "foo bar");
    line.discard(1);
    assert_eq!(line.src, " bar");
    line.discard(2);
    assert_eq!(line.src, "");
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
