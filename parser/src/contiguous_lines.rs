use crate::internal::*;

#[derive(Debug, Clone)]
pub struct ContiguousLines<'bmp, 'src> {
  // NB: lines kept in reverse, as there is no VeqDeque in bumpalo
  // and we almost always want to consume from the front, so fake it
  lines: Vec<'bmp, Line<'bmp, 'src>>,
}

impl<'bmp, 'src> ContiguousLines<'bmp, 'src> {
  pub fn new(mut lines: Vec<'bmp, Line<'bmp, 'src>>) -> Self {
    lines.reverse();
    ContiguousLines { lines }
  }

  pub fn current(&self) -> Option<&Line<'bmp, 'src>> {
    self.lines.last()
  }

  pub fn current_token(&self) -> Option<&Token<'src>> {
    self.current().and_then(|line| line.current_token())
  }

  pub fn is_empty(&self) -> bool {
    self.lines.is_empty()
  }

  pub fn consume_current(&mut self) -> Option<Line<'bmp, 'src>> {
    self.lines.pop()
  }

  pub fn consume_current_token(&mut self) -> Option<Token<'src>> {
    self
      .consume_current()
      .and_then(|mut line| line.consume_current())
  }

  pub fn restore(&mut self, line: Line<'bmp, 'src>) {
    if !line.is_empty() {
      self.lines.push(line);
    }
  }

  pub fn retain(&mut self, f: impl FnMut(&Line<'bmp, 'src>) -> bool) {
    self.lines.retain(f);
  }

  pub fn any(&self, f: impl FnMut(&Line<'bmp, 'src>) -> bool) -> bool {
    self.lines.iter().any(f)
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
    self.lines.len() == 1 && self.current().unwrap().is_block_macro()
  }

  pub fn current_starts_with(&self, kind: TokenKind) -> bool {
    match self.current() {
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
