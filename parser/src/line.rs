use crate::prelude::*;
use crate::variants::token::*;

#[derive(Debug, Clone)]
pub struct Line<'bmp, 'src> {
  pub src: &'src str,
  all_tokens: Vec<'bmp, Token<'src>>,
  pos: usize,
}

impl<'bmp, 'src> Line<'bmp, 'src> {
  pub fn new(tokens: Vec<'bmp, Token<'src>>, src: &'src str) -> Self {
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

  pub fn current_is(&self, kind: TokenKind) -> bool {
    self.current_token().map_or(false, |t| t.kind == kind)
  }

  pub fn is_empty(&self) -> bool {
    self.pos >= self.all_tokens.len()
  }

  pub fn is_header(&self, len: usize) -> bool {
    if !self.starts_with_seq(&[EqualSigns, Whitespace]) {
      return false;
    }
    self.current_token().unwrap().lexeme.len() == len
  }

  pub fn is_block_macro(&self) -> bool {
    self.starts_with_seq(&[MacroName, Colon])
      && self.contains(OpenBracket)
      && self.ends_with_nonescaped(CloseBracket)
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
      Some(n) if n == 0 => false,
      Some(n) if self.nth_token(n).is_not(Word) => true,
      _ => false,
    }
  }

  pub fn consume_to_string_until(
    &mut self,
    kind: TokenKind,
    bump: &'bmp Bump,
  ) -> SourceString<'bmp> {
    let mut loc = self.location().expect("no tokens to consume");
    let mut s = String::new_in(bump);
    while let Some(token) = self.consume_if_not(kind) {
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

  pub fn consume_macro_target(&mut self, bump: &'bmp Bump) -> SourceString<'bmp> {
    let target = self.consume_to_string_until(OpenBracket, bump);
    debug_assert!(self.current_is(OpenBracket));
    self.discard(1); // `[`
    target
  }

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
    let mut loc = start.map_or_else(|| self.location().unwrap(), |t| t.loc);
    let mut num_tokens = 0;

    for token in self.tokens() {
      match token.kind {
        Whitespace | GreaterThan | OpenBracket => break,
        _ => num_tokens += 1,
      }
    }

    if num_tokens > 0 && self.all_tokens.get(self.pos + num_tokens - 1).is(Dot) {
      num_tokens -= 1;
    }

    let mut s = String::new_in(bump);
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

  pub fn location(&self) -> Option<SourceLocation> {
    self.current_token().map(|t| t.loc)
  }

  pub fn last_location(&self) -> Option<SourceLocation> {
    self.last_token().map(|t| t.loc)
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
