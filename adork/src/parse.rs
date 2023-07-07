use std::collections::VecDeque;
use std::io::BufRead;

mod ast;
mod author;
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
use crate::token::{Token, TokenType};

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
    while let Some(token) = self.lexer.next() {
      let token_type = token.token_type;
      tokens.push(token);
      if token_type == TokenType::Newlines {
        break;
      }
    }
    debug_assert!(tokens.len() > 0);
    Some(Line::new(tokens))
  }

  fn read_block(&mut self) -> Option<LineBlock> {
    if self.lexer.is_eof() {
      return None;
    }
    let mut lines = VecDeque::new();
    while let Some(line) = self.read_line() {
      let end_of_block = line.last_token().unwrap().ends_block();
      lines.push_back(line);
      if end_of_block {
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

  #[cfg(test)]
  pub(crate) fn line_test(input: &str) -> (Line, Parser<&[u8]>) {
    let lexer = Lexer::<&[u8]>::new_from(input);
    let mut parser = Parser::new(lexer);
    (parser.read_line().unwrap(), parser)
  }
}

// tests

#[cfg(test)]
mod tests {
  use super::*;
  use crate::parse::inline::Inline;
  use crate::t::*;
  use indoc::indoc;
  use std::collections::HashMap;

  #[test]
  fn test_parse_example_doc_header() {
    let input = indoc! {"
      // this comment line is ignored
      = Document Title
      Kismet R. Lee <kismet@asciidoctor.org>
      :description: The document's description.
      :sectanchors:
      :url-repo: https://my-git-repo.com

      The document body starts here.
    "};

    let expected_header = DocHeader {
      title: Some(DocTitle {
        heading: vec![Inline::Text(s("Document Title"))],
        subtitle: None,
      }),
      authors: vec![Author {
        first_name: s("Kismet"),
        middle_name: Some(s("R.")),
        last_name: s("Lee"),
        email: Some(s("kismet@asciidoctor.org")),
      }],
      revision: None,
      attrs: HashMap::from([
        (s("description"), s("The document's description.")),
        (s("sectanchors"), s("")),
        (s("url-repo"), s("https://my-git-repo.com")),
      ]),
    };

    let document = Parser::<&[u8]>::parse_str(input).unwrap();
    assert_eq!(document.header, Some(expected_header));
  }
}
