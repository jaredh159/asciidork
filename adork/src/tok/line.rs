use std::collections::VecDeque;

use crate::parse::Parser;
use crate::tok::{token::TokenIs, Token, TokenType, TokenType::*};

#[derive(Debug, PartialEq, Eq)]
pub struct Line {
  pub tokens: VecDeque<Token>,
}

impl Iterator for Line {
  type Item = Token;

  fn next(&mut self) -> Option<Self::Item> {
    self.tokens.pop_front()
  }
}

impl Line {
  pub fn new(tokens: Vec<Token>) -> Line {
    Line { tokens: tokens.into() }
  }

  pub fn is_empty(&self) -> bool {
    self.tokens.len() == 0
  }

  /// true if there is no whitespace until token type, and token type is found
  pub fn is_continuous_thru(&self, token_type: TokenType) -> bool {
    for token in &self.tokens {
      if token.is(token_type) {
        return true;
      } else if token.is(Whitespace) {
        return false;
      } else {
        continue;
      }
    }
    false
  }

  pub fn extract_until(&mut self, token_type: TokenType) -> Line {
    let mut tokens = VecDeque::new();
    while let Some(token) = self.consume_current() {
      if token.is(token_type) {
        self.tokens.push_front(token);
        break;
      } else {
        tokens.push_back(token);
      }
    }
    Line::new(tokens.into())
  }

  pub fn remove_all(&mut self, token_type: TokenType) {
    self.tokens.retain(|token| !token.is(token_type))
  }

  pub fn current_token(&self) -> Option<&Token> {
    self.tokens.front()
  }

  #[must_use]
  pub fn consume<const N: usize>(&mut self) -> [Option<Token>; N] {
    std::array::from_fn(|_| self.consume_current())
  }

  pub fn consume_current(&mut self) -> Option<Token> {
    self.tokens.pop_front()
  }

  pub fn consume_if(&mut self, token_type: TokenType) -> Option<Token> {
    match self.current_token() {
      Some(token) if token.is(token_type) => self.consume_current(),
      _ => None,
    }
  }

  pub fn consume_if_not(&mut self, token_type: TokenType) -> Option<Token> {
    match self.current_token() {
      Some(token) if !token.is(token_type) => self.consume_current(),
      _ => None,
    }
  }

  pub fn discard(&mut self, n: usize) {
    for _ in 0..n {
      self.consume_current();
    }
  }

  pub fn discard_until(&mut self, token_type: TokenType) {
    while let Some(token) = self.current_token() {
      if token.is(token_type) {
        break;
      }
      self.consume_current();
    }
  }

  pub fn discard_until_one_of(&mut self, token_types: &[TokenType]) {
    while let Some(token) = self.current_token() {
      if token_types.contains(&token.token_type) {
        break;
      }
      self.consume_current();
    }
  }

  pub fn discard_leading_whitespace(&mut self) {
    while let Some(token) = self.current_token() {
      if token.is(Whitespace) {
        self.consume_current();
      } else {
        break;
      }
    }
  }
  pub fn peek_token_is(&self, token_type: TokenType) -> bool {
    match self.peek_token() {
      Some(token) => token.is(token_type),
      None => false,
    }
  }

  pub fn peek_token(&self) -> Option<&Token> {
    self.tokens.get(1)
  }

  pub fn last_token(&self) -> Option<&Token> {
    self.tokens.back()
  }

  pub fn nth_token(&self, n: usize) -> Option<&Token> {
    self.tokens.get(n)
  }

  pub fn nth_token_is(&self, n: usize, token_type: TokenType) -> bool {
    match self.tokens.get(n) {
      Some(token) => token.is(token_type),
      None => false,
    }
  }

  pub fn nth_token_one_of(&self, n: usize, token_types: &[TokenType]) -> bool {
    match self.tokens.get(n) {
      Some(token) => token_types.contains(&token.token_type),
      None => false,
    }
  }

  pub fn current_is(&self, token_type: TokenType) -> bool {
    self.starts(token_type)
  }

  pub fn starts(&self, token_type: TokenType) -> bool {
    if self.tokens.is_empty() {
      return false;
    }
    self.tokens[0].token_type == token_type
  }

  pub fn contains(&self, token_type: TokenType) -> bool {
    for token in &self.tokens {
      if token.token_type == token_type {
        return true;
      }
    }
    false
  }

  pub fn contains_any(&self, token_types: &[TokenType]) -> bool {
    for token in &self.tokens {
      for token_type in token_types {
        if token.token_type == *token_type {
          return true;
        }
      }
    }
    false
  }

  pub fn contains_seq(&self, token_types: &[TokenType]) -> bool {
    if self.tokens.len() < token_types.len() {
      return false;
    }
    let Some(first_type) = token_types.first() else {
      return false;
    };
    'outer: for (i, token) in self.tokens.iter().enumerate() {
      if token.token_type == *first_type {
        if self.tokens.len() - i < token_types.len() {
          return false;
        }
        for (j, token_type) in token_types.iter().skip(1).enumerate() {
          if self.tokens[i + j + 1].token_type != *token_type {
            continue 'outer;
          }
        }
        return true;
      }
    }
    false
  }

  pub fn ends_constrained_inline(&self, token_type: TokenType) -> bool {
    if self.tokens.is_empty() {
      return false;
    }
    for (i, token) in self.tokens.iter().enumerate() {
      if token.token_type == token_type {
        return match self.tokens.get(i + 1) {
          Some(token) if !token.is(Word) => true,
          None => true,
          _ => false,
        };
      }
    }
    false
  }

  pub fn starts_with_one_of(&self, token_types: &[TokenType]) -> bool {
    for token_type in token_types {
      if self.starts(*token_type) {
        return true;
      }
    }
    false
  }

  pub fn starts_with_seq(&self, token_types: &[TokenType]) -> bool {
    if token_types.is_empty() {
      return false;
    }
    if self.tokens.len() < token_types.len() {
      return false;
    }
    for (i, token_type) in token_types.iter().enumerate() {
      if self.tokens[i].token_type != *token_type {
        return false;
      }
    }
    true
  }

  pub fn is_header(&self, len: usize) -> bool {
    if !self.starts_with_seq(&[EqualSigns, Whitespace]) {
      return false;
    }
    self.current_token().unwrap().len() == len
  }

  pub fn consume_to_string(&mut self, parser: &Parser) -> String {
    let mut s = String::with_capacity(self.len());
    while let Some(token) = self.consume_current() {
      s.push_str(parser.lexeme_str(&token));
    }
    s
  }

  pub fn consume_to_string_until(&mut self, token_type: TokenType, parser: &Parser) -> String {
    let mut s = String::new();
    while let Some(token) = self.consume_if_not(token_type) {
      s.push_str(parser.lexeme_str(&token));
    }
    s
  }

  pub fn len(&self) -> usize {
    let end = self.tokens.back().map(|token| token.end).unwrap_or(0);
    let start = self.tokens.front().map(|token| token.start).unwrap_or(0);
    end.saturating_sub(start)
  }

  pub fn print(&self, parser: &Parser) {
    print!("Line({}): `", self.tokens.len());
    for token in &self.tokens {
      print!("{}", parser.lexeme_str(token));
    }
    println!("`");
  }

  pub fn print_with(&self, prefix: &str, parser: &Parser) {
    print!("{} ", prefix);
    self.print(parser);
  }

  pub fn first_nonescaped(&self, token_type: TokenType) -> Option<&Token> {
    let mut prev_type = None;
    for token in &self.tokens {
      if token.is(token_type) && prev_type.is_not(Backslash) {
        return Some(token);
      }
      prev_type = Some(token);
    }
    None
  }

  fn clump_at<'a>(
    &'a self,
    starting_token_index: usize,
    stop_tokens: &[TokenType],
    parser: &'a Parser,
  ) -> Option<(Clump, usize)> {
    let Some(first_token) = self.tokens.get(starting_token_index) else {
        return None;
      };
    debug_assert!(first_token.token_type != Whitespace);
    let start = first_token.start;
    let mut end = first_token.end;
    let mut token_index = starting_token_index + 1;
    let mut num_tokens = 1;
    loop {
      match self.tokens.get(token_index) {
        Some(token) if stop_tokens.contains(&token.token_type) => {
          token_index += 1;
          break;
        }
        Some(token) if token.token_type == Whitespace => {
          token_index += 1;
          break;
        }
        Some(token) => {
          end = token.end;
          token_index += 1;
          num_tokens += 1;
        }
        None => break,
      }
    }

    Some((
      Clump::new(
        parser.get_str(start, end),
        starting_token_index,
        starting_token_index + num_tokens,
      ),
      token_index,
    ))
  }

  pub(crate) fn clump_until<'a>(
    &'a self,
    stop_tokens: &[TokenType],
    parser: &'a Parser,
  ) -> Option<Clump> {
    if let Some((clump, _)) = self.clump_at(0, stop_tokens, parser) {
      Some(clump)
    } else {
      None
    }
  }

  pub(crate) fn clumps<'a>(&'a self, parser: &'a Parser) -> Vec<Clump> {
    let mut token_index = 0;

    // skip leading whitespace
    loop {
      match self.tokens.get(token_index) {
        Some(token) if token.token_type == Whitespace => token_index += 1,
        _ => break,
      }
    }

    let mut clumps = Vec::new();
    while let Some((clump, next_start)) = self.clump_at(token_index, &[], parser) {
      clumps.push(clump);
      token_index = next_start;
    }

    clumps
  }

  pub(crate) fn discard_clump(&mut self, clump: &Clump) {
    self.tokens.drain(clump.start..clump.end);
  }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Clump<'a> {
  pub str: &'a str,
  pub start: usize,
  pub end: usize,
}

impl<'a> Clump<'a> {
  fn new(str: &'a str, start: usize, end: usize) -> Self {
    Self { str, start, end }
  }

  pub fn starts_with(&self, c: char) -> bool {
    self.str.starts_with(c)
  }

  pub fn ends_with(&self, c: char) -> bool {
    self.str.ends_with(c)
  }

  pub fn string(&self) -> String {
    self.str.to_string()
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::t::*;

  #[test]
  fn test_clump_until() {
    let cases: Vec<(&str, &[TokenType], Clump<'static>)> = vec![
      ("foo bar", &[Comma], Clump::new("foo", 0, 1)),
      ("foo; bar", &[Comma], Clump::new("foo;", 0, 2)),
      ("foo, bar", &[Comma], Clump::new("foo", 0, 1)),
      ("foo; bar", &[Comma, SemiColon], Clump::new("foo", 0, 1)),
    ];
    for (input, stop, expected) in cases {
      let (line, parser) = line_test(input);
      assert_eq!(line.clump_until(stop, &parser).unwrap(), expected);
    }
  }

  #[test]
  fn test_clumps() {
    let cases: Vec<(&str, Vec<Clump<'static>>)> = vec![
      (
        "foo bar",
        vec![Clump::new("foo", 0, 1), Clump::new("bar", 2, 3)],
      ),
      (
        "foo b.r",
        vec![Clump::new("foo", 0, 1), Clump::new("b.r", 2, 5)],
      ),
      (
        " foo bar",
        vec![Clump::new("foo", 1, 2), Clump::new("bar", 3, 4)],
      ),
      (" ", vec![]),
    ];
    for (input, expected) in cases {
      let (line, parser) = line_test(input);
      assert_eq!(line.clumps(&parser), expected);
    }
  }

  #[test]
  fn test_line_contains_seq() {
    let cases: Vec<(&str, &[TokenType], bool)> = vec![
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
    for (input, token_types, expected) in cases {
      let (line, _) = line_test(input);
      assert_eq!(line.contains_seq(token_types), expected);
    }
  }
}
