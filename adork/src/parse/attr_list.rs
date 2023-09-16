use crate::ast::AttrList;
use crate::parse::utils::Text;
use crate::parse::{Parser, Result};
use crate::tok::{self, TokenType::*};

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
  InDoubleQuotes,
  InSingleQuotes,
}

#[derive(Debug)]
struct AttrState {
  attr_list: AttrList,
  quotes: Quotes,
  attr: Text,
  name: Text,
  kind: AttrKind,
  escaping: bool,
  parse_range: (usize, usize),
  formatted_text: bool,
}

impl Parser {
  /// Parse an attribute list.
  ///
  /// _NB: Caller is responsible for ensuring the line contains an attr list
  /// and also for consuming the open bracket before calling this function._
  pub(super) fn parse_attr_list(&mut self, line: &mut tok::Line) -> Result<AttrList> {
    self.parse_attrs(line, false)
  }

  /// Parse an attribute list for formatted (inline) text
  ///
  /// _NB: Caller is responsible for ensuring the line contains an attr list
  /// and also for consuming the open bracket before calling this function._
  pub(super) fn parse_formatted_text_attr_list(
    &mut self,
    line: &mut tok::Line,
  ) -> Result<AttrList> {
    self.parse_attrs(line, true)
  }

  fn parse_attrs(&mut self, line: &mut tok::Line, formatted_text: bool) -> Result<AttrList> {
    debug_assert!(!line.current_is(OpenBracket));
    debug_assert!(line.contains_nonescaped(CloseBracket));

    if line.current_is(CloseBracket) {
      line.discard(1); // `]`
      return Ok(AttrList::new());
    }

    use AttrKind::*;
    use Quotes::*;
    let parse_start = line.current_token().unwrap().start;
    let parse_end = line.first_nonescaped(CloseBracket).unwrap().end - 1;
    let mut state = AttrState::new(formatted_text, (parse_start, parse_end));

    while let Some(token) = line.consume_current() {
      match token.token_type {
        CloseBracket if state.quotes == Default && !state.escaping => {
          state.commit_prev(self)?;
          break;
        }
        Backslash if line.current_is(Whitespace) => {
          state.attr.push_token(&token, self);
        }
        Backslash if state.quotes == Default => {
          state.escaping = true;
          continue;
        }
        Dot if state.quotes == Default => {
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
        SingleQuote if state.quotes == Default => state.quotes = InSingleQuotes,
        SingleQuote if state.quotes == InSingleQuotes => state.quotes = Default,
        DoubleQuote if state.quotes == Default => state.quotes = InDoubleQuotes,
        DoubleQuote if state.quotes == InDoubleQuotes => state.quotes = Default,
        Comma if state.quotes == Default => {
          state.commit_prev(self)?;
          state.kind = Positional;
        }
        EqualSigns if state.quotes == Default && token.len() == 1 => {
          std::mem::swap(&mut state.attr, &mut state.name);
          state.kind = Named;
        }
        Whitespace if state.quotes == Default => {}
        _ => state.attr.push_token(&token, self),
      }
      state.escaping = false;
    }

    Ok(state.attr_list)
  }
}

impl AttrState {
  fn new(formatted_text: bool, parse_range: (usize, usize)) -> AttrState {
    AttrState {
      attr_list: AttrList::new(),
      quotes: Quotes::Default,
      attr: Text::new(),
      name: Text::new(),
      kind: AttrKind::Positional,
      escaping: false,
      parse_range,
      formatted_text,
    }
  }

  fn err_if_formatted(&mut self, parser: &mut Parser) -> Result<()> {
    if self.formatted_text {
      parser.err_at(
        "formatted text only supports attribute shorthand: id, roles, & options",
        self.parse_range.0,
        self.parse_range.1,
      )?;
    }
    Ok(())
  }

  fn commit_prev(&mut self, parser: &mut Parser) -> Result<()> {
    use AttrKind::*;
    if !self.attr.is_empty() {
      match &self.kind {
        Positional => {
          self.err_if_formatted(parser)?;
          self.attr_list.positional.push(self.attr.take())
        }
        Named => {
          self.err_if_formatted(parser)?;
          self
            .attr_list
            .named
            .insert(self.name.take(), self.attr.take());
        }
        Role => self.attr_list.roles.push(self.attr.take()),
        Id => {
          if self.attr_list.id.is_none() {
            self.attr_list.id = Some(self.attr.take())
          }
        }
        Option => self.attr_list.options.push(self.attr.take()),
      }
    }
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use std::collections::HashMap;

  use crate::ast::AttrList;
  use crate::t::*;

  #[test]
  fn test_parse_attr_list() {
    let cases = vec![
      (
        "[foo]",
        AttrList {
          positional: vec![s("foo")],
          ..AttrList::new()
        },
      ),
      (
        "[\\ ]", // keyboard macro
        AttrList {
          positional: vec![s("\\")],
          ..AttrList::new()
        },
      ),
      (
        "[Ctrl+\\]]",
        AttrList {
          positional: vec![s("Ctrl+]")],
          ..AttrList::new()
        },
      ),
      (
        "[#someid]",
        AttrList {
          id: Some(s("someid")),
          ..AttrList::new()
        },
      ),
      (
        "[.nowrap]",
        AttrList {
          roles: vec![s("nowrap")],
          ..AttrList::new()
        },
      ),
      (
        "[.nowrap.underline]",
        AttrList {
          roles: vec![s("nowrap"), s("underline")],
          ..AttrList::new()
        },
      ),
      (
        "[foo,bar]",
        AttrList {
          positional: vec![s("foo"), s("bar")],
          ..AttrList::new()
        },
      ),
      (
        "[foo,bar,a=b]",
        AttrList {
          positional: vec![s("foo"), s("bar")],
          named: HashMap::from([(s("a"), s("b"))]),
          ..AttrList::new()
        },
      ),
      (
        "[a=b,foo,b=c,bar]",
        AttrList {
          positional: vec![s("foo"), s("bar")],
          named: HashMap::from([(s("a"), s("b")), (s("b"), s("c"))]),
          ..AttrList::new()
        },
      ),
      (
        "[\"foo,bar\",baz]",
        AttrList {
          positional: vec![s("foo,bar"), s("baz")],
          ..AttrList::new()
        },
      ),
      (
        "[Sunset,300,400]",
        AttrList {
          positional: vec![s("Sunset"), s("300"), s("400")],
          ..AttrList::new()
        },
      ),
      (
        "[alt=Sunset,width=300,height=400]",
        AttrList {
          named: HashMap::from([
            (s("alt"), s("Sunset")),
            (s("width"), s("300")),
            (s("height"), s("400")),
          ]),
          ..AttrList::new()
        },
      ),
      (
        "[#custom-id,named=\"value of named\"]",
        AttrList {
          id: Some(s("custom-id")),
          named: HashMap::from([(s("named"), s("value of named"))]),
          ..AttrList::new()
        },
      ),
      (
        "[foo, bar]",
        AttrList {
          positional: vec![s("foo"), s("bar")],
          ..AttrList::new()
        },
      ),
      (
        "[\"foo]\"]",
        AttrList {
          positional: vec![s("foo]")],
          ..AttrList::new()
        },
      ),
      (
        "[foo\\]]",
        AttrList {
          positional: vec![s("foo]")],
          ..AttrList::new()
        },
      ),
      (
        "[foo='bar']",
        AttrList {
          named: HashMap::from([(s("foo"), s("bar"))]),
          ..AttrList::new()
        },
      ),
      (
        "[foo='.foo#id%opt']",
        AttrList {
          named: HashMap::from([(s("foo"), s(".foo#id%opt"))]),
          ..AttrList::new()
        },
      ),
    ];
    for (input, expected) in cases {
      let (mut line, mut parser) = line_test(input);
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
    for (input, formatted, expected, start, width) in cases {
      let (mut line, mut parser) = line_test(input);
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
