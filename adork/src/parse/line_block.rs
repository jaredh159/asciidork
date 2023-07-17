use std::collections::VecDeque;

use super::line::Line;
use crate::token::TokenType;

#[derive(Debug)]
pub struct LineBlock {
  pub lines: VecDeque<Line>,
}

impl Iterator for LineBlock {
  type Item = Line;

  fn next(&mut self) -> Option<Self::Item> {
    self.lines.pop_front()
  }
}

impl LineBlock {
  pub fn new(lines: VecDeque<Line>) -> LineBlock {
    assert!(lines.len() > 0);
    LineBlock { lines }
  }

  pub fn remove_all(&mut self, token_type: TokenType) {
    self.lines.retain(|line| !line.starts(token_type));
  }

  pub fn current_line(&self) -> Option<&Line> {
    self.lines.front()
  }

  pub fn is_empty(&self) -> bool {
    self.lines.len() == 0
  }

  pub fn current_line_starts_with(&self, token_type: TokenType) -> bool {
    match self.current_line() {
      Some(line) => line
        .current_token()
        .map(|token| token.is(token_type))
        .unwrap_or(false),
      None => false,
    }
  }

  pub fn current_line_satisfies(&self, predicate: impl Fn(&Line) -> bool) -> bool {
    match self.current_line() {
      Some(line) => predicate(line),
      None => false,
    }
  }

  pub fn consume_current(&mut self) -> Option<Line> {
    self.lines.pop_front()
  }

  pub fn peek_line(&self) -> Option<&Line> {
    self.lines.get(1)
  }

  pub fn last_line(&self) -> Option<&Line> {
    self.lines.back()
  }

  pub fn nth_line(&self, n: usize) -> Option<&Line> {
    self.lines.get(n)
  }
}
