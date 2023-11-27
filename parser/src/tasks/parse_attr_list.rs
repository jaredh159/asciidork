use bumpalo::Bump;

use crate::ast::*;
use crate::line::Line;
use crate::tasks::text::Text;
use crate::token::TokenKind::*;
use crate::{Parser, Result};

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
struct AttrState<'bmp> {
  attr_list: AttrList<'bmp>,
  quotes: Quotes,
  attr: Text<'bmp>,
  name: Text<'bmp>,
  kind: AttrKind,
  escaping: bool,
  parse_range: (usize, usize),
  formatted_text: bool,
}

impl<'bmp, 'src> Parser<'bmp, 'src> {
  /// Parse an attribute list.
  ///
  /// _NB: Caller is responsible for ensuring the line contains an attr list
  /// and also for consuming the open bracket before calling this function._
  pub(super) fn parse_attr_list(&self, line: &mut Line) -> Result<AttrList<'bmp>> {
    self.parse_attrs(line, false)
  }

  /// Parse an attribute list for formatted (inline) text
  ///
  /// _NB: Caller is responsible for ensuring the line contains an attr list
  /// and also for consuming the open bracket before calling this function._
  pub(super) fn parse_formatted_text_attr_list(&self, line: &mut Line) -> Result<AttrList<'bmp>> {
    self.parse_attrs(line, true)
  }

  fn parse_attrs(&self, line: &mut Line, formatted_text: bool) -> Result<AttrList<'bmp>> {
    debug_assert!(!line.current_is(OpenBracket));
    debug_assert!(line.contains_nonescaped(CloseBracket));

    if line.current_is(CloseBracket) {
      line.discard(1); // `]`
      return Ok(AttrList::new_in(self.bump));
    }

    use AttrKind::*;
    use Quotes::*;
    let parse_start = line.current_token().unwrap().loc.start;
    let parse_end = line.first_nonescaped(CloseBracket).unwrap().loc.end - 1;
    let mut state = AttrState::new_in(self.bump, formatted_text, (parse_start, parse_end));

    while let Some(token) = line.consume_current() {
      match token.kind {
        CloseBracket if state.quotes == Default && !state.escaping => {
          state.commit_prev(self)?;
          break;
        }
        Backslash if line.current_is(Whitespace) => {
          state.attr.push_token(&token);
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
        EqualSigns if state.quotes == Default && token.lexeme.len() == 1 => {
          std::mem::swap(&mut state.attr, &mut state.name);
          state.kind = Named;
        }
        Whitespace if state.quotes == Default => {}
        _ => state.attr.push_token(&token),
      }
      state.escaping = false;
    }
    Ok(state.attr_list)
  }
}

impl<'bmp> AttrState<'bmp> {
  fn new_in(bump: &Bump, formatted_text: bool, parse_range: (usize, usize)) -> AttrState {
    AttrState {
      attr_list: AttrList::new_in(bump),
      quotes: Quotes::Default,
      attr: Text::new_in(bump),
      name: Text::new_in(bump),
      kind: AttrKind::Positional,
      escaping: false,
      parse_range,
      formatted_text,
    }
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

  fn commit_prev(&mut self, parser: &Parser) -> Result<()> {
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
  use super::*;
  use bumpalo::collections::String;
  use bumpalo::vec as bvec;
  use bumpalo::Bump;

  macro_rules! s {
    (in $bump:expr;$s:expr) => {
      String::from_str_in($s, $bump)
    };
  }

  #[test]
  fn test_parse_attr_list() {
    let b = &Bump::new();
    let cases = vec![
      (
        "[foo]",
        AttrList {
          positional: bvec![in b; s!(in b; "foo")],
          ..AttrList::new_in(b)
        },
      ),
      (
        "[\\ ]", // keyboard macro
        AttrList {
          positional: bvec![in b; s!(in b; "\\")],
          ..AttrList::new_in(b)
        },
      ),
      (
        "[Ctrl+\\]]",
        AttrList {
          positional: bvec![in b; s!(in b; "Ctrl+]")],
          ..AttrList::new_in(b)
        },
      ),
      (
        "[#someid]",
        AttrList {
          id: Some(s!(in b; "someid")),
          ..AttrList::new_in(b)
        },
      ),
      (
        "[.nowrap]",
        AttrList {
          roles: bvec![in b; s!(in b; "nowrap")],
          ..AttrList::new_in(b)
        },
      ),
      (
        "[.nowrap.underline]",
        AttrList {
          roles: bvec![in b; s!(in b; "nowrap"), s!(in b; "underline")],
          ..AttrList::new_in(b)
        },
      ),
      (
        "[foo,bar]",
        AttrList {
          positional: bvec![in b; s!(in b; "foo"), s!(in b; "bar")],
          ..AttrList::new_in(b)
        },
      ),
      (
        "[foo,bar,a=b]",
        AttrList {
          positional: bvec![in b; s!(in b; "foo"), s!(in b;"bar")],
          named: Named::from(bvec![in b; (s!(in b;"a"), s!(in b; "b"))]),
          ..AttrList::new_in(b)
        },
      ),
      (
        "[a=b,foo,b=c,bar]",
        AttrList {
          positional: bvec![in b; s!(in b; "foo"), s!(in b; "bar")],
          named: Named::from(
            bvec![in b; (s!(in b; "a"), s!(in b; "b")), (s!(in b; "b"), s!(in b; "c"))],
          ),
          ..AttrList::new_in(b)
        },
      ),
      (
        "[\"foo,bar\",baz]",
        AttrList {
          positional: bvec![in b; s!(in b; "foo,bar"), s!(in b; "baz")],
          ..AttrList::new_in(b)
        },
      ),
      (
        "[Sunset,300,400]",
        AttrList {
          positional: bvec![in b; s!(in b; "Sunset"), s!(in b; "300"), s!(in b; "400")],
          ..AttrList::new_in(b)
        },
      ),
      (
        "[alt=Sunset,width=300,height=400]",
        AttrList {
          named: Named::from(bvec![in b;
            (s!(in b; "alt"), s!(in b; "Sunset")),
            (s!(in b; "width"), s!(in b; "300")),
            (s!(in b; "height"), s!(in b; "400")),
          ]),
          ..AttrList::new_in(b)
        },
      ),
      (
        "[#custom-id,named=\"value of named\"]",
        AttrList {
          id: Some(s!(in b; "custom-id")),
          named: Named::from(bvec![in b;(s!(in b; "named"), s!(in b; "value of named"))]),
          ..AttrList::new_in(b)
        },
      ),
      (
        "[foo, bar]",
        AttrList {
          positional: bvec![in b; s!(in b; "foo"), s!(in b; "bar")],
          ..AttrList::new_in(b)
        },
      ),
      (
        "[\"foo]\"]",
        AttrList {
          positional: bvec![in b; s!(in b; "foo]")],
          ..AttrList::new_in(b)
        },
      ),
      (
        "[foo\\]]",
        AttrList {
          positional: bvec![in b; s!(in b; "foo]")],
          ..AttrList::new_in(b)
        },
      ),
      (
        "[foo='bar']",
        AttrList {
          named: Named::from(bvec![in b; (s!(in b; "foo"), s!(in b; "bar"))]),
          ..AttrList::new_in(b)
        },
      ),
      (
        "[foo='.foo#id%opt']",
        AttrList {
          named: Named::from(bvec![in b; (s!(in b; "foo"), s!(in b; ".foo#id%opt"))]),
          ..AttrList::new_in(b)
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
