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
struct AttrState<'arena> {
  bump: &'arena Bump,
  attr_list: AttrList<'arena>,
  quotes: Quotes,
  attr: CollectText<'arena>,
  name: CollectText<'arena>,
  tokens: Deq<'arena, Token<'arena>>,
  kind: AttrKind,
  escaping: bool,
  parse_range: (u32, u32),
  formatted_text: bool,
  prev_token: Option<TokenKind>,
  is_legacy_anchor: bool,
}

impl<'arena> Parser<'arena> {
  /// Parse an attribute list.
  ///
  /// _NB: Caller is responsible for ensuring the line contains an attr list
  /// and also for consuming the open bracket before calling this function._
  pub(crate) fn parse_attr_list(&mut self, line: &mut Line<'arena>) -> Result<AttrList<'arena>> {
    self.parse_attrs(line, false)
  }

  /// Parse an attribute list for formatted (inline) text
  ///
  /// _NB: Caller is responsible for ensuring the line contains an attr list
  /// and also for consuming the open bracket before calling this function._
  pub(super) fn parse_formatted_text_attr_list(
    &mut self,
    line: &mut Line<'arena>,
  ) -> Result<AttrList<'arena>> {
    self.parse_attrs(line, true)
  }

  fn parse_attrs(
    &mut self,
    line: &mut Line<'arena>,
    formatted_text: bool,
  ) -> Result<AttrList<'arena>> {
    use AttrKind::*;
    use Quotes::*;
    let parse_start = line.current_token().unwrap().loc.start;
    let parse_end = line.first_nonescaped(CloseBracket).unwrap().0.loc.end - 1;
    let mut state = AttrState::new_in(self.bump, formatted_text, (parse_start, parse_end));

    if line.current_is(OpenBracket) {
      line.consume_current().unwrap(); // `[`
      state.attr_list.loc.end += 1;
      state.attr.loc.start += 1;
      state.is_legacy_anchor = true;
    }

    if line.current_is(CloseBracket) {
      let mut close = line.consume_current().unwrap(); // `]`
      if state.is_legacy_anchor {
        close = line.consume_current().unwrap(); // second `]`
      }
      return Ok(AttrList::new(
        SourceLocation::new(parse_start - 1, close.loc.end),
        self.bump,
      ));
    }

    while let Some(token) = line.consume_current() {
      let kind = token.kind;
      match kind {
        CloseBracket if state.quotes == Default && !state.escaping => {
          if state.is_legacy_anchor {
            line.consume_current().unwrap(); // second `]`
          }
          state.commit_prev(self)?;
          break;
        }
        CloseBracket if state.quotes != Default => {
          state.push_token(token);
          let parse_end = line.first_nonescaped(CloseBracket).unwrap().0.loc.end - 1;
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
        Dots
          if state.quotes == Default
            && token.len() == 1
            && (state.kind == Role || state.kind == Id || state.prev_token != Some(Word)) =>
        {
          state.commit_prev(self)?;
          state.kind = Role;
        }
        Hash if state.quotes == Default => {
          state.commit_prev(self)?;
          if state.attr_list.id.is_some() {
            self.err_token_start("More than one id attribute", &token)?
          }
          state.kind = Id;
        }
        Percent if state.quotes == Default && state.kind != Named => {
          state.commit_prev(self)?;
          state.kind = Option;
        }
        SingleQuote if state.at_transition() && state.quotes == Default => {
          state.skip_char();
          state.quotes = InSingle;
        }
        SingleQuote if state.quotes == InSingle => {
          state.commit_prev(self)?;
          state.skip_char();
          state.quotes = Default;
        }
        DoubleQuote if state.at_transition() && state.quotes == Default => {
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

impl<'arena> AttrState<'arena> {
  fn new_in(bump: &Bump, formatted_text: bool, parse_range: (u32, u32)) -> AttrState {
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
      tokens: Deq::new(bump),
      kind: AttrKind::Positional,
      escaping: false,
      parse_range,
      formatted_text,
      prev_token: None,
      is_legacy_anchor: false,
    }
  }

  fn skip_char(&mut self) {
    self.attr.loc = self.attr.loc.incr();
  }

  fn err_if_formatted(&self, parser: &Parser) -> Result<()> {
    if self.formatted_text {
      parser.err_at(
        "Formatted text only supports attribute shorthand: id, roles, & options",
        self.parse_range.0,
        self.parse_range.1,
      )?;
    }
    Ok(())
  }

  fn at_transition(&self) -> bool {
    if self.prev_token.is_none() || self.tokens.is_empty() {
      true
    } else {
      self.kind == AttrKind::Named && self.attr.is_empty()
    }
  }

  fn switch_to_named(&mut self) {
    std::mem::swap(&mut self.attr, &mut self.name);
    self.attr.loc = self.name.loc.incr_end().clamp_end(); // skip `=`
  }

  fn push_token(&mut self, token: Token<'arena>) {
    self.attr.push_token(&token);
    self.tokens.push(token);
  }

  fn commit_prev(&mut self, parser: &mut Parser<'arena>) -> Result<()> {
    use AttrKind::*;
    if !self.attr.is_empty() || self.kind == Named {
      match &self.kind {
        Positional
          if (self.is_legacy_anchor
            || self.tokens.iter().all(|t| t.is(Word) || t.is(Whitespace))) =>
        {
          self.err_if_formatted(parser)?;
          let src = self.attr.take_src();
          if self.is_legacy_anchor && self.attr_list.id.is_none() {
            self.attr_list.id = Some(src);
          } else {
            self.attr_list.positional.push(Some(
              bvec![in self.bump; InlineNode::new(Text(src.src), src.loc)].into(),
            ));
          }
        }
        Positional => {
          self.err_if_formatted(parser)?;
          self.attr.drop_src();
          let line = Line::new(std::mem::replace(&mut self.tokens, Deq::new(self.bump)));
          let inlines = parser.parse_inlines(&mut line.into_lines())?;
          self.attr_list.positional.push(Some(inlines));
        }
        Named => {
          self.err_if_formatted(parser)?;
          let name = self.name.take_src();
          if &name == "id" {
            self.attr_list.id = Some(self.attr.take_src());
          // special case: empty string for named, `foo=""`
          } else if self.tokens.len() == 1 {
            self.tokens.pop_front(); // remove name
            self
              .attr_list
              .named
              .insert(name, InlineNodes::new(self.bump));
          } else if self.tokens.len() > 1 {
            self.tokens.pop_front(); // remove name
            self.attr.drop_src();
            let line = Line::new(std::mem::replace(&mut self.tokens, Deq::new(self.bump)));
            let restore = parser.ctx.subs;
            parser.ctx.subs = if matches!(name.src.as_str(), "subs" | "cols") {
              Substitutions::none()
            } else {
              Substitutions::attr_value()
            };
            let inlines = parser.parse_inlines(&mut line.into_lines())?;
            parser.ctx.subs = restore;
            self.attr_list.named.insert(name, inlines);
          }
        }
        Role => {
          self.attr.loc = self.attr.loc.incr_start(); // skip `.`
          self.attr_list.roles.push(self.attr.take_src());
        }
        Id => {
          if self.attr_list.id.is_none() {
            self.attr.loc = self.attr.loc.incr_start(); // skip `#`
            self.attr_list.id = Some(self.attr.take_src());
          }
        }
        Option => {
          self.attr.loc = self.attr.loc.incr_start(); // skip `%`
          self.attr_list.options.push(self.attr.take_src());
        }
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
  use test_utils::*;

  #[test]
  fn test_parse_attr_list() {
    let cases = vec![
      ("[]", attr_list!(0..2)),
      (
        "[foo]",
        AttrList {
          positional: vecb![Some(nodes![node!("foo"; 1..4)])],
          ..attr_list!(0..5)
        },
      ),
      (
        "[foo bar]",
        AttrList {
          positional: vecb![Some(nodes![node!("foo bar"; 1..8)])],
          ..attr_list!(0..9)
        },
      ),
      (
        "[ foo bar ]",
        AttrList {
          positional: vecb![Some(nodes![node!("foo bar"; 2..9)])],
          ..attr_list!(0..11)
        },
      ),
      (
        "[ foo , bar ]",
        AttrList {
          positional: vecb![
            Some(nodes![node!("foo"; 2..5)]),
            Some(nodes![node!("bar"; 8..11)]),
          ],
          ..attr_list!(0..13)
        },
      ),
      (
        "[line-comment=%%]",
        AttrList {
          named: Named::from(vecb![(src!("line-comment", 1..13), just!("%%", 14..16))]),
          ..attr_list!(0..17)
        },
      ),
      (
        "[link=https://example.com]", // named, without quotes
        AttrList {
          named: Named::from(vecb![(
            src!("link", 1..5),
            just!("https://example.com", 6..25),
          )]),
          ..attr_list!(0..26)
        },
      ),
      (
        "[lines=1;3..4;6..-1]",
        AttrList {
          named: Named::from(vecb![(src!("lines", 1..6), just!("1;3..4;6..-1", 7..19),)]),
          ..attr_list!(0..20)
        },
      ),
      (
        "[link=\"https://example.com\"]",
        AttrList {
          named: Named::from(vecb![(
            src!("link", 1..5),
            just!("https://example.com", 7..26),
          )]),
          ..attr_list!(0..28)
        },
      ),
      (
        "[\\ ]", // keyboard macro
        AttrList {
          positional: vecb![Some(nodes![node!("\\"; 1..2)])],
          ..attr_list!(0..4)
        },
      ),
      (
        "[Ctrl+\\]]",
        AttrList {
          positional: vecb![Some(nodes![node!("Ctrl+]"; 1..8)])],
          ..attr_list!(0..9)
        },
      ),
      (
        "[#someid]",
        AttrList {
          id: Some(src!("someid", 2..8)),
          ..attr_list!(0..9)
        },
      ),
      (
        "[id=someid]",
        AttrList {
          id: Some(src!("someid", 4..10)),
          ..attr_list!(0..11)
        },
      ),
      (
        "[id=someid,]", // trailing comma allowed
        AttrList {
          id: Some(src!("someid", 4..10)),
          ..attr_list!(0..12)
        },
      ),
      (
        "[#someid.nowrap]",
        AttrList {
          id: Some(src!("someid", 2..8)),
          roles: vecb![src!("nowrap", 9..15)],
          ..attr_list!(0..16)
        },
      ),
      (
        "[.nowrap]",
        AttrList {
          roles: vecb![src!("nowrap", 2..8)],
          ..attr_list!(0..9)
        },
      ),
      (
        "[.nowrap.underline]",
        AttrList {
          roles: vecb![src!("nowrap", 2..8), src!("underline", 9..18)],
          ..attr_list!(0..19)
        },
      ),
      (
        "[foo,bar]",
        AttrList {
          positional: vecb![
            Some(nodes![node!("foo"; 1..4)]),
            Some(nodes![node!("bar"; 5..8)]),
          ],
          ..attr_list!(0..9)
        },
      ),
      (
        "[foo,bar,a=b]",
        AttrList {
          positional: vecb![
            Some(nodes![node!("foo"; 1..4)]),
            Some(nodes![node!("bar"; 5..8)]),
          ],
          named: Named::from(vecb![(src!("a", 9..10), just!("b", 11..12))]),
          ..attr_list!(0..13)
        },
      ),
      (
        "[a=b,foo,b=c,bar]",
        AttrList {
          positional: vecb![
            Some(nodes![node!("foo"; 5..8)]),
            Some(nodes![node!("bar"; 13..16)]),
          ],
          named: Named::from(vecb![
            (src!("a", 1..2), just!("b", 3..4)),
            (src!("b", 9..10), just!("c", 11..12)),
          ]),
          ..attr_list!(0..17)
        },
      ),
      (
        "[\"foo,bar\",baz]",
        AttrList {
          positional: vecb![
            Some(nodes![node!("foo,bar"; 2..9)]),
            Some(nodes![node!("baz"; 11..14)]),
          ],
          ..attr_list!(0..15)
        },
      ),
      (
        "[Sunset,300,400]",
        AttrList {
          positional: vecb![
            Some(nodes![node!("Sunset"; 1..7)]),
            Some(nodes![node!("300"; 8..11)]),
            Some(nodes![node!("400"; 12..15)]),
          ],
          ..attr_list!(0..16)
        },
      ),
      (
        "[alt=Sunset,width=300,height=400]",
        AttrList {
          named: Named::from(vecb![
            (src!("alt", 1..4), just!("Sunset", 5..11)),
            (src!("width", 12..17), just!("300", 18..21)),
            (src!("height", 22..28), just!("400", 29..32)),
          ]),
          ..attr_list!(0..33)
        },
      ),
      (
        "[#custom-id,named=\"value of named\"]",
        AttrList {
          id: Some(src!("custom-id", 2..11)),
          named: Named::from(vecb![(
            src!("named", 12..17),
            just!("value of named", 19..33),
          )]),
          ..attr_list!(0..35)
        },
      ),
      (
        "[foo, bar]",
        AttrList {
          positional: vecb![
            Some(nodes![node!("foo"; 1..4)]),
            Some(nodes![node!("bar"; 6..9)]),
          ],
          ..attr_list!(0..10)
        },
      ),
      (
        "[,bar]",
        AttrList {
          positional: vecb![None, Some(nodes![node!("bar"; 2..5)])],
          ..attr_list!(0..6)
        },
      ),
      (
        "[ , bar]",
        AttrList {
          positional: vecb![None, Some(nodes![node!("bar"; 4..7)])],
          ..attr_list!(0..8)
        },
      ),
      (
        "[, , bar]",
        AttrList {
          positional: vecb![None, None, Some(nodes![node!("bar"; 5..8)])],
          ..attr_list!(0..9)
        },
      ),
      (
        "[\"foo]\"]",
        AttrList {
          positional: vecb![Some(nodes![node!("foo]"; 2..6)])],
          ..attr_list!(0..8)
        },
      ),
      (
        "[foo\\]]",
        AttrList {
          positional: vecb![Some(nodes![node!("foo]"; 1..6)])],
          ..attr_list!(0..7)
        },
      ),
      (
        "[foo='bar']",
        AttrList {
          named: Named::from(vecb![(src!("foo", 1..4), just!("bar", 6..9))]),
          ..attr_list!(0..11)
        },
      ),
      (
        "[foo='.foo#id%opt']",
        AttrList {
          named: Named::from(vecb![(src!("foo", 1..4), just!(".foo#id%opt", 6..17))]),
          ..attr_list!(0..19)
        },
      ),
      (
        "[foo=\"\"]",
        AttrList {
          named: Named::from(vecb![(src!("foo", 1..4), InlineNodes::new(leaked_bump()))]),
          ..attr_list!(0..8)
        },
      ),
      (
        "[width=50%]",
        AttrList {
          named: Named::from(vecb![(src!("width", 1..6), just!("50%", 7..10))]),
          ..attr_list!(0..11)
        },
      ),
      (
        "[don't]",
        AttrList {
          positional: vecb![Some(nodes![
            node!("don"; 1..4),
            node!(CurlyQuote(CurlyKind::LegacyImplicitApostrophe), 4..5),
            node!("t"; 5..6),
          ])],
          ..attr_list!(0..7)
        },
      ),
      (
        "[don\"t]",
        AttrList {
          positional: vecb![Some(nodes![node!("don\"t"; 1..6),])],
          ..attr_list!(0..7)
        },
      ),
    ];
    for (input, expected) in cases {
      let mut parser = test_parser!(input);
      let mut line = parser.read_line().unwrap().unwrap();
      line.discard(1); // `[`
      let attr_list = parser.parse_attr_list(&mut line).unwrap();
      expect_eq!(attr_list, expected, from: input);
    }
  }

  #[test]
  fn test_parse_legacy_attr_list() {
    let cases = vec![
      (
        "[[foo]]",
        AttrList {
          id: Some(src!("foo", 2..5)),
          ..attr_list!(0..7)
        },
      ),
      (
        "[[f.o]]",
        AttrList {
          id: Some(src!("f.o", 2..5)),
          ..attr_list!(0..7)
        },
      ),
      (
        "[[foo,bar]]",
        AttrList {
          id: Some(src!("foo", 2..5)),
          positional: vecb![Some(nodes![node!("bar"; 6..9)])],
          ..attr_list!(0..11)
        },
      ),
      ("[[]]", AttrList { ..attr_list!(0..4) }),
    ];

    for (input, expected) in cases {
      let mut parser = test_parser!(input);
      let mut line = parser.read_line().unwrap().unwrap();
      line.discard(1); // `[`
      let attr_list = parser.parse_attr_list(&mut line).unwrap();
      expect_eq!(attr_list, expected);
    }
  }

  #[test]
  fn test_parse_attr_list_errs() {
    let cases = vec![
      (
        "[#foo#bar]",
        true,
        error! {"
           --> test.adoc:1:6
            |
          1 | [#foo#bar]
            |      ^ More than one id attribute
        "},
      ),
      (
        "[#foo#bar]",
        false,
        error! {"
           --> test.adoc:1:6
            |
          1 | [#foo#bar]
            |      ^ More than one id attribute
        "},
      ),
      (
        "[foobar]",
        true,
        error! {"
           --> test.adoc:1:2
            |
          1 | [foobar]
            |  ^^^^^^ Formatted text only supports attribute shorthand: id, roles, & options
        "},
      ),
      (
        "[#a,b=c]",
        true,
        error! {"
           --> test.adoc:1:2
            |
          1 | [#a,b=c]
            |  ^^^^^^ Formatted text only supports attribute shorthand: id, roles, & options
        "},
      ),
    ];

    for (input, formatted, expected) in cases {
      let mut parser = test_parser!(input);
      let mut line = parser.read_line().unwrap().unwrap();
      line.discard(1); // `[`
      let diag = parser.parse_attrs(&mut line, formatted).err().unwrap();
      expect_eq!(diag.plain_text(), expected, from: input);
    }
  }
}
