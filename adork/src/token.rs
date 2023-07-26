#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Token {
  pub token_type: TokenType,
  pub start: usize,
  pub end: usize,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TokenType {
  Colon,
  CommentBlock,
  CommentLine,
  EqualSigns,
  GreaterThan,
  LessThan,
  Newline,
  SemiColon,
  Whitespace,
  Word,
}

impl Token {
  pub fn new(token_type: TokenType, start: usize, end: usize) -> Token {
    Token {
      token_type,
      start,
      end,
    }
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

  pub fn ends_block(&self) -> bool {
    match self {
      Token {
        token_type: TokenType::Newline,
        start,
        end,
      } if end - start > 1 => true,
      _ => false,
    }
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
