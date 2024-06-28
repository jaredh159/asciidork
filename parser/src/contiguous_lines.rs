use crate::internal::*;

#[derive(Debug, Clone)]
pub struct ContiguousLines<'arena> {
  lines: Deq<'arena, Line<'arena>>,
}

impl<'arena> ContiguousLines<'arena> {
  pub const fn new(lines: Deq<'arena, Line<'arena>>) -> Self {
    ContiguousLines { lines }
  }

  pub fn num_lines(&self) -> usize {
    self.lines.len()
  }

  pub fn num_tokens(&self) -> usize {
    self.lines.iter().map(Line::num_tokens).sum()
  }

  pub fn current(&self) -> Option<&Line<'arena>> {
    self.lines.get(0)
  }

  pub fn current_mut(&mut self) -> Option<&mut Line<'arena>> {
    self.lines.get_mut(0)
  }

  pub fn current_satisfies(&self, f: impl Fn(&Line<'arena>) -> bool) -> bool {
    self.current().map(f).unwrap_or(false)
  }

  fn first(&self) -> Option<&Line<'arena>> {
    self.current()
  }

  pub fn iter(&'arena self) -> impl ExactSizeIterator<Item = &Line<'arena>> + '_ {
    self.lines.iter()
  }

  pub fn pop(&mut self) -> Option<Line<'arena>> {
    self.lines.pop()
  }

  pub fn last(&self) -> Option<&Line<'arena>> {
    self.lines.last()
  }

  pub fn last_mut(&mut self) -> Option<&mut Line<'arena>> {
    self.lines.last_mut()
  }

  pub fn nth(&self, n: usize) -> Option<&Line<'arena>> {
    self.lines.get(n)
  }

  pub fn current_token(&self) -> Option<&Token<'arena>> {
    self.current().and_then(|line| line.current_token())
  }

  pub fn nth_token(&self, n: usize) -> Option<&Token<'arena>> {
    self.current().and_then(|line| line.nth_token(n))
  }

  pub fn is_empty(&self) -> bool {
    self.lines.is_empty()
  }

  pub fn consume_current(&mut self) -> Option<Line<'arena>> {
    self.lines.pop_front()
  }

  pub fn consume_current_token(&mut self) -> Option<Token<'arena>> {
    self
      .consume_current()
      .and_then(|mut line| line.consume_current())
  }

  pub fn extend(&mut self, other: BumpVec<'arena, Line<'arena>>) {
    self.lines.reserve(other.len());
    self.lines.extend(other);
  }

  pub fn restore_if_nonempty(&mut self, line: Line<'arena>) {
    if !line.is_empty() {
      self.lines.push_front(line);
    }
  }

  pub fn any(&self, f: impl FnMut(&Line<'arena>) -> bool) -> bool {
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

  pub fn loc(&self) -> Option<SourceLocation> {
    if let Some(line) = self.lines.first() {
      line.loc()
    } else {
      None
    }
  }

  pub fn last_loc(&self) -> Option<SourceLocation> {
    self.lines.last().and_then(|line| line.last_loc())
  }

  pub fn is_quoted_paragraph(&self) -> bool {
    use TokenKind::*;
    if self.lines.len() < 2 {
      return false;
    }
    let last_line = self.last().unwrap();
    if !last_line.starts_with_seq(&[Dashes, Whitespace])
      || last_line.num_tokens() < 3
      || !last_line.current_is_len(Dashes, 2)
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

  pub fn starts_list(&self) -> bool {
    self
      .first()
      .map(|line| line.starts_list_item())
      .unwrap_or(false)
  }

  pub fn starts_with_seq(&self, kinds: &[TokenKind]) -> bool {
    self
      .first()
      .map(|line| line.starts_with_seq(kinds))
      .unwrap_or(false)
  }

  pub fn starts_nested_list(&self, stack: &ListStack, allow_attrs: bool) -> bool {
    let Some(line) = self.first() else {
      return false;
    };
    if !allow_attrs || !line.is_attr_list() {
      return line.starts_nested_list(stack);
    }
    self
      .nth(1)
      .map(|line| line.starts_nested_list(stack))
      .unwrap_or(false)
  }

  pub fn starts_list_continuation(&self) -> bool {
    if self.num_lines() < 2 {
      return false;
    }
    let Some(line) = self.first() else {
      return false;
    };
    line.is_list_continuation()
  }

  pub fn starts(&self, kind: TokenKind) -> bool {
    self.first().map(|line| line.starts(kind)).unwrap_or(false)
  }

  pub fn starts_with_comment_line(&self) -> bool {
    self.first().map(Line::is_comment).unwrap_or(false)
  }

  pub fn discard_leading_comment_lines(&mut self) {
    while self.starts_with_comment_line() {
      self.consume_current();
    }
  }

  pub fn discard_until(&mut self, predicate: impl Fn(&Line<'arena>) -> bool) {
    while let Some(line) = self.first() {
      if predicate(line) {
        return;
      }
      self.consume_current();
    }
  }

  pub fn starts_section(&self, meta: &ChunkMeta<'arena>) -> bool {
    self.section_start_level(meta).is_some()
  }

  pub fn section_start_level(&self, meta: &ChunkMeta<'arena>) -> Option<u8> {
    for line in self.iter() {
      if line.is_attr_list() || line.is_chunk_title() {
        continue;
      } else if let Some(level) = line.heading_level() {
        return match meta.attrs_has_str_positional("discrete") {
          true => None,
          false => Some(level),
        };
      } else {
        return None;
      }
    }
    None
  }

  pub fn trim_uniform_leading_whitespace(&mut self) -> bool {
    if self.is_empty() || !self.first().unwrap().starts(TokenKind::Whitespace) {
      return false;
    }
    let len = self.first().unwrap().current_token().unwrap().lexeme.len();
    if !self
      .lines
      .iter()
      .all(|l| l.current_is_len(TokenKind::Whitespace, len) && l.num_tokens() > 1)
    {
      return false;
    }

    for line in self.lines.iter_mut() {
      line.discard_assert(TokenKind::Whitespace);
    }
    true
  }
}

impl<'arena> DefaultIn<'arena> for Line<'arena> {
  fn default_in(bump: &'arena Bump) -> Self {
    Line::new(Deq::new(bump))
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use test_utils::assert_eq;

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
      let mut parser = Parser::from_str(input, bump);
      let lines = parser.read_lines().unwrap();
      assert_eq!(lines.is_quoted_paragraph(), expected, from: input);
    }
  }
}
