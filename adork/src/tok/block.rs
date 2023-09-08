use std::collections::VecDeque;

use super::line::Line;
use crate::tok::TokenType;

#[derive(Debug)]
pub struct Block {
  pub lines: VecDeque<Line>,
}

impl Iterator for Block {
  type Item = Line;

  fn next(&mut self) -> Option<Self::Item> {
    self.lines.pop_front()
  }
}

impl Block {
  pub fn new(lines: VecDeque<Line>) -> Block {
    assert!(!lines.is_empty());
    Block { lines }
  }

  pub fn remove_all(&mut self, token_type: TokenType) {
    self.lines.retain(|line| !line.starts(token_type));
  }

  pub fn current_line(&self) -> Option<&Line> {
    self.lines.front()
  }

  pub fn current_line_mut(&mut self) -> Option<&mut Line> {
    self.lines.front_mut()
  }

  pub fn restore(&mut self, line: Line) {
    self.lines.push_front(line);
  }

  pub fn is_empty(&self) -> bool {
    self.lines.len() == 0
  }

  pub fn contains_seq(&self, token_types: &[TokenType]) -> bool {
    self.lines.iter().any(|line| line.contains_seq(token_types))
  }

  pub fn ends_constrained_inline(&self, token_type: TokenType) -> bool {
    self
      .lines
      .iter()
      .any(|line| line.ends_constrained_inline(token_type))
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

impl From<Line> for Block {
  fn from(line: Line) -> Self {
    Block::new(vec![line].into())
  }
}
