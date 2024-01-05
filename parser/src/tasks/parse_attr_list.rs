use crate::internal::*;
use crate::variants::token::*;
use ast::variants::inline::*;

#[derive(Debug, PartialEq, Eq)]
enum AttrKind {
  Positional,
  Named,
  Role,
  Id,
  Option,
}

#[derive(Debug, PartialEq, Eq)]
enum Quotes {
  Default,
  InDouble,
  InSingle,
}

#[derive(Debug)]
struct AttrState<'bmp: 'src, 'src> {
  bump: &'bmp Bump,
  attr_list: AttrList<'bmp>,
  quotes: Quotes,
  attr: CollectText<'bmp>,
  name: CollectText<'bmp>,
  tokens: Vec<'bmp, Token<'src>>,
  kind: AttrKind,
  escaping: bool,
  parse_range: (usize, usize),
  formatted_text: bool,
  prev_token: Option<TokenKind>,
}

impl<'bmp, 'src> Parser<'bmp, 'src> {
  /// Parse an attribute list.
  ///
  /// _NB: Caller is responsible for ensuring the line contains an attr list
  /// and also for consuming the open bracket before calling this function._
  pub(super) fn parse_attr_list(&mut self, line: &mut Line<'bmp, 'src>) -> Result<AttrList<'bmp>> {
    self.parse_attrs(line, false)
  }

  /// Parse an attribute list for formatted (inline) text
  ///
  /// _NB: Caller is responsible for ensuring the line contains an attr list
  /// and also for consuming the open bracket before calling this function._
  pub(super) fn parse_formatted_text_attr_list(
    &mut self,
    line: &mut Line<'bmp, 'src>,
  ) -> Result<AttrList<'bmp>> {
    self.parse_attrs(line, true)
  }

  fn parse_attrs(
    &mut self,
    line: &mut Line<'bmp, 'src>,
    formatted_text: bool,
  ) -> Result<AttrList<'bmp>> {
    debug_assert!(!line.current_is(OpenBracket));
    debug_assert!(line.contains_nonescaped(CloseBracket));

    if line.current_is(CloseBracket) {
      let close = line.consume_current().unwrap(); // `]`
      return Ok(AttrList::new(close.loc.decr_start(), self.bump));
    }

    use AttrKind::*;
    use Quotes::*;
    let parse_start = line.current_token().unwrap().loc.start;
    let parse_end = line.first_nonescaped(CloseBracket).unwrap().loc.end - 1;
    let mut state = AttrState::new_in(self.bump, formatted_text, (parse_start, parse_end));

    while let Some(token) = line.consume_current() {
      let kind = token.kind;
      match kind {
        CloseBracket if state.quotes == Default && !state.escaping => {
          state.commit_prev(self)?;
          break;
        }
        CloseBracket if state.quotes != Default => {
          state.push_token(token);
          let parse_end = line.first_nonescaped(CloseBracket).unwrap().loc.end - 1;
          state.parse_range.1 = parse_end;
          state.attr_list.loc.end = parse_end + 1;
        }
        Backslash if line.current_is(Whitespace) => {
          state.push_token(token);
        }
        Backslash if state.quotes == Default => {
          state.escaping = true;
          continue;
        }
        Dot
          if state.quotes == Default
            && (state.kind == Role || state.kind == Id || state.prev_token != Some(Word)) =>
        {
          state.commit_prev(self)?;
          state.kind = Role;
        }
        Hash if state.quotes == Default => {
          state.commit_prev(self)?;
          if state.attr_list.id.is_some() {
            self.err("more than one id attribute", Some(&token))?
          }
          state.kind = Id;
        }
        Percent if state.quotes == Default => {
          state.commit_prev(self)?;
          state.kind = Option;
        }
        SingleQuote if state.quotes == Default => {
          state.skip_char();
          state.quotes = InSingle;
        }
        SingleQuote if state.quotes == InSingle => {
          state.commit_prev(self)?;
          state.skip_char();
          state.quotes = Default;
        }
        DoubleQuote if state.quotes == Default => {
          state.skip_char();
          state.quotes = InDouble;
        }
        DoubleQuote if state.quotes == InDouble => {
          state.commit_prev(self)?;
          state.skip_char();
          state.quotes = Default;
        }
        Comma
          if state.quotes == Default
            && (state.prev_token.is_none() || state.prev_token == Some(Comma)) =>
        {
          state.skip_positional();
          state.kind = Positional;
        }
        Comma if state.quotes == Default => {
          state.commit_prev(self)?;
          state.skip_char();
          state.kind = Positional;
        }
        EqualSigns if state.quotes == Default && token.lexeme.len() == 1 => {
          state.switch_to_named();
          state.kind = Named;
        }
        Whitespace if state.quotes == Default => {
          // skip leading and trailing whitespace
          if state.attr.is_empty() || token.loc.end == parse_end || line.current_is(Comma) {
            state.commit_prev(self)?;
            state.skip_char();
          } else {
            state.push_token(token);
          }
        }
        _ => state.push_token(token),
      }

      state.escaping = false;

      // don't consider "insignificant" whitespace as a previous token
      if kind != Whitespace || state.quotes != Default {
        state.prev_token = Some(kind)
      }
    }
    Ok(state.attr_list)
  }
}

impl<'bmp, 'src> AttrState<'bmp, 'src> {
  fn new_in(bump: &Bump, formatted_text: bool, parse_range: (usize, usize)) -> AttrState {
    let start_loc = SourceLocation::new(parse_range.0, parse_range.0);
    AttrState {
      bump,
      attr_list: AttrList::new(
        SourceLocation::new(parse_range.0 - 1, parse_range.1 + 1),
        bump,
      ),
      quotes: Quotes::Default,
      attr: CollectText::new_in(start_loc, bump),
      name: CollectText::new_in(start_loc, bump),
      tokens: Vec::new_in(bump),
      kind: AttrKind::Positional,
      escaping: false,
      parse_range,
      formatted_text,
      prev_token: None,
    }
  }

  fn skip_char(&mut self) {
    self.attr.loc = self.attr.loc.incr();
  }

  fn err_if_formatted(&self, parser: &Parser) -> Result<()> {
    if self.formatted_text {
      parser.err_at(
        "formatted text only supports attribute shorthand: id, roles, & options",
        self.parse_range.0,
        self.parse_range.1,
      )?;
    }
    Ok(())
  }

  fn switch_to_named(&mut self) {
    std::mem::swap(&mut self.attr, &mut self.name);
    self.attr.loc = self.name.loc.incr_end().clamp_end(); // skip `=`
  }

  fn push_token(&mut self, token: Token<'src>) {
    self.attr.push_token(&token);
    self.tokens.push(token);
  }

  fn commit_prev(&mut self, parser: &mut Parser<'bmp, 'src>) -> Result<()> {
    use AttrKind::*;
    if !self.attr.is_empty() {
      match &self.kind {
        // could be optimized to not call parse_inlines more exhaustively by
        // tracking every type of token that would indicate we need to parse
        Positional if self.tokens.iter().all(|t| t.is(Word) || t.is(Whitespace)) => {
          self.err_if_formatted(parser)?;
          let src = self.attr.take_src();
          self.attr_list.positional.push(Some(
            bvec![in self.bump; InlineNode::new(Text(src.src), src.loc)],
          ));
        }
        Positional => {
          self.err_if_formatted(parser)?;
          let tokens = std::mem::replace(&mut self.tokens, Vec::new_in(self.bump));
          let line = parser.line_from(tokens, self.attr.take_src().loc);
          let inlines = parser.parse_inlines(&mut line.into_lines_in(self.bump))?;
          self.attr_list.positional.push(Some(inlines));
        }
        Named => {
          self.err_if_formatted(parser)?;
          self
            .attr_list
            .named
            .insert(self.name.take_src(), self.attr.take_src());
        }
        Role => {
          self.attr.loc = self.attr.loc.incr_start(); // skip `.`
          self.attr_list.roles.push(self.attr.take_src())
        }
        Id => {
          if self.attr_list.id.is_none() {
            self.attr.loc = self.attr.loc.incr_start(); // skip `#`
            self.attr_list.id = Some(self.attr.take_src())
          }
        }
        Option => self.attr_list.options.push(self.attr.take_src()),
      }
      self.tokens.clear();
    }
    Ok(())
  }

  fn skip_positional(&mut self) {
    self.attr_list.positional.push(None);
    self.skip_char();
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::test::*;

  #[test]
  fn test_parse_attr_list() {
    let b = &Bump::new();
    let cases = vec![
      ("[]", AttrList::new(l(0, 2), b)),
      (
        "[foo]",
        AttrList {
          positional: b.vec([Some(b.vec([n_text("foo", 1, 4, b)]))]),
          ..AttrList::new(l(0, 5), b)
        },
      ),
      (
        "[foo bar]",
        AttrList {
          positional: b.vec([Some(b.vec([n_text("foo bar", 1, 8, b)]))]),
          ..AttrList::new(l(0, 9), b)
        },
      ),
      (
        "[ foo bar ]",
        AttrList {
          positional: b.vec([Some(b.vec([n_text("foo bar", 2, 9, b)]))]),
          ..AttrList::new(l(0, 11), b)
        },
      ),
      (
        "[ foo , bar ]",
        AttrList {
          positional: b.vec([
            Some(b.vec([n_text("foo", 2, 5, b)])),
            Some(b.vec([n_text("bar", 8, 11, b)])),
          ]),
          ..AttrList::new(l(0, 13), b)
        },
      ),
      (
        "[link=https://example.com]", // named, without quotes
        AttrList {
          named: Named::from(b.vec([(
            b.src("link", l(1, 5)),
            b.src("https://example.com", l(6, 25)),
          )])),
          ..AttrList::new(l(0, 26), b)
        },
      ),
      (
        "[link=\"https://example.com\"]",
        AttrList {
          named: Named::from(b.vec([(
            b.src("link", l(1, 5)),
            b.src("https://example.com", l(7, 26)),
          )])),
          ..AttrList::new(l(0, 28), b)
        },
      ),
      (
        "[\\ ]", // keyboard macro
        AttrList {
          positional: b.vec([Some(b.vec([n_text("\\", 1, 2, b)]))]),
          ..AttrList::new(l(0, 4), b)
        },
      ),
      (
        "[Ctrl+\\]]",
        AttrList {
          positional: b.vec([Some(b.vec([n_text("Ctrl+]", 1, 8, b)]))]),
          ..AttrList::new(l(0, 9), b)
        },
      ),
      (
        "[#someid]",
        AttrList {
          id: Some(b.src("someid", l(2, 8))),
          ..AttrList::new(l(0, 9), b)
        },
      ),
      (
        "[#someid.nowrap]",
        AttrList {
          id: Some(b.src("someid", l(2, 8))),
          roles: b.vec([b.src("nowrap", l(9, 15))]),
          ..AttrList::new(l(0, 16), b)
        },
      ),
      (
        "[.nowrap]",
        AttrList {
          roles: b.vec([b.src("nowrap", l(2, 8))]),
          ..AttrList::new(l(0, 9), b)
        },
      ),
      (
        "[.nowrap.underline]",
        AttrList {
          roles: b.vec([b.src("nowrap", l(2, 8)), b.src("underline", l(9, 18))]),
          ..AttrList::new(l(0, 19), b)
        },
      ),
      (
        "[foo,bar]",
        AttrList {
          positional: b.vec([
            Some(b.vec([n_text("foo", 1, 4, b)])),
            Some(b.vec([n_text("bar", 5, 8, b)])),
          ]),
          ..AttrList::new(l(0, 9), b)
        },
      ),
      (
        "[foo,bar,a=b]",
        AttrList {
          positional: b.vec([
            Some(b.vec([n_text("foo", 1, 4, b)])),
            Some(b.vec([n_text("bar", 5, 8, b)])),
          ]),
          named: Named::from(b.vec([(b.src("a", l(9, 10)), b.src("b", l(11, 12)))])),
          ..AttrList::new(l(0, 13), b)
        },
      ),
      (
        "[a=b,foo,b=c,bar]",
        AttrList {
          positional: b.vec([
            Some(b.vec([n_text("foo", 5, 8, b)])),
            Some(b.vec([n_text("bar", 13, 16, b)])),
          ]),
          named: Named::from(b.vec([
            (b.src("a", l(1, 2)), b.src("b", l(3, 4))),
            (b.src("b", l(9, 10)), b.src("c", l(11, 12))),
          ])),
          ..AttrList::new(l(0, 17), b)
        },
      ),
      (
        "[\"foo,bar\",baz]",
        AttrList {
          positional: b.vec([
            Some(b.vec([n_text("foo,bar", 2, 9, b)])),
            Some(b.vec([n_text("baz", 11, 14, b)])),
          ]),
          ..AttrList::new(l(0, 15), b)
        },
      ),
      (
        "[Sunset,300,400]",
        AttrList {
          positional: b.vec([
            Some(b.vec([n_text("Sunset", 1, 7, b)])),
            Some(b.vec([n_text("300", 8, 11, b)])),
            Some(b.vec([n_text("400", 12, 15, b)])),
          ]),
          ..AttrList::new(l(0, 16), b)
        },
      ),
      (
        "[alt=Sunset,width=300,height=400]",
        AttrList {
          named: Named::from(b.vec([
            (b.src("alt", l(1, 4)), b.src("Sunset", l(5, 11))),
            (b.src("width", l(12, 17)), b.src("300", l(18, 21))),
            (b.src("height", l(22, 28)), b.src("400", l(29, 32))),
          ])),
          ..AttrList::new(l(0, 33), b)
        },
      ),
      (
        "[#custom-id,named=\"value of named\"]",
        AttrList {
          id: Some(b.src("custom-id", l(2, 11))),
          named: Named::from(b.vec([(
            b.src("named", l(12, 17)),
            b.src("value of named", l(19, 33)),
          )])),
          ..AttrList::new(l(0, 35), b)
        },
      ),
      (
        "[foo, bar]",
        AttrList {
          positional: b.vec([
            Some(b.vec([n_text("foo", 1, 4, b)])),
            Some(b.vec([n_text("bar", 6, 9, b)])),
          ]),
          ..AttrList::new(l(0, 10), b)
        },
      ),
      (
        "[,bar]",
        AttrList {
          positional: b.vec([None, Some(b.vec([n_text("bar", 2, 5, b)]))]),
          ..AttrList::new(l(0, 6), b)
        },
      ),
      (
        "[ , bar]",
        AttrList {
          positional: b.vec([None, Some(b.vec([n_text("bar", 4, 7, b)]))]),
          ..AttrList::new(l(0, 8), b)
        },
      ),
      (
        "[, , bar]",
        AttrList {
          positional: b.vec([None, None, Some(b.vec([n_text("bar", 5, 8, b)]))]),
          ..AttrList::new(l(0, 9), b)
        },
      ),
      (
        "[\"foo]\"]",
        AttrList {
          positional: b.vec([Some(b.vec([n_text("foo]", 2, 6, b)]))]),
          ..AttrList::new(l(0, 8), b)
        },
      ),
      (
        "[foo\\]]",
        AttrList {
          positional: b.vec([Some(b.vec([n_text("foo]", 1, 6, b)]))]),
          ..AttrList::new(l(0, 7), b)
        },
      ),
      (
        "[foo='bar']",
        AttrList {
          named: Named::from(b.vec([(b.src("foo", l(1, 4)), b.src("bar", l(6, 9)))])),
          ..AttrList::new(l(0, 11), b)
        },
      ),
      (
        "[foo='.foo#id%opt']",
        AttrList {
          named: Named::from(b.vec([(b.src("foo", l(1, 4)), b.src(".foo#id%opt", l(6, 17)))])),
          ..AttrList::new(l(0, 19), b)
        },
      ),
    ];
    for (input, expected) in cases {
      let mut parser = Parser::new(b, input);
      let mut line = parser.read_line().unwrap();
      line.discard(1); // `[`
      let attr_list = parser.parse_attr_list(&mut line).unwrap();
      assert_eq!(attr_list, expected);
    }
  }

  #[test]
  fn test_parse_attr_list_errs() {
    let cases = vec![
      ("[#foo#bar]", false, "more than one id", 5, 1),
      ("[#foo#bar]", true, "more than one id", 5, 1),
      ("[foobar]", true, "only supports attribute shorthand", 1, 6),
      (
        "[#lol,rofl=copter]",
        true,
        "only supports attribute shorthand",
        1,
        16,
      ),
    ];

    let b = &Bump::new();
    for (input, formatted, expected, start, width) in cases {
      let mut parser = Parser::new(b, input);
      let mut line = parser.read_line().unwrap();
      line.discard(1); // `[`
      let result = parser.parse_attrs(&mut line, formatted);
      if let Err(diag) = result {
        assert!(diag.message.contains(expected));
        assert_eq!(diag.underline_start, start);
        assert_eq!(diag.underline_width, width);
      } else {
        panic!("expected error, got {:?}", result.unwrap());
      }
    }
  }
}
