use crate::ast::UrlScheme;
use crate::parse::Parser;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Token {
  pub token_type: TokenType,
  pub start: usize,
  pub end: usize,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TokenType {
  Ampersand,
  Backtick,
  Backslash,
  Bang,
  Caret,
  CloseBracket,
  Colon,
  Comma,
  CommentBlock,
  CommentLine,
  DoubleQuote,
  Dot,
  EqualSigns,
  GreaterThan,
  Hash,
  LessThan,
  MacroName,
  Newline,
  OpenBracket,
  Percent,
  Plus,
  SemiColon,
  SingleQuote,
  Star,
  Tilde,
  Underscore,
  Whitespace,
  Word,
}

impl Token {
  pub fn new(token_type: TokenType, start: usize, end: usize) -> Token {
    Token { token_type, start, end }
  }

  pub fn empty() -> Token {
    Token {
      token_type: TokenType::Whitespace,
      start: 0,
      end: 0,
    }
  }

  pub fn len(&self) -> usize {
    self.end - self.start
  }

  pub fn is(&self, token_type: TokenType) -> bool {
    self.token_type == token_type
  }

  pub fn is_len(&self, token_type: TokenType, len: usize) -> bool {
    self.token_type == token_type && self.end - self.start == len
  }

  pub fn to_url_scheme(&self, parser: &Parser) -> Option<UrlScheme> {
    match self.token_type {
      TokenType::MacroName => {
        let macro_name = parser.lexeme_str(self);
        match macro_name {
          "https:" => Some(UrlScheme::Https),
          "http:" => Some(UrlScheme::Http),
          "ftp:" => Some(UrlScheme::Ftp),
          "irc:" => Some(UrlScheme::Irc),
          "mailto:" => Some(UrlScheme::Mailto),
          _ => None,
        }
      }
      _ => None,
    }
  }
  pub fn is_url_scheme(&self, parser: &Parser) -> bool {
    self.to_url_scheme(parser).is_some()
  }

  pub fn print(&self, parser: &Parser) {
    println!(
      "token lexeme: `{}` type: {:?}",
      parser.lexeme_str(self),
      self.token_type
    );
  }

  pub fn print_with(&self, prefix: &str, parser: &Parser) {
    print!("{} ", prefix);
    self.print(parser);
  }
}

pub trait TokenIs {
  fn is_url_scheme(&self, parser: &Parser) -> bool;
  fn is(&self, token_type: TokenType) -> bool;
  fn is_not(&self, token_type: TokenType) -> bool {
    !self.is(token_type)
  }
}

impl TokenIs for Option<&Token> {
  fn is(&self, token_type: TokenType) -> bool {
    self.map_or(false, |t| t.is(token_type))
  }
  fn is_url_scheme(&self, parser: &Parser) -> bool {
    self.map_or(false, |t| t.is_url_scheme(parser))
  }
}

// export const TOKEN = {
//   TEXT: `TEXT`,
//   ASTERISK: `ASTERISK`,
//   DOUBLE_ASTERISK: `DOUBLE_ASTERISK`,
//   TRIPLE_ASTERISK: `TRIPLE_ASTERISK`,
//   THEMATIC_BREAK: `THEMATIC_BREAK`,
//   STRAIGHT_DOUBLE_QUOTE: `STRAIGHT_DOUBLE_QUOTE`,
//   STRAIGHT_SINGLE_QUOTE: `STRAIGHT_SINGLE_QUOTE`,
//   DOUBLE_COLON: `DOUBLE_COLON`,
//   FORWARD_SLASH: `FORWARD_SLASH`,
//   PIPE: `PIPE`,
//   PLUS: `PLUS`,
//   HASH: `HASH`,
//   TRIPLE_PLUS: `TRIPLE_PLUS`,
//   QUADRUPLE_PLUS: `QUADRUPLE_PLUS`,
//   WHITESPACE: `WHITESPACE`,
//   DOUBLE_DASH: `DOUBLE_DASH`,
//   UNDERSCORE: `UNDERSCORE`,
//   LEFT_SINGLE_CURLY: `LEFT_SINGLE_CURLY`,
//   RIGHT_SINGLE_CURLY: `RIGHT_SINGLE_CURLY`,
//   LEFT_DOUBLE_CURLY: `LEFT_DOUBLE_CURLY`,
//   RIGHT_DOUBLE_CURLY: `RIGHT_DOUBLE_CURLY`,
//   LEFT_BRACKET: `LEFT_BRACKET`,
//   RIGHT_BRACKET: `RIGHT_BRACKET`,
//   XREF_OPEN: `XREF_OPEN`,
//   XREF_CLOSE: `XREF_CLOSE`,
//   LEFT_PARENS: `LEFT_PARENS`,
//   RIGHT_PARENS: `RIGHT_PARENS`,
//   FOOTNOTE_PREFIX: `FOOTNOTE_PREFIX`,
//   FOOTNOTE_STANZA: `FOOTNOTE_STANZA`,
//   FOOTNOTE_PARAGRAPH_SPLIT: `FOOTNOTE_PARAGRAPH_SPLIT`,
//   DEGREE_SYMBOL: `DEGREE_SYMBOL`,
//   POUND_SYMBOL: `POUND_SYMBOL`,
//   DOLLAR_SYMBOL: `DOLLAR_SYMBOL`,
//   ENTITY: `ENTITY`,
//   EQUALS: `EQUALS`,
//   COMMA: `COMMA`,
//   CARET: `CARET`,
//   BACKTICK: `BACKTICK`,
//   RAW_PASSTHROUGH: `RAW_PASSTHROUGH`,
//   DOT: `DOT`,
//   SEMICOLON: `SEMICOLON`,
//   COLON: `COLON`,
//   EXCLAMATION_MARK: `EXCLAMATION_MARK`,
//   QUESTION_MARK: `QUESTION_MARK`,
//   ILLEGAL: `ILLEGAL`,
//   EOL: `EOL`,
//   DOUBLE_EOL: `DOUBLE_EOL`,
//   EOF: `EOF`,
//   EOD: `EOD`, // end of (possibly multi-file) document
//   EOX: `EOX`, // special matcher token: `EOX` -- not technically a token type
// } as const;
