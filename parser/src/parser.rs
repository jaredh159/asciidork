use std::cell::RefCell;

use bumpalo::collections::Vec as BumpVec;
use bumpalo::{collections::String, Bump};

use crate::block::Block;
use crate::lexer::Lexer;
use crate::line::Line;
use crate::source_location::SourceLocation;
use crate::token::{Token, TokenKind::*};
use crate::Diagnostic;

#[derive(Debug)]
pub struct Node<'alloc> {
  pub loc: SourceLocation,
  pub text: String<'alloc>,
}

#[derive(Debug)]
pub struct Parser<'alloc, 'src> {
  pub(super) allocator: &'alloc Bump,
  pub(super) lexer: Lexer<'src>,
  pub(super) peeked_block: Option<Block<'alloc, 'src>>,
  pub(super) ctx: Context,
  pub(super) errors: RefCell<Vec<Diagnostic>>,
  pub(super) bail: bool, // todo: naming...
}

#[derive(Debug)]
pub(crate) struct Context {
  pub(crate) subs: Substitutions,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Substitutions {
  pub(crate) special_chars: bool,
  pub(crate) inline_formatting: bool,
  pub(crate) macros: bool,
}

impl<'alloc, 'src> Parser<'alloc, 'src> {
  pub fn new(allocator: &'alloc Bump, src: &'src str) -> Parser<'alloc, 'src> {
    Parser {
      allocator,
      lexer: Lexer::new(src),
      peeked_block: None,
      ctx: Context { subs: Substitutions::all() },
      errors: RefCell::new(Vec::new()),
      bail: true,
    }
  }

  pub(crate) fn read_line(&mut self) -> Option<Line<'alloc, 'src>> {
    self.lexer.consume_line(self.allocator)
  }

  pub(crate) fn read_block(&mut self) -> Option<Block<'alloc, 'src>> {
    if let Some(block) = self.peeked_block.take() {
      return Some(block);
    }
    self.lexer.consume_empty_lines();
    if self.lexer.is_eof() {
      return None;
    }
    let mut lines = BumpVec::new_in(self.allocator);
    while let Some(line) = self.lexer.consume_line(self.allocator) {
      lines.push(line);
      if self.lexer.peek_is('\n') {
        break;
      }
    }
    debug_assert!(!lines.is_empty());
    Some(Block::new(lines))
  }

  pub fn parse(&mut self) -> Node<'alloc> {
    let mut node = Node {
      loc: self.lexer.loc(),
      text: String::new_in(self.allocator),
    };
    loop {
      match self.lexer.next_token() {
        Token { kind: Eof, .. } => break,
        Token { loc, lexeme, .. } => {
          node.loc.extend(loc);
          node.text.push_str(lexeme);
        }
      }
    }
    node
  }
}

impl Substitutions {
  pub fn all() -> Self {
    Self {
      special_chars: true,
      inline_formatting: true,
      macros: true,
    }
  }

  pub fn none() -> Self {
    Self {
      special_chars: false,
      inline_formatting: false,
      macros: false,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_parser() {
    let bump = &Bump::new();
    let input = "hello:world";
    let mut parser = Parser::new(bump, input);
    let node = parser.parse();
    assert_eq!(node.text, "hello:world");
    assert_eq!(node.loc, SourceLocation::new(0, 11));
  }
}
