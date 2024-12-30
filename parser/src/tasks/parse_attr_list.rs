use crate::internal::*;
use crate::token::TokenSpec::*;
use crate::variants::token::*;

impl<'arena> Parser<'arena> {
  pub(super) fn parse_inline_anchor(
    &mut self,
    line: &mut Line<'arena>,
  ) -> Result<Option<AnchorSrc<'arena>>> {
    self.parse_anchor(line, false)
  }

  pub(crate) fn parse_block_anchor(
    &mut self,
    line: &mut Line<'arena>,
  ) -> Result<Option<AnchorSrc<'arena>>> {
    self.parse_anchor(line, true)
  }

  fn parse_anchor(
    &mut self,
    line: &mut Line<'arena>,
    is_entire_line: bool,
  ) -> Result<Option<AnchorSrc<'arena>>> {
    let current = line.current_token().unwrap();
    if matches!(
      current.kind,
      SingleQuote | DoubleQuote | Whitespace | CloseBracket | Digits
    ) {
      return Ok(None);
    }
    let start = current.loc.start - 2;
    let id = line.consume_to_string_until_one_of(&[Kind(Comma), Kind(CloseBracket)], self.bump);
    let mut anchor = AnchorSrc {
      loc: SourceLocation::new(start, id.loc.end),
      reftext: None,
      id,
    };
    if line.current_is(CloseBracket) {
      line.discard(2);
      anchor.loc.end += 2;
      return Ok(Some(anchor));
    }
    line.discard_assert(Comma);
    let reftext_line = if is_entire_line {
      let mut reftext = Line::new(Deq::new(self.bump));
      std::mem::swap(&mut reftext, line);
      let last = reftext.pop().unwrap();
      let penult = reftext.pop().unwrap();
      anchor.loc.end = last.loc.end;
      assert!(last.kind == CloseBracket && penult.kind == CloseBracket);
      reftext
    } else {
      let mut reftext = Line::new(Deq::with_capacity(8, self.bump));
      while !line.current_is(CloseBracket) {
        reftext.push(line.consume_current().unwrap());
      }
      line.discard_assert(CloseBracket);
      let last = line.consume_current().unwrap();
      assert!(last.kind == CloseBracket);
      anchor.loc.end = last.loc.end;
      reftext
    };
    let reftext = self.parse_inlines(&mut reftext_line.into_lines())?;
    anchor.reftext = Some(reftext);
    Ok(Some(anchor))
  }

  pub(crate) fn parse_biblio_anchor(
    &mut self,
    _line: &mut Line<'arena>,
  ) -> Result<AnchorSrc<'arena>> {
    todo!("biblio anchor")
  }

  pub(crate) fn parse_block_attr_list(
    &mut self,
    line: &mut Line<'arena>,
  ) -> Result<AttrList<'arena>> {
    self._parse_attr_list(line, true, false)
  }

  pub(crate) fn parse_inline_attr_list(
    &mut self,
    line: &mut Line<'arena>,
  ) -> Result<AttrList<'arena>> {
    self._parse_attr_list(line, false, false)
  }

  pub(super) fn parse_formatted_text_attr_list(
    &mut self,
    line: &mut Line<'arena>,
  ) -> Result<AttrList<'arena>> {
    self._parse_attr_list(line, false, true)
  }

  pub(crate) fn parse_link_macro_attr_list(
    &mut self,
    line: &mut Line<'arena>,
  ) -> Result<AttrList<'arena>> {
    let mut in_double_quote = false;
    let mut last_kind = TokenKind::Eof;
    let mut parse_as_attr_list = false;
    let mut in_nested_attr_list = false;
    let mut num_tokens = 0;
    for token in line.iter().take(line.len() - 1) {
      num_tokens += 1;
      match (last_kind, token.kind) {
        (Backslash, CloseBracket) if in_nested_attr_list => in_nested_attr_list = false,
        (Backslash, CloseBracket | OpenBracket | DoubleQuote) => {}
        (_, DoubleQuote) => in_double_quote = !in_double_quote,
        (_, OpenBracket) => in_nested_attr_list = true,
        (_, CloseBracket) if in_nested_attr_list => in_nested_attr_list = false,
        (_, CloseBracket) => {
          num_tokens -= 1;
          break;
        }
        (_, EqualSigns) if token.len() == 1 && !in_double_quote => {
          parse_as_attr_list = true;
          break;
        }
        _ => {}
      }
      last_kind = token.kind;
    }
    if parse_as_attr_list {
      return self._parse_attr_list(line, false, false);
    }

    let mut attrs = AttrList::new(line.loc().unwrap().decr_start(), self.bump);
    if line.current_is(CloseBracket) {
      let end_bracket = line.consume_current().unwrap();
      attrs.loc.extend(end_bracket.loc);
      return Ok(attrs);
    }

    let mut tokens = Deq::with_capacity(line.len() - 1, self.bump);
    for _ in 0..num_tokens {
      tokens.push(line.consume_current().unwrap());
    }
    let attr_line = Line::new(unquote(tokens));
    let nodes = self.parse_inlines(&mut attr_line.into_lines())?;
    attrs.positional.push(Some(nodes));
    debug_assert!(line.current_is(CloseBracket));
    let close_bracket = line.consume_current().expect("attr list close bracket");
    attrs.loc.extend(close_bracket.loc);
    Ok(attrs)
  }

  fn _parse_attr_list(
    &mut self,
    line: &mut Line<'arena>,
    full_line: bool,
    formatted_text: bool,
  ) -> Result<AttrList<'arena>> {
    let mut attrs = AttrList::new(line.loc().unwrap().decr_start(), self.bump);

    if line.current_is(CloseBracket) {
      let end_bracket = line.consume_current().unwrap();
      attrs.loc.extend(end_bracket.loc);
      return Ok(attrs);
    }

    // all attrs except last
    let delimiters = self.find_delimiters(line);
    for num_tokens in delimiters.into_iter() {
      let mut tokens = Deq::with_capacity(num_tokens, self.bump);
      for _ in 0..num_tokens {
        tokens.push(line.consume_current().unwrap());
      }
      self.parse_attr(tokens, formatted_text, &mut attrs)?;
      line.discard_assert(Comma);
    }

    // last attr
    let mut tokens = Deq::with_capacity(line.len() - 1, self.bump);

    if full_line {
      while line.peek_token().is_some() {
        tokens.push(line.consume_current().unwrap());
      }
    } else if !line.current_is(CloseBracket) {
      while !line.peek_token().kind(CloseBracket) || line.current_is(Backslash) {
        tokens.push(line.consume_current().unwrap());
      }
      tokens.push(line.consume_current().unwrap());
    }

    self.parse_attr(tokens, formatted_text, &mut attrs)?;

    let close_bracket = line.consume_current().expect("attr list close bracket");
    attrs.loc.extend(close_bracket.loc);

    Ok(attrs)
  }

  fn parse_attr(
    &mut self,
    tokens: Deq<'arena, Token<'arena>>,
    formatted_text: bool,
    attr_list: &mut AttrList<'arena>,
  ) -> Result<()> {
    let tokens = trim(tokens);
    if tokens.is_empty() {
      attr_list.positional.push(None);
      return Ok(());
    }
    match self.attr_ir(tokens) {
      AttrIr::Positional(tokens, _) | AttrIr::Id(tokens) if formatted_text => {
        self.err_at(
          ONLY_SHORTHAND_ERR,
          tokens.first().unwrap().loc.start,
          tokens.last().unwrap().loc.end,
        )?;
      }
      AttrIr::Options(groups) | AttrIr::Roles(groups) if formatted_text => {
        self.err_at(
          ONLY_SHORTHAND_ERR,
          groups.first().unwrap().first().unwrap().loc.start,
          groups.last().unwrap().last().unwrap().loc.end,
        )?;
      }
      AttrIr::Named(name, tokens) if formatted_text => {
        self.err_at(
          ONLY_SHORTHAND_ERR,
          name.loc.start,
          tokens.last().unwrap().loc.end,
        )?;
      }
      AttrIr::Positional(mut tokens, with_shorthand) => {
        if with_shorthand {
          let mut pos_tokens = Deq::new(self.bump);
          while !matches!(tokens.first().unwrap().kind, Hash | Percent | Dots) {
            pos_tokens.push(tokens.pop_front().unwrap());
          }
          attr_list
            .positional
            .push(self.parse_attr_nodes(pos_tokens)?);
          self.parse_attr(tokens, formatted_text, attr_list)?;
        } else {
          attr_list.positional.push(self.parse_attr_nodes(tokens)?);
        }
      }
      AttrIr::Named(name, tokens) => {
        let restore = self.ctx.subs;
        // TODO: is this correct?
        self.ctx.subs = if matches!(name.src.as_str(), "subs" | "cols") {
          Substitutions::none()
        } else {
          Substitutions::attr_value()
        };
        let nodes = self.parse_attr_nodes(tokens)?.unwrap();
        self.ctx.subs = restore;
        attr_list.insert_named(name, nodes);
      }
      AttrIr::Options(groups) => self.push_attr_groups(groups, &mut attr_list.options),
      AttrIr::Roles(groups) => self.push_attr_groups(groups, &mut attr_list.roles),
      AttrIr::Id(tokens) => {
        let mut line = Line::new(tokens);
        let src = line.consume_to_string(self.bump);
        if attr_list.id.is_some() {
          self.err_at("More than one id attribute", src.loc.start, src.loc.end)?;
        } else {
          attr_list.id = Some(src);
        }
      }
      AttrIr::Shorthand(tokens) => {
        debug_assert!(tokens.len() > 1);
        let mut line = Line::new(tokens);
        let stop: &[TokenSpec] = &[Kind(Hash), Kind(Percent), Len(1, Dots)];
        loop {
          let Some(token) = line.consume_current() else {
            break;
          };
          match token.kind {
            Dots if token.len() == 1 => attr_list
              .roles
              .push(line.consume_to_string_until_one_of(stop, self.bump)),
            Hash => {
              let src = line.consume_to_string_until_one_of(stop, self.bump);
              if attr_list.id.is_some() {
                self.err_token_start("More than one id attribute", &token)?
              } else {
                attr_list.id = Some(src);
              }
            }
            Percent => attr_list
              .options
              .push(line.consume_to_string_until_one_of(stop, self.bump)),
            _ => unreachable!("Parser::parse_attr"),
          }
        }
      }
    }
    Ok(())
  }

  fn parse_attr_nodes(
    &mut self,
    mut tokens: Deq<'arena, Token<'arena>>,
  ) -> Result<Option<InlineNodes<'arena>>> {
    if tokens.len() == 1 && tokens.first().kind(Word) {
      let first = tokens.pop_front().unwrap();
      let mut nodes = InlineNodes::new(self.bump);
      nodes.push(InlineNode::new(
        Inline::Text(self.string(&first.lexeme)),
        first.loc,
      ));
      Ok(Some(nodes))
    } else if tokens.is_empty() {
      Ok(Some(InlineNodes::new(self.bump)))
    } else {
      let line = Line::new(tokens);
      Ok(Some(self.parse_inlines(&mut line.into_lines())?))
    }
  }

  fn find_delimiters(&self, line: &Line<'arena>) -> BumpVec<'arena, usize> {
    let mut delimiters = BumpVec::with_capacity_in(5, self.bump);
    let mut double_start_len: Option<usize> = None;
    let mut single_start_len: Option<usize> = None;
    let mut num_tokens = 0;
    for token in line.iter().take(line.len() - 1) {
      num_tokens += 1;
      match (token.kind, double_start_len, single_start_len) {
        (DoubleQuote, None, _) => {
          double_start_len = Some(delimiters.len());
        }
        (DoubleQuote, Some(prev_len), _) => {
          while delimiters.len() > prev_len {
            num_tokens += delimiters.pop().unwrap(); // tokens before skipped comma
            num_tokens += 1; // comma
          }
          double_start_len = None;
        }
        (SingleQuote, _, None) => {
          single_start_len = Some(delimiters.len());
        }
        (SingleQuote, _, Some(prev_len)) => {
          delimiters.truncate(prev_len);
          single_start_len = None;
        }
        (Comma, _, _) => {
          delimiters.push(num_tokens - 1);
          num_tokens = 0;
        }
        _ => {}
      }
    }
    delimiters
  }

  fn parse_key_value_attr(&self, mut tokens: Deq<'arena, Token<'arena>>) -> AttrIr<'arena> {
    let first = tokens.pop_front().unwrap();
    let mut name = self.string(&first.lexeme);
    let mut name_loc = first.loc;
    loop {
      let next = tokens.pop_front().unwrap();
      match next.kind {
        EqualSigns => break,
        Whitespace => {}
        _ => {
          name.push_str(&next.lexeme);
          name_loc.extend(next.loc);
        }
      }
    }
    if tokens.first().kind(Whitespace) {
      tokens.remove_first();
    }
    match name.as_str() {
      "options" | "opts" => AttrIr::Options(self.parse_attr_subgroups(Comma, unquote(tokens))),
      "role" => AttrIr::Roles(self.parse_attr_subgroups(Whitespace, unquote(tokens))),
      "id" => AttrIr::Id(unquote(tokens)),
      _ => AttrIr::Named(SourceString::new(name, name_loc), unquote(tokens)),
    }
  }

  fn parse_attr_subgroups(
    &self,
    delimiter: TokenKind,
    tokens: Deq<'arena, Token<'arena>>,
  ) -> Deq<'arena, Deq<'arena, Token<'arena>>> {
    let mut groups = Deq::new(self.bump);
    let mut current = Deq::new(self.bump);
    for token in tokens.into_iter() {
      if token.kind == delimiter {
        let trimmed = trim(current);
        if !trimmed.is_empty() {
          groups.push(trimmed);
        }
        current = Deq::new(self.bump);
      } else {
        current.push(token);
      }
    }
    let trimmed = trim(current);
    if !trimmed.is_empty() {
      groups.push(trimmed);
    }
    groups
  }

  fn attr_ir(&self, tokens: Deq<'arena, Token<'arena>>) -> AttrIr<'arena> {
    enum Kind {
      Shorthand,
      Positional,
      KeyValue,
    }
    let mut saw_shorthand_symbol = false;
    let kind = tokens.iter().enumerate().fold(None, |acc, (i, token)| {
      if acc.is_some() {
        return acc;
      }
      match token.kind {
        Dots | Hash | Percent if i == 0 && token.len() == 1 && tokens.len() > 1 => {
          Some(Kind::Shorthand)
        }
        Dots | Hash | Percent if token.len() == 1 => {
          saw_shorthand_symbol = true;
          acc
        }
        DoubleQuote | SingleQuote => Some(Kind::Positional),
        Whitespace if !tokens.get(i + 1).matches(EqualSigns, "=") => Some(Kind::Positional),
        EqualSigns if i == 0 => Some(Kind::Positional),
        EqualSigns if token.len() == 1 => Some(Kind::KeyValue),
        _ => acc,
      }
    });
    match kind {
      Some(Kind::Shorthand) => AttrIr::Shorthand(tokens),
      Some(Kind::KeyValue) => self.parse_key_value_attr(tokens),
      _ => {
        let orig_len = tokens.len();
        let tokens = unquote(tokens);
        if saw_shorthand_symbol && tokens.len() == orig_len {
          AttrIr::Positional(tokens, true)
        } else {
          AttrIr::Positional(tokens, false)
        }
      }
    }
  }

  fn push_attr_groups(
    &self,
    groups: Deq<'arena, Deq<'arena, Token<'arena>>>,
    sink: &mut BumpVec<'arena, SourceString<'arena>>,
  ) {
    for mut group in groups.into_iter() {
      if group.is_empty() {
        continue;
      } else if group.len() == 1 {
        let token = group.pop_front().unwrap();
        let src = token.into_source_string();
        sink.push(src);
      } else {
        let mut line = Line::new(group);
        sink.push(line.consume_to_string(self.bump));
      }
    }
  }
}

#[derive(Debug)]
enum AttrIr<'a> {
  Positional(Deq<'a, Token<'a>>, bool),
  Named(SourceString<'a>, Deq<'a, Token<'a>>),
  Options(Deq<'a, Deq<'a, Token<'a>>>),
  Roles(Deq<'a, Deq<'a, Token<'a>>>),
  Shorthand(Deq<'a, Token<'a>>),
  Id(Deq<'a, Token<'a>>),
}

#[derive(Debug, PartialEq, Eq)]
pub struct AnchorSrc<'arena> {
  pub id: SourceString<'arena>,
  pub reftext: Option<InlineNodes<'arena>>,
  pub loc: SourceLocation,
}

fn unquote<'a>(mut tokens: Deq<'a, Token<'a>>) -> Deq<'a, Token<'a>> {
  if tokens.len() > 1
    && (tokens.first().kind(DoubleQuote) && tokens.last().kind(DoubleQuote)
      || tokens.first().kind(SingleQuote) && tokens.last().kind(SingleQuote))
  {
    tokens.remove_first();
    tokens.pop();
  }
  tokens
}

fn trim<'a>(mut tokens: Deq<'a, Token<'a>>) -> Deq<'a, Token<'a>> {
  while tokens.first().kind(Whitespace) {
    tokens.remove_first();
  }
  while tokens.last().kind(Whitespace) {
    tokens.pop();
  }
  tokens
}

const ONLY_SHORTHAND_ERR: &str =
  "Formatted text only supports attribute shorthand: id, roles, & options";

#[cfg(test)]
mod tests {
  use super::*;
  use test_utils::*;

  // replicates nearly all tests from: asciidoctor/test/attribute_list_test.rb
  #[test]
  fn test_parse_attr_list_asciidoctor() {
    let cases = vec![
      ("[]", attr_list!(0..2)),
      (
        "['']",
        AttrList {
          positional: vecb![Some(nodes![])],
          ..attr_list!(0..4)
        },
      ),
      (
        "[foo]",
        AttrList {
          positional: vecb![Some(nodes![node!("foo"; 1..4)])],
          ..attr_list!(0..5)
        },
      ),
      (
        "[\"foo\"]",
        AttrList {
          positional: vecb![Some(nodes![node!("foo"; 2..5)])],
          ..attr_list!(0..7)
        },
      ),
      (
        "['foo']",
        AttrList {
          positional: vecb![Some(nodes![node!("foo"; 2..5)])],
          ..attr_list!(0..7)
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
        "[\"ba\\\"zaar\"]",
        AttrList {
          positional: vecb![Some(nodes![
            node!("ba"; 2..4),
            node!(Inline::Discarded, 4..5),
            node!("\"zaar"; 5..10),
          ])],
          ..attr_list!(0..12)
        },
      ),
      (
        "['ba\\'zaar']",
        AttrList {
          positional: vecb![Some(nodes![
            node!("ba"; 2..4),
            node!(Inline::Discarded, 4..5),
            node!("'zaar"; 5..10),
          ])],
          ..attr_list!(0..12)
        },
      ),
      (
        "[']",
        AttrList {
          positional: vecb![Some(nodes![node!("'"; 1..2)])],
          ..attr_list!(0..3)
        },
      ),
      (
        "[=foo=]",
        AttrList {
          positional: vecb![Some(nodes![node!("=foo="; 1..6)])],
          ..attr_list!(0..7)
        },
      ),
      (
        "[foo , ]",
        AttrList {
          positional: vecb![Some(nodes![node!("foo"; 1..4)]), None],
          ..attr_list!(0..8)
        },
      ),
      (
        "[, foo]",
        AttrList {
          positional: vecb![None, Some(nodes![node!("foo"; 3..6)])],
          ..attr_list!(0..7)
        },
      ),
      (
        "[, foo bar]",
        AttrList {
          positional: vecb![None, Some(nodes![node!("foo bar"; 3..10)])],
          ..attr_list!(0..11)
        },
      ),
      (
        "[first, second one, third]",
        AttrList {
          positional: vecb![
            Some(nodes![node!("first"; 1..6)]),
            Some(nodes![node!("second one"; 8..18)]),
            Some(nodes![node!("third"; 20..25)]),
          ],
          ..attr_list!(0..26)
        },
      ),
      (
        "[first,,third,]",
        AttrList {
          positional: vecb![
            Some(nodes![node!("first"; 1..6)]),
            None,
            Some(nodes![node!("third"; 8..13)]),
            None,
          ],
          ..attr_list!(0..15)
        },
      ),
      // named
      (
        "[foo=']",
        AttrList {
          positional: vecb![None],
          named: Named::from(vecb![(src!("foo", 1..4), just!("'", 5..6))]),
          ..attr_list!(0..7)
        },
      ),
      (
        "[foo=\"]",
        AttrList {
          positional: vecb![None],
          named: Named::from(vecb![(src!("foo", 1..4), just!("\"", 5..6))]),
          ..attr_list!(0..7)
        },
      ),
      (
        "[foo=bar]",
        AttrList {
          positional: vecb![None],
          named: Named::from(vecb![(src!("foo", 1..4), just!("bar", 5..8))]),
          ..attr_list!(0..9)
        },
      ),
      (
        "[foo=\"bar\"]",
        AttrList {
          positional: vecb![None],
          named: Named::from(vecb![(src!("foo", 1..4), just!("bar", 6..9))]),
          ..attr_list!(0..11)
        },
      ),
      (
        "[foo='bar']",
        AttrList {
          positional: vecb![None],
          named: Named::from(vecb![(src!("foo", 1..4), just!("bar", 6..9))]),
          ..attr_list!(0..11)
        },
      ),
      (
        "[foo=]",
        AttrList {
          positional: vecb![None],
          named: Named::from(vecb![(src!("foo", 1..4), nodes![])]),
          ..attr_list!(0..6)
        },
      ),
      (
        "[foo=,bar=baz]",
        AttrList {
          positional: vecb![None, None],
          named: Named::from(vecb![
            (src!("foo", 1..4), nodes![]),
            (src!("bar", 6..9), just!("baz", 10..13)),
          ]),
          ..attr_list!(0..14)
        },
      ),
      (
        "[height=100,caption=\"\",link=\"images/octocat.png\"]",
        AttrList {
          positional: vecb![None, None, None],
          named: Named::from(vecb![
            (src!("height", 1..7), just!("100", 8..11)),
            // NB: asciidoctor parses the value as an empty string, not sure if matters...
            (src!("caption", 12..19), nodes![]),
            (src!("link", 23..27), just!("images/octocat.png", 29..47)),
          ]),
          ..attr_list!(0..49)
        },
      ),
      (
        "[height=100,caption='',link='images/octocat.png']", // <-- single quotes
        AttrList {
          positional: vecb![None, None, None],
          named: Named::from(vecb![
            (src!("height", 1..7), just!("100", 8..11)),
            // NB: asciidoctor parses the value as an empty string, not sure if matters...
            (src!("caption", 12..19), nodes![]),
            (src!("link", 23..27), just!("images/octocat.png", 29..47)),
          ]),
          ..attr_list!(0..49)
        },
      ),
      (
        "[first=value, second=two, third=3]",
        AttrList {
          positional: vecb![None, None, None],
          named: Named::from(vecb![
            (src!("first", 1..6), just!("value", 7..12)),
            (src!("second", 14..20), just!("two", 21..24)),
            (src!("third", 26..31), just!("3", 32..33)),
          ]),
          ..attr_list!(0..34)
        },
      ),
      (
        "[first='value', second=\"value two\", third=three]",
        AttrList {
          positional: vecb![None, None, None],
          named: Named::from(vecb![
            (src!("first", 1..6), just!("value", 8..13)),
            (src!("second", 16..22), just!("value two", 24..33)),
            (src!("third", 36..41), just!("three", 42..47)),
          ]),
          ..attr_list!(0..48)
        },
      ),
      (
        "[     first    =     'value', second     =\"value two\"     , third=       three   ]",
        AttrList {
          positional: vecb![None, None, None],
          named: Named::from(vecb![
            (src!("first", 6..11), just!("value", 22..27)),
            (src!("second", 30..36), just!("value two", 43..52)),
            (src!("third", 60..65), just!("three", 73..78)),
          ]),
          ..attr_list!(0..82)
        },
      ),
      (
        "[first, second=\"value two\", third=three, Sherlock Holmes]",
        AttrList {
          positional: vecb![
            Some(nodes![node!("first"; 1..6)]),
            None, // named 1
            None, // named 2, this is weird, but trying to match asciidoctor
            Some(nodes![node!("Sherlock Holmes"; 41..56)]),
          ],
          named: Named::from(vecb![
            (src!("second", 8..14), just!("value two", 16..25)),
            (src!("third", 28..33), just!("three", 34..39)),
          ]),
          ..attr_list!(0..57)
        },
      ),
      (
        "[first,,third=,,fifth=five]",
        AttrList {
          positional: vecb![
            Some(nodes![node!("first"; 1..6)]),
            None, // second pos, explicitly none because `,,`
            None, // third pos, None because Named 1 is in that "spot"
            None, // fourth pos, explicitly none because `,,`
            None, // fifth pos, None because Named 2 is in that "spot"
          ],
          named: Named::from(vecb![
            (src!("third", 8..13), nodes![]),
            (src!("fifth", 16..21), just!("five", 22..26)),
          ]),
          ..attr_list!(0..27)
        },
      ),
      (
        "[quote, options='opt1,,opt2 , opt3 foo']",
        AttrList {
          positional: vecb![Some(nodes![node!("quote"; 1..6)])],
          options: vecb![
            src!("opt1", 17..21),
            src!("opt2", 23..27),
            src!("opt3 foo", 30..38),
          ],
          ..attr_list!(0..40)
        },
      ),
      (
        "[quote, opts=\"opt1,,opt2 , opt3\"]",
        AttrList {
          positional: vecb![Some(nodes![node!("quote"; 1..6)])],
          options: vecb![
            src!("opt1", 14..18),
            src!("opt2", 20..24),
            src!("opt3", 27..31),
          ],
          ..attr_list!(0..33)
        },
      ),
      (
        "[quote, opts=]",
        AttrList {
          positional: vecb![Some(nodes![node!("quote"; 1..6)])],
          ..attr_list!(0..14)
        },
      ),
    ];
    for (input, expected) in cases {
      // parse as block
      let mut block_parser = test_parser!(input);
      let mut line = block_parser.read_line().unwrap().unwrap();
      line.discard_assert(TokenKind::OpenBracket);
      let attr_list = block_parser.parse_block_attr_list(&mut line).unwrap();
      expect_eq!(attr_list, expected, from: input);
      // parse as inline
      let mut inline_input = String::from(input);
      inline_input.push_str("foo bar");
      let mut inline_parser = test_parser!(&inline_input);
      let mut line = inline_parser.read_line().unwrap().unwrap();
      line.discard_assert(TokenKind::OpenBracket);
      let attr_list = inline_parser.parse_inline_attr_list(&mut line).unwrap();
      expect_eq!(attr_list, expected, from: input);
      expect_eq!("foo bar", &line.reassemble_src(), from: input);
    }
  }

  #[test]
  fn test_parse_attr_list_more() {
    let cases = vec![
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
          positional: vecb![None],
          named: Named::from(vecb![(src!("line-comment", 1..13), just!("%%", 14..16))]),
          ..attr_list!(0..17)
        },
      ),
      (
        "[link=https://example.com]", // named, without quotes
        AttrList {
          positional: vecb![None],
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
          positional: vecb![None],
          named: Named::from(vecb![(src!("lines", 1..6), just!("1;3..4;6..-1", 7..19),)]),
          ..attr_list!(0..20)
        },
      ),
      (
        "[link=\"https://example.com\"]",
        AttrList {
          positional: vecb![None],
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
          positional: vecb![Some(nodes![
            node!("Ctrl+"; 1..6),
            // TODO: check this renders correctly in dr-html
            node!(Inline::Discarded, 6..7),
            node!("]"; 7..8),
          ])],
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
          positional: vecb![None],
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
        "[role=nowrap]",
        AttrList {
          roles: vecb![src!("nowrap", 6..12)],
          ..attr_list!(0..13)
        },
      ),
      (
        "[role=nowrap,]",
        AttrList {
          positional: vecb![None],
          roles: vecb![src!("nowrap", 6..12)],
          ..attr_list!(0..14)
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
        "[role=nowrap underline]",
        AttrList {
          roles: vecb![src!("nowrap", 6..12), src!("underline", 13..22)],
          ..attr_list!(0..23)
        },
      ),
      (
        "[role=\"nowrap underline\"]",
        AttrList {
          roles: vecb![src!("nowrap", 7..13), src!("underline", 14..23)],
          ..attr_list!(0..25)
        },
      ),
      (
        "[foo,bar,a=b]",
        AttrList {
          positional: vecb![
            Some(nodes![node!("foo"; 1..4)]),
            Some(nodes![node!("bar"; 5..8)]),
            None,
          ],
          named: Named::from(vecb![(src!("a", 9..10), just!("b", 11..12))]),
          ..attr_list!(0..13)
        },
      ),
      (
        "[a=b,foo,b=c,bar]",
        AttrList {
          positional: vecb![
            None,
            Some(nodes![node!("foo"; 5..8)]),
            None,
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
          positional: vecb![None, None, None],
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
          positional: vecb![None],
          id: Some(src!("custom-id", 2..11)),
          named: Named::from(vecb![(
            src!("named", 12..17),
            just!("value of named", 19..33),
          )]),
          ..attr_list!(0..35)
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
        "[foo\\]]",
        AttrList {
          positional: vecb![Some(nodes![
            node!("foo"; 1..4),
            node!(Inline::Discarded, 4..5),
            node!("]"; 5..6),
          ])],
          ..attr_list!(0..7)
        },
      ),
      (
        "[foo='.foo#id%opt']",
        AttrList {
          positional: vecb![None],
          named: Named::from(vecb![(src!("foo", 1..4), just!(".foo#id%opt", 6..17))]),
          ..attr_list!(0..19)
        },
      ),
      (
        "[width=50%]",
        AttrList {
          positional: vecb![None],
          named: Named::from(vecb![(src!("width", 1..6), just!("50%", 7..10))]),
          ..attr_list!(0..11)
        },
      ),
      (
        "[don't]",
        AttrList {
          positional: vecb![Some(nodes![
            node!("don"; 1..4),
            node!(
              Inline::CurlyQuote(CurlyKind::LegacyImplicitApostrophe),
              4..5
            ),
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
      (
        "[%header%footer%autowidth]",
        AttrList {
          options: vecb![
            src!("header", 2..8),
            src!("footer", 9..15),
            src!("autowidth", 16..25),
          ],
          ..attr_list!(0..26)
        },
      ),
      (
        "[example%collapsible]",
        AttrList {
          positional: vecb![Some(just!("example", 1..8))],
          options: vecb![src!("collapsible", 9..20)],
          ..attr_list!(0..21)
        },
      ),
      (
        "[example#collapsible]",
        AttrList {
          positional: vecb![Some(just!("example", 1..8))],
          id: Some(src!("collapsible", 9..20)),
          ..attr_list!(0..21)
        },
      ),
      (
        "[example#coll_psible.cust-class]",
        AttrList {
          positional: vecb![Some(just!("example", 1..8))],
          id: Some(src!("coll_psible", 9..20)),
          roles: vecb![src!("cust-class", 21..31)],
          ..attr_list!(0..32)
        },
      ),
    ];
    for (input, expected) in cases {
      // parse as block
      let mut block_parser = test_parser!(input);
      let mut line = block_parser.read_line().unwrap().unwrap();
      line.discard_assert(TokenKind::OpenBracket);
      let attr_list = block_parser.parse_block_attr_list(&mut line).unwrap();
      expect_eq!(attr_list, expected, from: input);
      // parse as inline
      let mut inline_input = String::from(input);
      inline_input.push_str("foo bar");
      let mut inline_parser = test_parser!(&inline_input);
      let mut line = inline_parser.read_line().unwrap().unwrap();
      line.discard_assert(TokenKind::OpenBracket);
      let attr_list = inline_parser.parse_inline_attr_list(&mut line).unwrap();
      expect_eq!(attr_list, expected, from: input);
      expect_eq!("foo bar", &line.reassemble_src(), from: input);
    }
  }

  #[test]
  fn test_parse_attr_list_block_only() {
    let cases = vec![(
      "[\"foo]\"]", // invalid for inline
      AttrList {
        positional: vecb![Some(nodes![node!("foo]"; 2..6)])],
        ..attr_list!(0..8)
      },
    )];
    for (input, expected) in cases {
      // parse as block
      let mut block_parser = test_parser!(input);
      let mut line = block_parser.read_line().unwrap().unwrap();
      line.discard_assert(TokenKind::OpenBracket);
      let attr_list = block_parser.parse_block_attr_list(&mut line).unwrap();
      expect_eq!(attr_list, expected, from: input);
    }
  }

  #[test]
  fn test_block_anchors() {
    let cases = vec![
      (
        "[[foo]]",
        Some(AnchorSrc {
          id: src!("foo", 2..5),
          reftext: None,
          loc: (0..7).into(),
        }),
      ),
      (
        "[[f.o]]",
        Some(AnchorSrc {
          id: src!("f.o", 2..5),
          reftext: None,
          loc: (0..7).into(),
        }),
      ),
      ("[[]]", None),
      ("[[ foo ]]", None),   // not valid per asciidoctor
      ("[[\"foo\"]]", None), // not valid per asciidoctor
      (
        "[[foo,bar]]",
        Some(AnchorSrc {
          id: src!("foo", 2..5),
          reftext: Some(just!("bar", 6..9)),
          loc: (0..11).into(),
        }),
      ),
      (
        "[[step-2,be _sure_]]",
        Some(AnchorSrc {
          id: src!("step-2", 2..8),
          reftext: Some(nodes![
            node!("be "; 9..12),
            node!(Inline::Italic(just!("sure", 13..17)), 12..18),
          ]),
          loc: (0..20).into(),
        }),
      ),
    ];
    for (input, expected) in cases {
      let mut parser = test_parser!(input);
      let mut line = parser.read_line().unwrap().unwrap();
      line.discard(2); // `[[`
      let anchor = parser.parse_block_anchor(&mut line).unwrap();
      expect_eq!(anchor, expected, from: input);
    }
  }

  #[test]
  fn test_inline_anchors() {
    let cases = vec![
      (
        "[[foo]]",
        Some(AnchorSrc {
          id: src!("foo", 2..5),
          reftext: None,
          loc: (0..7).into(),
        }),
        "",
      ),
      (
        "[[foo]]bar",
        Some(AnchorSrc {
          id: src!("foo", 2..5),
          reftext: None,
          loc: (0..7).into(),
        }),
        "bar",
      ),
      ("[[]]", None, "]]"),
      ("[[ foo ]]", None, " foo ]]"),
      ("[[\"foo\"]] bar", None, "\"foo\"]] bar"),
      (
        "[[foo,bar]] baz",
        Some(AnchorSrc {
          id: src!("foo", 2..5),
          reftext: Some(just!("bar", 6..9)),
          loc: (0..11).into(),
        }),
        " baz",
      ),
      (
        "[[step-2,be _sure_]] foo",
        Some(AnchorSrc {
          id: src!("step-2", 2..8),
          reftext: Some(nodes![
            node!("be "; 9..12),
            node!(Inline::Italic(just!("sure", 13..17)), 12..18),
          ]),
          loc: (0..20).into(),
        }),
        " foo",
      ),
    ];
    for (input, expected, line_after) in cases {
      let mut parser = test_parser!(input);
      let mut line = parser.read_line().unwrap().unwrap();
      line.discard(2); // `[[`
      let anchor = parser.parse_inline_anchor(&mut line).unwrap();
      expect_eq!(anchor, expected, from: input);
      expect_eq!(line.reassemble_src(), line_after, from: input);
    }
  }

  #[test]
  fn test_find_attr_delims() {
    let cases: Vec<(&str, &[usize])> = vec![
      ("[]", &[]),
      ("[#a,b=c]", &[2]),
      ("[foo,bar]", &[1]),
      ("[foo , ]", &[2]),
      ("[\"foo,bar\"]", &[]),
      ("[\"foo,bar]", &[2]),
      ("[foo,\"bar]", &[1]),
      ("['foo,bar']", &[]),
      ("['foo,bar]", &[2]),
      ("['foo',bar]", &[3]),
      ("['foo',\"bar,\"]", &[3]),
      ("[foo=bar]", &[]),
      ("[foo=bar,baz=qux]", &[3]),
      ("[foo=bar,baz='qux,']", &[3]),
      ("[foo=bar,baz='qux,,,,']", &[3]),
      ("[foo=bar,baz=qux,]", &[3, 3]),
      ("[foo=bar,baz=qux,,]", &[3, 3, 0]),
      ("[foo=bar,baz='qux,]", &[3, 4]),
      ("[foo=bar ,baz=qux]", &[4]),
      ("[foo=bar,baz=qux, ,]", &[3, 3, 1]),
      ("[,,,]", &[0, 0, 0]),
      ("[, , ,]", &[0, 1, 1]),
      ("[{example-caption},foo]", &[2]), // AttrRef(skip), Word, Comma, Word
      ("[{blank},foo]", &[1]),           // AttrRef(skip), Comma, Word
      ("[foo=bar,opts='opt1,opt2']", &[3]),
      ("[\"foo,bar\",baz]", &[5]),
      ("[foo%bar]", &[]),
      ("['foo%bar']", &[]),
    ];
    for (input, expected) in cases {
      let mut parser = test_parser!(input);
      let mut line = parser.read_line().unwrap().unwrap();
      line.discard_assert(TokenKind::OpenBracket);
      let delims = parser.find_delimiters(&line);
      expect_eq!(delims.as_slice(), expected, from: input);
    }
  }

  #[test]
  fn test_parse_attr_ir() {
    let cases = vec![
      ("''", "Positional([], w_symbol: false)"),
      ("\"\"", "Positional([], w_symbol: false)"),
      ("foo", "Positional([Word`foo`], w_symbol: false)"),
      ("\"foo\"", "Positional([Word`foo`], w_symbol: false)"),
      (
        "foo\"",
        "Positional([Word`foo`, DoubleQuote`\"`], w_symbol: false)",
      ),
      (
        "\"foo",
        "Positional([DoubleQuote`\"`, Word`foo`], w_symbol: false)",
      ),
      ("'", "Positional([SingleQuote`'`], w_symbol: false)"),
      ("\"", "Positional([DoubleQuote`\"`], w_symbol: false)"),
      (
        "=foo=",
        "Positional([EqualSigns`=`, Word`foo`, EqualSigns`=`], w_symbol: false)",
      ),
      (
        "foo bar",
        "Positional([Word`foo`, Whitespace` `, Word`bar`], w_symbol: false)",
      ),
      // shorthand
      ("#id", "Shorthand([Hash`#`, Word`id`])"),
      (
        "#id.role",
        "Shorthand([Hash`#`, Word`id`, Dots`.`, Word`role`])",
      ),
      (
        "#id.role_foo",
        "Shorthand([Hash`#`, Word`id`, Dots`.`, Word`role`, Underscore`_`, Word`foo`])",
      ),
      (
        "#id.role%opt",
        "Shorthand([Hash`#`, Word`id`, Dots`.`, Word`role`, Percent`%`, Word`opt`])",
      ),
      // named
      (
        "foo=\"bar=baz\"",
        "Named(foo, [Word`bar`, EqualSigns`=`, Word`baz`])",
      ),
      ("foo=bar", "Named(foo, [Word`bar`])"),
      ("foo='bar'", "Named(foo, [Word`bar`])"),
      ("foo='", "Named(foo, [SingleQuote`'`])"),
      ("foo=\"bar\"", "Named(foo, [Word`bar`])"),
      ("foo=\"bar", "Named(foo, [DoubleQuote`\"`, Word`bar`])"),
      ("foo = bar", "Named(foo, [Word`bar`])"),
      // options
      ("options='opt1'", "Options([Word`opt1`])"),
      ("options=opt1", "Options([Word`opt1`])"),
      ("options='opt1,opt2'", "Options([Word`opt1`], [Word`opt2`])"),
      (
        "options='opt1 bar,opt2'",
        "Options([Word`opt1`, Whitespace` `, Word`bar`], [Word`opt2`])",
      ),
      (
        "opts='opt1,,opt2 , opt3'",
        "Options([Word`opt1`], [Word`opt2`], [Word`opt3`])",
      ),
      // roles
      ("role='role1'", "Roles([Word`role1`])"),
      ("role=role1", "Roles([Word`role1`])"),
      ("role=role1 role2", "Roles([Word`role1`], [Word`role2`])"),
      // misc
      (
        "\"foo,bar\"",
        "Positional([Word`foo`, Comma`,`, Word`bar`], w_symbol: false)",
      ),
      (
        "\"foo%bar\"",
        "Positional([Word`foo`, Percent`%`, Word`bar`], w_symbol: false)",
      ),
      (
        "foo%bar",
        "Positional([Word`foo`, Percent`%`, Word`bar`], w_symbol: true)",
      ),
      (
        "[.role]#foo#",
        "Positional([OpenBracket`[`, Dots`.`, Word`role`, CloseBracket`]`, Hash`#`, Word`foo`, Hash`#`], w_symbol: true)",
      ),
    ];
    for (input, expected) in cases {
      let mut parser = test_parser!(input);
      let line = parser.read_line().unwrap().unwrap();
      let mut deq = Deq::new(parser.bump);
      deq.extend(line.into_iter());
      let parse_kind = parser.attr_ir(deq);
      expect_eq!(parse_kind.assert_string(), expected, from: input);
    }
  }

  #[test]
  fn test_parse_formatted_text_attr_list() {
    let cases = vec![(
      "[#tigers]#a text span#",
      AttrList {
        positional: vecb![],
        id: Some(src!("tigers", 2..8)),
        ..attr_list!(0..9)
      },
    )];
    for (input, expected) in cases {
      let mut parser = test_parser!(input);
      let mut line = parser.read_line().unwrap().unwrap();
      line.discard_assert(TokenKind::OpenBracket);
      let attr_list = parser.parse_formatted_text_attr_list(&mut line).unwrap();
      expect_eq!(attr_list, expected, from: input);
    }
  }

  // https://docs.asciidoctor.org/asciidoc/latest/macros/link-macro-attribute-parsing/
  #[test]
  fn test_parse_link_macro_attr_list() {
    let cases = vec![
      (
        "[foo, bar]",
        AttrList {
          positional: vecb![Some(just!("foo, bar", 1..9))],
          ..attr_list!(0..10)
        },
      ),
      (
        "[foo, bar, role=resource]",
        AttrList {
          positional: vecb![Some(just!("foo", 1..4)), Some(just!("bar", 6..9))],
          roles: vecb![src!("resource", 16..24)],
          ..attr_list!(0..25)
        },
      ),
      (
        "[Discuss AsciiDoc]",
        AttrList {
          positional: vecb![Some(just!("Discuss AsciiDoc", 1..17))],
          ..attr_list!(0..18)
        },
      ),
      (
        "[Discuss AsciiDoc,role=resource,window=_blank]",
        AttrList {
          positional: vecb![Some(just!("Discuss AsciiDoc", 1..17)), None],
          named: Named::from(vecb![(src!("window", 32..38), just!("_blank", 39..45))]),
          roles: vecb![src!("resource", 23..31)],
          ..attr_list!(0..46)
        },
      ),
      (
        "[\"Google, DuckDuckGo, Ecosia\",role=teal]",
        AttrList {
          positional: vecb![Some(just!("Google, DuckDuckGo, Ecosia", 2..28))],
          roles: vecb![src!("teal", 35..39)],
          ..attr_list!(0..40)
        },
      ),
      (
        "[\"1=2 posits the problem of inequality\"]",
        AttrList {
          positional: vecb![Some(just!("1=2 posits the problem of inequality", 2..38))],
          ..attr_list!(0..40)
        },
      ),
      (
        "[\"href=\"#top\" attribute\"]",
        AttrList {
          positional: vecb![Some(just!("href=\"#top\" attribute", 2..23))],
          ..attr_list!(0..25)
        },
      ),
    ];
    for (input, expected) in cases {
      let mut parser = test_parser!(input);
      let mut line = parser.read_line().unwrap().unwrap();
      line.discard_assert(TokenKind::OpenBracket);
      let attr_list = parser.parse_link_macro_attr_list(&mut line).unwrap();
      expect_eq!(attr_list, expected, from: input);
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
           --> test.adoc:1:5
            |
          1 | [#a,b=c]
            |     ^^^ Formatted text only supports attribute shorthand: id, roles, & options
        "},
      ),
      (
        "[#a,opts='opt1,opt2']",
        true,
        error! {"
           --> test.adoc:1:11
            |
          1 | [#a,opts='opt1,opt2']
            |           ^^^^^^^^^ Formatted text only supports attribute shorthand: id, roles, & options
        "},
      ),
      (
        "[#a,foo=bar,id=baz]",
        false,
        error! {"
           --> test.adoc:1:16
            |
          1 | [#a,foo=bar,id=baz]
            |                ^^^ More than one id attribute
        "},
      ),
    ];

    for (input, formatted, expected) in cases {
      let mut parser = test_parser!(input);
      let mut line = parser.read_line().unwrap().unwrap();
      line.discard(1); // `[`
      let diag = parser
        ._parse_attr_list(&mut line, true, formatted)
        .err()
        .unwrap();
      expect_eq!(diag.plain_text(), expected, from: input);
    }
  }

  impl AttrIr<'_> {
    fn assert_string(&self) -> String {
      match self {
        AttrIr::Id(tokens) => format!("Id({})", toks_to_string(tokens)),
        AttrIr::Positional(tokens, with_shorthand) => format!(
          "Positional({}, w_symbol: {})",
          toks_to_string(tokens),
          with_shorthand
        ),
        AttrIr::Named(name, ts) => format!("Named({}, {})", name.src, toks_to_string(ts)),
        AttrIr::Shorthand(tokens) => format!("Shorthand({})", toks_to_string(tokens)),
        AttrIr::Options(gs) => format!(
          "Options({})",
          gs.iter().map(toks_to_string).collect::<Vec<_>>().join(", ")
        ),
        AttrIr::Roles(gs) => format!(
          "Roles({})",
          gs.iter().map(toks_to_string).collect::<Vec<_>>().join(", ")
        ),
      }
    }
  }

  fn toks_to_string(tokens: &Deq<Token>) -> String {
    format!(
      "[{}]",
      tokens
        .iter()
        .map(|t| format!("{:?}`{}`", t.kind, t.lexeme))
        .collect::<Vec<_>>()
        .join(", ")
    )
  }
}
