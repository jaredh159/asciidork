use std::collections::VecDeque;
use std::io::BufRead;

mod ast;
mod author;
mod display_err;
mod doc_attrs;
mod doc_header;
mod inline;
pub(super) mod line;
mod line_block;

use crate::either::Either;
use crate::err::ParseErr;
use crate::lexer::Lexer;
use crate::parse::ast::*;
use crate::parse::line::Line;
use crate::parse::line_block::LineBlock;
use crate::token::Token;

type Result<T> = std::result::Result<T, ParseErr>;

pub struct Parser<R: BufRead> {
  lexer: Lexer<R>,
  document: Document,
}

impl<R: BufRead> Parser<R> {
  pub fn new(lexer: Lexer<R>) -> Parser<R> {
    Parser {
      lexer,
      document: Document {
        doctype: DocType::Article,
        header: None,
        content: DocContent::Blocks(vec![]),
      },
    }
  }

  pub fn from(reader: R) -> Parser<R> {
    let lexer = Lexer::new(reader);
    Parser::new(lexer)
  }

  pub fn parse_str(input: &str) -> Result<Document> {
    let lexer = Lexer::<&[u8]>::new_from(input);
    let parser = Parser::new(lexer);
    parser.parse()
  }

  pub fn parse(mut self) -> Result<Document> {
    let header_result = self.parse_document_header()?;
    if header_result.is_right() {
      let doc_header = header_result.take_right().unwrap();
      self.document.header = Some(doc_header);
    }
    Ok(self.document)
  }

  pub(crate) fn lexeme_string(&self, token: &Token) -> String {
    self.lexer.string(token)
  }

  pub(crate) fn lexeme_str(&self, token: &Token) -> &str {
    self.lexer.lexeme(token)
  }

  pub(crate) fn read_line(&mut self) -> Option<Line> {
    if self.lexer.is_eof() {
      return None;
    }

    let mut tokens = vec![];
    while !self.lexer.current_is(b'\n') && !self.lexer.is_eof() {
      tokens.push(self.lexer.next().unwrap());
    }
    self.lexer.consume_newline();

    Some(Line::new(tokens))
  }

  fn read_block(&mut self) -> Option<LineBlock> {
    self.lexer.consume_empty_lines();
    if self.lexer.is_eof() {
      return None;
    }

    let mut lines = VecDeque::new();
    while let Some(line) = self.read_line() {
      lines.push_back(line);
      if self.lexer.current_is(b'\n') {
        break;
      }
    }

    Some(LineBlock::new(lines))
  }

  fn parse_document_header(&mut self) -> Result<Either<LineBlock, DocHeader>> {
    let first_block = self.read_block().expect("non-empty document");
    if !doc_header::is_doc_header(&first_block) {
      Ok(Either::Left(first_block))
    } else {
      Ok(Either::Right(self.parse_doc_header(first_block)?))
    }
  }
}

// tests

#[cfg(test)]
mod tests {
  use crate::t::*;

  #[test]
  fn test_read_line() {
    let input = "hello world\ngoodbye\n\nfoo\n";
    let mut parser = parser_of(input);

    let hello_world = parser.read_line().unwrap();
    assert_eq!(parser.lexeme_str(&hello_world.tokens[0]), "hello");
    assert_eq!(hello_world.tokens.len(), 3);

    let goodbye = parser.read_line().unwrap();
    assert_eq!(parser.lexeme_str(&goodbye.tokens[0]), "goodbye");
    assert_eq!(goodbye.tokens.len(), 1);

    assert_eq!(parser.read_line().unwrap().tokens.len(), 0); // empty line

    let foo = parser.read_line().unwrap();
    assert_eq!(parser.lexeme_str(&foo.tokens[0]), "foo");
    assert_eq!(foo.tokens.len(), 1);

    assert!(parser.read_line().is_none()); // eof
  }

  #[test]
  fn test_read_blocks() {
    let input = "hello\n\ngoodbye\n";
    let mut parser = parser_of(input);
    assert!(parser.read_block().is_some());
    assert!(parser.read_block().is_some());
    assert!(parser.read_block().is_none());

    let input = "// comment\nhello\n\ngoodbye\n";
    let mut parser = parser_of(input);
    assert!(parser.read_block().is_some());
    assert!(parser.read_block().is_some());
    assert!(parser.read_block().is_none());
  }
}
