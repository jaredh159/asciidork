use bumpalo::collections::Vec as BumpVec;

use crate::ast::SourceLocation;
use crate::{line::Line, token::*};

#[derive(Debug)]
pub struct Block<'bmp, 'src> {
  // NB: lines kept in reverse, as there is no VeqDeque in bumpalo
  // and we almost always want to consume from the front, so fake it
  pub(crate) lines: BumpVec<'bmp, Line<'bmp, 'src>>,
}

impl<'bmp, 'src> Block<'bmp, 'src> {
  pub fn new(mut lines: BumpVec<'bmp, Line<'bmp, 'src>>) -> Self {
    lines.reverse();
    Block { lines }
  }

  pub fn current_line(&self) -> Option<&Line<'bmp, 'src>> {
    self.lines.last()
  }

  pub fn current_token(&self) -> Option<&Token<'src>> {
    self.current_line().and_then(|line| line.current_token())
  }

  pub fn is_empty(&self) -> bool {
    self.lines.is_empty()
  }

  pub fn consume_current(&mut self) -> Option<Line<'bmp, 'src>> {
    self.lines.pop()
  }

  pub fn restore(&mut self, line: Line<'bmp, 'src>) {
    self.lines.push(line);
  }

  pub fn contains_seq(&self, kinds: &[TokenKind]) -> bool {
    self.lines.iter().any(|line| line.contains_seq(kinds))
  }

  pub fn terminates_constrained(&self, stop_tokens: &[TokenKind]) -> bool {
    self
      .lines
      .iter()
      .any(|line| line.terminates_constrained(stop_tokens))
  }

  pub fn is_block_macro(&self) -> bool {
    self.lines.len() == 1 && self.current_line().unwrap().is_block_macro()
  }

  pub fn current_line_starts_with(&self, kind: TokenKind) -> bool {
    match self.current_line() {
      Some(line) => line
        .current_token()
        .map(|token| token.is(kind))
        .unwrap_or(false),
      None => false,
    }
  }

  pub fn location(&self) -> Option<SourceLocation> {
    if let Some(line) = self.lines.last() {
      line.location()
    } else {
      None
    }
  }
}
