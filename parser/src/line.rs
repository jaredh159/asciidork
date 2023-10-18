use bumpalo::collections::Vec as BumpVec;

use crate::token::{Token, TokenKind};

#[derive(Debug)]
pub struct Line<'alloc, 'src> {
  pub src: &'src str,
  tokens: BumpVec<'alloc, Token<'src>>,
  pos: usize,
}

impl<'alloc, 'src> Line<'alloc, 'src> {
  pub fn new(tokens: BumpVec<'alloc, Token<'src>>, src: &'src str) -> Self {
    Line { tokens, src, pos: 0 }
  }

  pub fn current_token(&self) -> Option<&Token<'src>> {
    self.tokens.get(self.pos)
  }

  pub fn current_is(&self, kind: TokenKind) -> bool {
    self.current_token().map_or(false, |t| t.kind == kind)
  }

  pub fn is_empty(&self) -> bool {
    self.pos >= self.tokens.len()
  }

  pub fn discard(&mut self, n: usize) {
    for _ in 0..n {
      _ = self.consume_current();
    }
  }

  pub fn contains_nonescaped(&self, token_type: TokenKind) -> bool {
    self.first_nonescaped(token_type).is_some()
  }

  pub fn first_nonescaped(&self, token_type: TokenKind) -> Option<&Token> {
    let mut prev: Option<TokenKind> = None;
    for i in self.pos..self.tokens.len() {
      let token = &self.tokens[i];
      if token.is(token_type) && prev.map_or(true, |k| k != TokenKind::Backslash) {
        return Some(token);
      }
      prev = Some(token.kind);
    }
    None
  }

  #[must_use]
  pub fn consume_current(&mut self) -> Option<Token<'src>> {
    if self.is_empty() {
      return None;
    }
    let token = std::mem::take(&mut self.tokens[self.pos]);
    self.pos += 1;
    self.src = &self.src[token.lexeme.len()..];
    Some(token)
  }
}

#[cfg(test)]
mod tests {
  use crate::lexer::Lexer;
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
}
