use std::cell::RefCell;

use bumpalo::collections::Vec as BumpVec;
use bumpalo::Bump;

use crate::ast::*;
use crate::block::Block;
use crate::lexer::Lexer;
use crate::line::Line;
use crate::Diagnostic;

#[derive(Debug)]
pub struct Parser<'bmp: 'src, 'src> {
  pub(super) bump: &'bmp Bump,
  pub(super) lexer: Lexer<'src>,
  pub(super) document: Document<'bmp>,
  pub(super) peeked_block: Option<Block<'bmp, 'src>>,
  pub(super) ctx: Context,
  pub(super) errors: RefCell<Vec<Diagnostic>>,
  pub(super) bail: bool, // todo: naming...
}

pub struct ParseResult<'bmp> {
  pub document: Document<'bmp>,
  pub warnings: Vec<Diagnostic>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum Delimiter {
  Sidebar,
}

#[derive(Debug)]
pub(crate) struct Context {
  pub(crate) subs: Substitutions,
  pub(crate) delimiter: Option<Delimiter>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Substitutions {
  pub(crate) special_chars: bool,
  /// aka: `quotes`
  pub(crate) inline_formatting: bool,
  pub(crate) attr_refs: bool,
  pub(crate) char_replacement: bool,
  pub(crate) macros: bool,
  pub(crate) post_replacement: bool,
}

impl<'bmp, 'src> Parser<'bmp, 'src> {
  pub fn new(bump: &'bmp Bump, src: &'src str) -> Parser<'bmp, 'src> {
    Parser {
      bump,
      lexer: Lexer::new(src),
      document: Document::new_in(bump),
      peeked_block: None,
      ctx: Context {
        subs: Substitutions::all(),
        delimiter: None,
      },
      errors: RefCell::new(Vec::new()),
      bail: true,
    }
  }

  pub(crate) fn debug_loc(&self, loc: SourceLocation) {
    println!("{:?}, {}", loc, self.lexer.loc_src(loc));
  }

  pub(crate) fn read_line(&mut self) -> Option<Line<'bmp, 'src>> {
    self.lexer.consume_line(self.bump)
  }

  pub(crate) fn line_from(
    &self,
    tokens: bumpalo::collections::Vec<'bmp, crate::token::Token<'src>>,
    loc: SourceLocation,
  ) -> Line<'bmp, 'src> {
    Line::new(tokens, self.lexer.loc_src(loc))
  }

  pub(crate) fn read_block(&mut self) -> Option<Block<'bmp, 'src>> {
    if let Some(block) = self.peeked_block.take() {
      return Some(block);
    }
    self.lexer.consume_empty_lines();
    if self.lexer.is_eof() {
      return None;
    }
    let mut lines = BumpVec::new_in(self.bump);
    while let Some(line) = self.lexer.consume_line(self.bump) {
      lines.push(line);
      if self.lexer.peek_is('\n') {
        break;
      }
    }
    debug_assert!(!lines.is_empty());
    Some(Block::new(lines))
  }

  pub fn restore_block(&mut self, block: Block<'bmp, 'src>) {
    debug_assert!(self.peeked_block.is_none());
    self.peeked_block = Some(block);
  }

  pub fn parse(mut self) -> std::result::Result<ParseResult<'bmp>, Vec<Diagnostic>> {
    self.document.header = self.parse_document_header()?;

    while let Some(block) = self.parse_block()? {
      self.document.content.push_block(block);
    }

    Ok(ParseResult {
      document: self.document,
      warnings: vec![],
    })
  }
}

impl Substitutions {
  pub fn all() -> Self {
    Self {
      special_chars: true,
      inline_formatting: true,
      attr_refs: true,
      char_replacement: true,
      macros: true,
      post_replacement: true,
    }
  }

  pub fn none() -> Self {
    Self {
      special_chars: false,
      inline_formatting: false,
      attr_refs: false,
      char_replacement: false,
      macros: false,
      post_replacement: false,
    }
  }
}

impl From<Diagnostic> for Vec<Diagnostic> {
  fn from(diagnostic: Diagnostic) -> Self {
    vec![diagnostic]
  }
}
