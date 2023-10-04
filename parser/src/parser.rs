use bumpalo::{collections::String, Bump};

use crate::lexer::Lexer;
use crate::source_location::SourceLocation;
use crate::token::{Token, TokenKind::*};

// temp
pub struct Node<'alloc> {
  pub loc: SourceLocation,
  pub text: String<'alloc>,
}

pub struct Parser<'alloc> {
  allocator: &'alloc Bump,
  lexer: Lexer<'alloc>,
}

impl<'alloc> Parser<'alloc> {
  pub fn new(allocator: &'alloc Bump, src: &'alloc str) -> Parser<'alloc> {
    Parser {
      allocator,
      lexer: Lexer::new(allocator, src),
    }
  }

  pub fn parse(&mut self) -> Node<'alloc> {
    let mut node_loc = self.lexer.loc();
    let mut text = String::new_in(self.allocator);
    loop {
      match self.lexer.next_token() {
        Token { kind: Word, loc, value } => {
          node_loc.extend(loc);
          text.push_str(value.as_string());
        }
        Token { kind: Eof, .. } => break,
        Token { loc, .. } => {
          node_loc.extend(loc);
          text.push_str("•");
        }
      }
    }
    Node { loc: node_loc, text }
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
    assert_eq!(node.text, "hello•world");
    assert_eq!(node.loc, SourceLocation::new(0, 11));
  }
}
