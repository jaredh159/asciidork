use std::str::Chars;

use bumpalo::Bump;

use super::source_location::SourceLocation;
use super::token::{Token, TokenKind, TokenKind::*};

#[derive(Debug)]
pub struct Lexer<'alloc> {
  allocator: &'alloc Bump,
  src: &'alloc str,
  chars: Chars<'alloc>,
  peek: Option<char>,
}

impl<'alloc> Lexer<'alloc> {
  pub fn new(allocator: &'alloc Bump, src: &'alloc str) -> Lexer<'alloc> {
    let mut lexer = Lexer {
      allocator,
      src,
      chars: src.chars(),
      peek: None,
    };
    lexer.peek = lexer.chars.next();
    lexer
  }

  fn advance(&mut self) -> Option<char> {
    std::mem::replace(&mut self.peek, self.chars.next())
  }

  fn advance_if(&mut self, c: char) -> bool {
    if self.peek == Some(c) {
      self.advance();
      true
    } else {
      false
    }
  }

  fn advance_while(&mut self, c: char) {
    while self.advance_if(c) {}
  }

  pub fn next_token(&mut self) -> Token {
    match self.advance() {
      Some('=') => self.repeating('=', EqualSigns),
      Some(' ') | Some('\t') => self.whitespace(),
      Some('&') => self.single(Ampersand),
      Some('\n') => self.single(Newline),
      Some(':') => self.single(Colon),
      Some(';') => self.single(SemiColon),
      Some('<') => self.single(LessThan),
      Some('>') => self.single(GreaterThan),
      Some(',') => self.single(Comma),
      Some('^') => self.single(Caret),
      Some('~') => self.single(Tilde),
      Some('_') => self.single(Underscore),
      Some('*') => self.single(Star),
      Some('.') => self.single(Dot),
      Some('!') => self.single(Bang),
      Some('`') => self.single(Backtick),
      Some('+') => self.single(Plus),
      Some('[') => self.single(OpenBracket),
      Some(']') => self.single(CloseBracket),
      Some('#') => self.single(Hash),
      Some('%') => self.single(Percent),
      Some('"') => self.single(DoubleQuote),
      Some('/') if self.starts_comment() => self.comment(),
      Some('\'') => self.single(SingleQuote),
      Some('\\') => self.single(Backslash),
      Some(_) => self.word(),
      None => Token {
        kind: Eof,
        loc: SourceLocation::new(self.offset(), self.offset()),
        lexeme: "",
      },
    }
  }

  fn offset(&self) -> usize {
    self.src.len() - self.chars.as_str().len() - self.peek.is_some() as usize // O(1) √
  }

  pub fn loc(&self) -> SourceLocation {
    SourceLocation::from(self.offset())
  }

  fn single(&self, kind: TokenKind) -> Token {
    let end = self.offset();
    let start = end - 1;
    Token {
      kind,
      loc: SourceLocation::new(start, end),
      lexeme: &self.src[start..end],
    }
  }

  fn repeating(&mut self, c: char, kind: TokenKind) -> Token {
    let start = self.offset() - 1;
    self.advance_while(c);
    let end = self.offset();
    Token {
      kind,
      loc: SourceLocation::new(start, end),
      lexeme: &self.src[start..end],
    }
  }

  fn word(&mut self) -> Token {
    let start = self.offset() - 1;
    self.advance_until_one_of(&[
      ' ', '\t', '\n', ':', ';', '<', '>', ',', '^', '_', '~', '*', '!', '`', '+', '.', '[', ']',
      '=', '"', '\'', '\\', '%', '#', '&',
    ]);
    let end = self.offset();
    let lexeme = &self.src[start..end];
    // check for macros...
    if self.peek == Some(':') {
      if self.is_macro_name(lexeme) {
        self.advance();
        return Token {
          kind: MacroName,
          loc: SourceLocation::new(start, end + 1),
          lexeme: &self.src[start..end + 1],
        };
        // ...checking for contiguous footnote `somethingfootnote:[]`
      } else if lexeme.ends_with('e') && lexeme.ends_with("footnote") {
        self.chars = self.src[end - 8..].chars();
        self.peek = self.chars.next();
        return Token {
          kind: Word,
          loc: SourceLocation::new(start, end - 8),
          lexeme: &self.src[start..end - 8],
        };
      }
    }
    Token {
      kind: Word,
      loc: SourceLocation::new(start, end),
      lexeme: &self.src[start..end],
    }
  }

  fn is_macro_name(&self, lexeme: &str) -> bool {
    matches!(
      lexeme,
      "footnote" | "image" | "irc" | "icon" | "link" | "http" | "https" | "ftp" | "mailto"
    )
  }

  fn advance_until(&mut self, stop: char) {
    loop {
      match self.peek {
        None => break,
        Some(c) if c == stop => break,
        _ => {
          self.advance();
        }
      }
    }
  }

  fn advance_until_one_of(&mut self, chars: &[char]) {
    loop {
      match self.peek {
        Some(c) if chars.contains(&c) => break,
        None => break,
        _ => {
          self.advance();
        }
      }
    }
  }

  fn advance_while_one_of(&mut self, chars: &[char]) {
    loop {
      match self.peek {
        Some(c) if chars.contains(&c) => {}
        _ => break,
      }
      self.advance();
    }
  }

  fn comment(&mut self) -> Token {
    let start = self.offset() - 1;
    // TODO: block comments, testing if we have 2 more slashes
    self.advance_until('\n');
    let end = self.offset();
    Token {
      kind: CommentLine,
      loc: SourceLocation::new(start, end),
      lexeme: &self.src[start..end],
    }
  }

  fn whitespace(&mut self) -> Token<'_> {
    let start = self.offset() - 1;
    self.advance_while_one_of(&[' ', '\t']);
    let end = self.offset();
    Token {
      kind: Whitespace,
      loc: SourceLocation::new(start, end),
      lexeme: &self.src[start..end],
    }
  }

  fn starts_comment(&self) -> bool {
    if self.peek != Some('/') {
      return false;
    }
    let offset = self.offset();
    if offset == 1 {
      return true;
    }
    let prev_offset = self.offset().saturating_sub(1);
    // must be at the beginning of a line, so `https://foobar` not match
    matches!(self.src.chars().nth(prev_offset), Some('\n') | None)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::source_location::SourceLocation;
  use crate::token::TokenKind;

  #[test]
  fn test_tokens() {
    let cases = vec![
      ("foo.bar", vec![(Word, "foo"), (Dot, "."), (Word, "bar")]),
      (
        "roflfootnote:",
        vec![(Word, "rofl"), (MacroName, "footnote:")],
      ),
      ("footnote:", vec![(MacroName, "footnote:")]),
      ("==", vec![(EqualSigns, "==")]),
      ("===", vec![(EqualSigns, "===")]),
      ("// foo", vec![(CommentLine, "// foo")]),
      (
        "foo=;",
        vec![(Word, "foo"), (EqualSigns, "="), (SemiColon, ";")],
      ),
      (
        "foo;=",
        vec![(Word, "foo"), (SemiColon, ";"), (EqualSigns, "=")],
      ),
      (
        "foo;image:&bar,lol^~_*.!`+[]#'\"\\%",
        vec![
          (Word, "foo"),
          (SemiColon, ";"),
          (MacroName, "image:"),
          (Ampersand, "&"),
          (Word, "bar"),
          (Comma, ","),
          (Word, "lol"),
          (Caret, "^"),
          (Tilde, "~"),
          (Underscore, "_"),
          (Star, "*"),
          (Dot, "."),
          (Bang, "!"),
          (Backtick, "`"),
          (Plus, "+"),
          (OpenBracket, "["),
          (CloseBracket, "]"),
          (Hash, "#"),
          (SingleQuote, "'"),
          (DoubleQuote, "\""),
          (Backslash, "\\"),
          (Percent, "%"),
        ],
      ),
      (
        "Foobar\n\n",
        vec![(Word, "Foobar"), (Newline, "\n"), (Newline, "\n")],
      ),
      (
        "== Title",
        vec![(EqualSigns, "=="), (Whitespace, " "), (Word, "Title")],
      ),
      (
        "// this comment line is ignored
= Document Title
Kismet R. Lee <kismet@asciidoctor.org>
:description: The document's description.
:sectanchors:
:url-repo: https://my-git-repo.com

The document body starts here.
",
        vec![
          (CommentLine, "// this comment line is ignored"),
          (Newline, "\n"),
          (EqualSigns, "="),
          (Whitespace, " "),
          (Word, "Document"),
          (Whitespace, " "),
          (Word, "Title"),
          (Newline, "\n"),
          (Word, "Kismet"),
          (Whitespace, " "),
          (Word, "R"),
          (Dot, "."),
          (Whitespace, " "),
          (Word, "Lee"),
          (Whitespace, " "),
          (LessThan, "<"),
          (Word, "kismet@asciidoctor"),
          (Dot, "."),
          (Word, "org"),
          (GreaterThan, ">"),
          (Newline, "\n"),
          (Colon, ":"),
          (Word, "description"),
          (Colon, ":"),
          (Whitespace, " "),
          (Word, "The"),
          (Whitespace, " "),
          (Word, "document"),
          (SingleQuote, "'"),
          (Word, "s"),
          (Whitespace, " "),
          (Word, "description"),
          (Dot, "."),
          (Newline, "\n"),
          (Colon, ":"),
          (Word, "sectanchors"),
          (Colon, ":"),
          (Newline, "\n"),
          (Colon, ":"),
          (Word, "url-repo"),
          (Colon, ":"),
          (Whitespace, " "),
          (MacroName, "https:"),
          (Word, "//my-git-repo"),
          (Dot, "."),
          (Word, "com"),
          (Newline, "\n"),
          (Newline, "\n"),
          (Word, "The"),
          (Whitespace, " "),
          (Word, "document"),
          (Whitespace, " "),
          (Word, "body"),
          (Whitespace, " "),
          (Word, "starts"),
          (Whitespace, " "),
          (Word, "here"),
          (Dot, "."),
          (Newline, "\n"),
        ],
      ),
    ];
    for (input, expected) in cases {
      let bump = &Bump::new();
      let mut lexer = Lexer::new(bump, input);
      let mut index = 0;
      for (token_type, lexeme) in expected {
        let start = index;
        let end = start + lexeme.len();
        let expected_token = Token {
          kind: token_type,
          loc: SourceLocation::new(start, end),
          lexeme,
        };
        assert_eq!(lexer.next_token(), expected_token);
        index = end;
      }
      assert_eq!(lexer.next_token().kind, Eof);
    }
  }

  #[test]
  fn test_tokens_full() {
    let bump = &Bump::new();
    let input = "&^foobar[";
    let mut lexer = Lexer::new(bump, input);
    assert_eq!(lexer.src, input);
    assert_eq!(
      lexer.next_token(),
      Token {
        kind: TokenKind::Ampersand,
        loc: SourceLocation::new(0, 1),
        lexeme: "&",
      }
    );
    assert_eq!(
      lexer.next_token(),
      Token {
        kind: TokenKind::Caret,
        loc: SourceLocation::new(1, 2),
        lexeme: "^",
      }
    );
    assert_eq!(
      lexer.next_token(),
      Token {
        kind: TokenKind::Word,
        loc: SourceLocation::new(2, 8),
        lexeme: "foobar",
      }
    );
    assert_eq!(
      lexer.next_token(),
      Token {
        kind: TokenKind::OpenBracket,
        loc: SourceLocation::new(8, 9),
        lexeme: "[",
      }
    );
    assert_eq!(
      lexer.next_token(),
      Token {
        kind: TokenKind::Eof,
        loc: SourceLocation::new(9, 9),
        lexeme: "",
      }
    );
  }
}
