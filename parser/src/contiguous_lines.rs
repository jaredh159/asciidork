use crate::internal::*;

#[derive(Debug, Clone)]
pub struct ContiguousLines<'bmp, 'src> {
  // NB: lines kept in reverse, as there is no VeqDeque in bumpalo
  // and we almost always want to consume from the front, so fake it
  lines: BumpVec<'bmp, Line<'bmp, 'src>>,
}

impl<'bmp, 'src> ContiguousLines<'bmp, 'src> {
  pub fn new(mut lines: BumpVec<'bmp, Line<'bmp, 'src>>) -> Self {
    lines.reverse();
    ContiguousLines { lines }
  }

  pub fn current(&self) -> Option<&Line<'bmp, 'src>> {
    self.lines.last()
  }

  pub fn current_mut(&mut self) -> Option<&mut Line<'bmp, 'src>> {
    self.lines.last_mut()
  }

  pub fn last(&self) -> Option<&Line<'bmp, 'src>> {
    self.lines.first()
  }

  pub fn last_mut(&mut self) -> Option<&mut Line<'bmp, 'src>> {
    self.lines.first_mut()
  }

  pub fn remove_last_unchecked(&mut self) -> Line<'bmp, 'src> {
    self.lines.remove(0)
  }

  pub fn nth(&self, n: usize) -> Option<&Line<'bmp, 'src>> {
    self.lines.get(self.lines.len() - n - 1)
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

  pub fn is_quoted_paragraph(&self) -> bool {
    use TokenKind::*;
    if self.lines.len() < 2 {
      return false;
    }
    let last_line = self.last().unwrap();
    if !last_line.starts_with_seq(&[Word, Whitespace])
      || !last_line.src.starts_with("-- ")
      || last_line.num_tokens() < 3
    {
      return false;
    }
    let first_line = self.current().unwrap();
    if !first_line.starts(DoubleQuote) {
      return false;
    }
    let penult = self.nth(self.lines.len() - 2).unwrap();
    penult.ends(DoubleQuote)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_is_quoted_paragraph() {
    let cases = vec![
      ("\"foo bar\nso baz\"\n-- me", true),
      ("foo bar\nso baz\"\n-- me", false),
      ("\"foo bar\nso baz\n-- me", false),
      ("\"foo bar\nso baz\"\n-- ", false),
      ("\"foo bar\nso baz\"\nme -- too", false),
    ];
    let bump = &Bump::new();
    for (input, expected) in cases {
      let mut parser = Parser::new(bump, input);
      let lines = parser.read_lines().unwrap();
      assert_eq!(
        lines.is_quoted_paragraph(),
        expected,
        "input was:\n\n```\n{}\n```\n",
        input
      );
    }
  }
}
