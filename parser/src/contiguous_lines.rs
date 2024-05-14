use crate::internal::*;

#[derive(Debug, Clone)]
pub struct ContiguousLines<'bmp, 'src> {
  // NB: lines kept in reverse, as there is no VecDeque in bumpalo
  // and we almost always want to consume from the front, so fake it
  reversed_lines: BumpVec<'bmp, Line<'bmp, 'src>>,
}

impl<'bmp, 'src> ContiguousLines<'bmp, 'src> {
  pub fn new(mut lines: BumpVec<'bmp, Line<'bmp, 'src>>) -> Self {
    lines.reverse();
    ContiguousLines { reversed_lines: lines }
  }

  pub fn num_lines(&self) -> usize {
    self.reversed_lines.len()
  }

  pub fn num_tokens(&self) -> usize {
    self.reversed_lines.iter().map(Line::num_tokens).sum()
  }

  pub fn current(&self) -> Option<&Line<'bmp, 'src>> {
    self.reversed_lines.last()
  }

  pub fn current_mut(&mut self) -> Option<&mut Line<'bmp, 'src>> {
    self.reversed_lines.last_mut()
  }

  pub fn current_satisfies(&self, f: impl Fn(&Line<'bmp, 'src>) -> bool) -> bool {
    self.current().map(f).unwrap_or(false)
  }

  pub fn first(&self) -> Option<&Line<'bmp, 'src>> {
    self.reversed_lines.last()
  }

  pub fn iter(&'bmp self) -> impl ExactSizeIterator<Item = &Line<'bmp, 'src>> + '_ {
    LinesIter {
      lines: self,
      pos: self.num_lines() - 1,
    }
  }

  pub fn last(&self) -> Option<&Line<'bmp, 'src>> {
    self.reversed_lines.first()
  }

  pub fn last_mut(&mut self) -> Option<&mut Line<'bmp, 'src>> {
    self.reversed_lines.first_mut()
  }

  pub fn remove_last_unchecked(&mut self) -> Line<'bmp, 'src> {
    self.reversed_lines.remove(0)
  }

  pub fn nth(&self, n: usize) -> Option<&Line<'bmp, 'src>> {
    self.reversed_lines.get(self.reversed_lines.len() - n - 1)
  }

  pub fn current_token(&self) -> Option<&Token<'src>> {
    self.current().and_then(|line| line.current_token())
  }

  pub fn nth_token(&self, n: usize) -> Option<&Token<'src>> {
    self.current().and_then(|line| line.nth_token(n))
  }

  pub fn is_empty(&self) -> bool {
    self.reversed_lines.is_empty()
  }

  pub fn consume_current(&mut self) -> Option<Line<'bmp, 'src>> {
    self.reversed_lines.pop()
  }

  pub fn consume_current_token(&mut self) -> Option<Token<'src>> {
    self
      .consume_current()
      .and_then(|mut line| line.consume_current())
  }

  pub fn extend(&mut self, mut other: BumpVec<'bmp, Line<'bmp, 'src>>) {
    other.reverse();
    other.extend(self.reversed_lines.drain(..));
    self.reversed_lines = other;
  }

  pub fn restore_if_nonempty(&mut self, line: Line<'bmp, 'src>) {
    if !line.is_empty() {
      self.reversed_lines.push(line);
    }
  }

  pub fn any(&self, f: impl FnMut(&Line<'bmp, 'src>) -> bool) -> bool {
    self.reversed_lines.iter().any(f)
  }

  pub fn contains_seq(&self, kinds: &[TokenKind]) -> bool {
    self
      .reversed_lines
      .iter()
      .any(|line| line.contains_seq(kinds))
  }

  pub fn terminates_constrained(&self, stop_tokens: &[TokenKind]) -> bool {
    self
      .reversed_lines
      .iter()
      .any(|line| line.terminates_constrained(stop_tokens))
  }

  pub fn is_block_macro(&self) -> bool {
    self.reversed_lines.len() == 1 && self.current().unwrap().is_block_macro()
  }

  pub fn loc(&self) -> Option<SourceLocation> {
    if let Some(line) = self.reversed_lines.last() {
      line.loc()
    } else {
      None
    }
  }

  pub fn last_loc(&self) -> Option<SourceLocation> {
    self.reversed_lines.first().and_then(|line| line.last_loc())
  }

  pub fn is_quoted_paragraph(&self) -> bool {
    use TokenKind::*;
    if self.reversed_lines.len() < 2 {
      return false;
    }
    let last_line = self.last().unwrap();
    if !last_line.starts_with_seq(&[Dashes, Whitespace])
      || !last_line.src.starts_with("-- ")
      || last_line.num_tokens() < 3
    {
      return false;
    }
    let first_line = self.current().unwrap();
    if !first_line.starts(DoubleQuote) {
      return false;
    }
    let penult = self.nth(self.reversed_lines.len() - 2).unwrap();
    penult.ends(DoubleQuote)
  }

  pub fn starts_list(&self) -> bool {
    self
      .first()
      .map(|line| line.starts_list_item())
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

  pub fn discard_until(&mut self, predicate: impl Fn(&Line<'bmp, 'src>) -> bool) {
    while let Some(line) = self.first() {
      if predicate(line) {
        return;
      }
      self.consume_current();
    }
  }

  pub fn starts_section(&self, meta: &ChunkMeta<'bmp>) -> bool {
    self.section_start_level(meta).is_some()
  }

  pub fn section_start_level(&self, meta: &ChunkMeta<'bmp>) -> Option<u8> {
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
      .reversed_lines
      .iter()
      .all(|l| l.current_is_len(TokenKind::Whitespace, len) && l.num_tokens() > 1)
    {
      return false;
    }

    for line in self.reversed_lines.iter_mut() {
      line.discard_assert(TokenKind::Whitespace);
    }
    true
  }
}

struct LinesIter<'bmp, 'src> {
  lines: &'bmp ContiguousLines<'bmp, 'src>,
  pos: usize,
}

impl<'bmp, 'src> Iterator for LinesIter<'bmp, 'src> {
  type Item = &'bmp Line<'bmp, 'src>;

  fn next(&mut self) -> Option<Self::Item> {
    if self.pos == usize::MAX {
      None
    } else {
      let item = self.lines.reversed_lines.get(self.pos);
      self.pos = if self.pos == 0 { usize::MAX } else { self.pos - 1 };
      item
    }
  }
}

impl<'bmp, 'src> ExactSizeIterator for LinesIter<'bmp, 'src> {
  fn len(&self) -> usize {
    self.lines.reversed_lines.len()
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
      let mut parser = Parser::new(bump, input);
      let lines = parser.read_lines().unwrap();
      assert_eq!(lines.is_quoted_paragraph(), expected, from: input);
    }
  }
}
