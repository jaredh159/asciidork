use crate::internal::*;
use crate::variants::token::*;

#[derive(Debug, Clone)]
pub struct ContiguousLines<'arena> {
  lines: Deq<'arena, Line<'arena>>,
}

impl<'arena> ContiguousLines<'arena> {
  pub const fn new(lines: Deq<'arena, Line<'arena>>) -> Self {
    ContiguousLines { lines }
  }

  pub fn empty(bump: &'arena Bump) -> Self {
    ContiguousLines::new(Deq::new(bump))
  }

  pub fn with_capacity(capacity: usize, bump: &'arena Bump) -> Self {
    ContiguousLines::new(Deq::with_capacity(capacity, bump))
  }

  pub fn push(&mut self, line: Line<'arena>) {
    self.lines.push(line);
  }

  pub fn len(&self) -> usize {
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

  pub fn iter(&'arena self) -> impl ExactSizeIterator<Item = &'arena Line<'arena>> + 'arena {
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

  pub fn contains_seq(&self, specs: &[TokenSpec]) -> bool {
    self.lines.iter().any(|line| line.contains_seq(specs))
  }

  pub fn contains_len(&self, kind: TokenKind, len: usize) -> bool {
    self.lines.iter().any(|line| line.contains_len(kind, len))
  }

  pub fn terminates_constrained(&self, stop_tokens: &[TokenSpec], ctx: &InlineCtx) -> bool {
    self
      .lines
      .iter()
      .any(|line| line.terminates_constrained(stop_tokens, ctx))
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

  pub fn first_loc(&self) -> Option<SourceLocation> {
    self.lines.first().and_then(|line| line.loc())
  }

  pub fn is_quoted_paragraph(&self) -> bool {
    if self.lines.len() < 2 {
      return false;
    }
    let last_line = self.last().unwrap();
    if !last_line.starts_with_seq(&[TokenSpec::Kind(Dashes), TokenSpec::Kind(Whitespace)])
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
    for line in self.lines.iter() {
      if line.starts_list_item() {
        return true;
      } else if line.is_comment() {
        continue;
      } else {
        return false;
      }
    }
    false
  }

  pub fn starts_extra_description_list_term(&self, ctx: ListMarker) -> bool {
    for line in self.lines.iter() {
      if line.list_marker() == Some(ctx) {
        return true;
      } else if line.is_comment() {
        continue;
      } else {
        return false;
      }
    }
    false
  }

  pub fn starts_with_seq(&self, kinds: &[TokenSpec]) -> bool {
    self
      .first()
      .map(|line| line.starts_with_seq(kinds))
      .unwrap_or(false)
  }

  pub fn starts_nested_list(&self, stack: &ListStack, allow_attrs: bool) -> bool {
    let Some(line) = self.first() else {
      return false;
    };
    if !allow_attrs || !line.is_block_attr_list() {
      return line.starts_nested_list(stack);
    }
    self
      .nth(1)
      .map(|line| line.starts_nested_list(stack))
      .unwrap_or(false)
  }

  pub fn starts_list_continuation(&self) -> bool {
    if self.len() < 2 {
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

  pub fn discard_until(&mut self, predicate: impl Fn(&Line<'arena>) -> bool) -> bool {
    while let Some(line) = self.first() {
      if predicate(line) {
        return true;
      }
      self.consume_current();
    }
    false
  }

  pub fn trim_uniform_leading_whitespace(&mut self) -> bool {
    if self.is_empty() || !self.first().unwrap().starts(Whitespace) {
      return false;
    }
    let len = self.first().unwrap().current_token().unwrap().lexeme.len();
    if !self
      .lines
      .iter()
      .all(|l| l.current_is_len(Whitespace, len) && l.num_tokens() > 1)
    {
      return false;
    }

    for line in self.lines.iter_mut() {
      line.discard_assert(Whitespace);
    }
    true
  }

  pub fn get_indentation(&self) -> usize {
    let mut indent = usize::MAX;
    for line in self.iter() {
      let line_indent = line.get_indentation();
      if line_indent < indent {
        indent = line_indent;
      }
    }
    if indent == usize::MAX {
      0
    } else {
      indent
    }
  }

  pub fn set_indentation(&mut self, indent: usize) {
    let current = self.get_indentation();
    if current == indent {
      return;
    }
    self.lines.iter_mut().for_each(|line| {
      let line_indent = line.get_indentation();
      if line_indent >= current {
        line.set_indentation(line_indent - current + indent);
      }
    });
  }

  #[cfg(debug_assertions)]
  pub fn debug_print(&self) {
    eprintln!("```");
    for line in self.iter() {
      eprintln!("{}", line.reassemble_src());
    }
    eprintln!("```");
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
  use test_utils::*;

  #[test]
  fn test_is_quoted_paragraph() {
    let cases = vec![
      ("\"foo bar\nso baz\"\n-- me", true),
      ("foo bar\nso baz\"\n-- me", false),
      ("\"foo bar\nso baz\n-- me", false),
      ("\"foo bar\nso baz\"\n-- ", false),
      ("\"foo bar\nso baz\"\nme -- too", false),
    ];
    for (input, expected) in cases {
      let mut parser = test_parser!(input);
      let lines = parser.read_lines().unwrap().unwrap();
      expect_eq!(lines.is_quoted_paragraph(), expected, from: input);
    }
  }
}
