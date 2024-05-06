use lazy_static::lazy_static;
use regex::Regex;

use super::TableTokens;
use crate::internal::*;
use TokenKind::*;

#[derive(Debug)]
struct CellStart {
  spec: CellSpec,
  drop_tokens: usize,
  drop_bytes: usize,
  resuming: usize,
}

lazy_static! {
  // multiplier(1), horiz(2), vert(3), width(4), style(5)
  pub static ref COLSPEC_RE: Regex =
    Regex::new(r"^\s*(?:(\d+)\*)?([<^>])?(?:\.([<^>]))?((?:\d+%?)|~)?(a|d|e|h|l|m|s)?\s*$").unwrap();
}

fn parse_col_spec(col_attr: &str, specs: &mut BumpVec<ColSpec>) {
  if col_attr.is_empty() {
    specs.push(ColSpec::default());
    return;
  }

  let Some(captures) = COLSPEC_RE.captures(col_attr) else {
    specs.push(ColSpec::default());
    return;
  };

  let mut spec = ColSpec::default();

  if let Some(h_align) = captures.get(2) {
    spec.h_align = h_align.as_str().parse().unwrap_or(Default::default());
  }

  if let Some(v_align) = captures.get(3) {
    spec.v_align = v_align.as_str().parse().unwrap_or(Default::default());
  }

  if let Some(width) = captures.get(4).map(|m| m.as_str()) {
    if width == "~" {
      spec.width = ColWidth::Auto;
    } else if let Some(digits) = width.strip_suffix('%') {
      spec.width = ColWidth::Percentage(digits.parse().unwrap_or(1));
    } else {
      spec.width = ColWidth::Proportional(width.parse().unwrap_or(1));
    }
  }

  if let Some(style) = captures.get(5) {
    spec.style = style.as_str().parse().unwrap_or(Default::default());
  }

  if let Some(repeat_match) = captures.get(1) {
    let repeat = repeat_match.as_str().parse().unwrap_or(1);
    if repeat > 1 {
      for _ in 1..repeat {
        specs.push(spec.clone());
      }
    }
  }
  specs.push(spec);
}

impl<'bmp, 'src> Parser<'bmp, 'src> {
  pub(super) fn parse_col_specs(&mut self, cols_attr: &str) -> BumpVec<'bmp, ColSpec> {
    let mut specs = bvec![in self.bump];
    if cols_attr.trim().is_empty() {
      return specs;
    }
    cols_attr
      .split(',')
      .for_each(|col| parse_col_spec(col, &mut specs));
    specs
  }

  pub(super) fn starts_cell(&self, tokens: &mut TableTokens, sep: u8) -> bool {
    self.peek_cell_start(tokens, sep).is_some()
  }

  pub(super) fn consume_cell_start(
    &self,
    tokens: &mut TableTokens,
    sep: u8,
  ) -> Option<(CellSpec, usize)> {
    let data = self.peek_cell_start(tokens, sep)?;
    tokens.discard(data.drop_tokens);
    if data.drop_bytes > 0 {
      let token = tokens.current_mut().unwrap();
      token.lexeme = &token.lexeme[data.drop_bytes..];
      token.loc.start += data.drop_bytes;
    }
    Some((data.spec, data.resuming))
  }

  fn peek_cell_start(&self, tokens: &mut TableTokens, sep: u8) -> Option<CellStart> {
    let Some(first_token) = tokens.current_mut() else {
      return None;
    };
    let Some(first_byte) = first_token.lexeme.as_bytes().first() else {
      return None;
    };

    // no explicit cell spec, but we're sitting on the sep, so infer default
    if first_byte == &sep {
      return Some(CellStart {
        spec: CellSpec::default(),
        drop_tokens: if first_token.len() == 1 { 1 } else { 0 },
        drop_bytes: if first_token.len() == 1 { 0 } else { 1 },
        resuming: first_token.loc.start + 1,
      });
    }

    // optimization: words are most common, so reject non-candidates
    if first_token.is(Word) {
      match first_byte {
        b'a' | b'd' | b'e' | b'h' | b'l' | b'm' | b's' => {}
        _ => return None,
      }
      // otherwise, it would need to be one of these to start a spec
    } else if !matches!(
      first_token.kind,
      Digits | Dots | LessThan | GreaterThan | Caret
    ) {
      return None;
    }

    // speculatively parse a cell spec
    let mut spec = CellSpec::default();
    let mut cursor = 0;
    parse_duplication_factor(tokens, &mut spec, &mut cursor);
    parse_span_factor(tokens, &mut spec, &mut cursor);
    parse_h_align(tokens, &mut spec, &mut cursor);
    parse_v_align(tokens, &mut spec, &mut cursor);

    // style can be found within a word only if they used a custom separator
    // that wasn't its own token or a word boundary, e.g. `x` in `3*2.4+>.^sx`
    let style_within_word = parse_style(tokens, &mut spec, &mut cursor);

    if cursor == 0 {
      return None;
    }
    let Some(cursor_token) = tokens.nth(cursor) else {
      return None;
    };
    let cursor_token_end = cursor_token.loc.end;
    match (style_within_word, cursor_token.lexeme.as_bytes()) {
      (false, bytes) if bytes == [sep] => Some(CellStart {
        spec,
        drop_tokens: cursor + 1,
        drop_bytes: 0,
        resuming: cursor_token_end,
      }),
      (true, bytes) if bytes.len() == 2 && bytes.get(1) == Some(&sep) => Some(CellStart {
        spec,
        drop_tokens: cursor + 1,
        drop_bytes: 0,
        resuming: cursor_token_end,
      }),
      (true, bytes) if bytes.len() > 2 && bytes.get(1) == Some(&sep) => {
        let joined = tokens.nth(cursor).unwrap();
        Some(CellStart {
          spec,
          drop_tokens: cursor,
          drop_bytes: 2,
          resuming: joined.loc.start + 2,
        })
      }
      _ => None,
    }
  }
}

fn parse_style(tokens: &TableTokens, spec: &mut CellSpec, cursor: &mut usize) -> bool {
  let Some(token) = tokens.nth(*cursor) else {
    return false;
  };
  if !token.is(Word) {
    return false;
  }
  let Some(style) = &token.lexeme[0..1].parse().ok() else {
    return false;
  };
  spec.style = Some(*style);
  if token.len() == 1 {
    *cursor += 1;
    false
  } else {
    true
  }
}

fn parse_duplication_factor(tokens: &TableTokens, spec: &mut CellSpec, cursor: &mut usize) {
  if tokens.has_seq_at(&[Digits, Star], *cursor) {
    if let Some(Ok(digits)) = tokens.nth(*cursor).map(|t| t.lexeme.parse::<u8>()) {
      spec.duplication = Some(digits);
      *cursor += 2;
    }
  }
}

fn parse_span_factor(tokens: &TableTokens, spec: &mut CellSpec, cursor: &mut usize) {
  if tokens.has_seq_at(&[Digits, Plus], *cursor) {
    if let Some(Ok(digits)) = tokens.nth(*cursor).map(|t| t.lexeme.parse::<u8>()) {
      spec.col_span = Some(digits);
      *cursor += 2;
    }
  } else if tokens.has_seq_at(&[Dots, Digits, Plus], *cursor) {
    if !tokens.nth(*cursor).is_len(Dots, 1) {
      return;
    }
    if let Some(Ok(digits)) = tokens.nth(*cursor + 1).map(|t| t.lexeme.parse::<u8>()) {
      spec.row_span = Some(digits);
      *cursor += 3;
    }
  } else if tokens.has_seq_at(&[Digits, Dots, Digits, Plus], *cursor) {
    if !tokens.nth(*cursor + 1).is_len(Dots, 1) {
      return;
    }
    let col = tokens.nth(*cursor).map(|t| t.lexeme.parse::<u8>());
    let row = tokens.nth(*cursor + 2).map(|t| t.lexeme.parse::<u8>());
    if let (Some(Ok(col)), Some(Ok(row))) = (col, row) {
      spec.col_span = Some(col);
      spec.row_span = Some(row);
      *cursor += 4;
    }
  }
}

fn parse_h_align(tokens: &TableTokens, spec: &mut CellSpec, cursor: &mut usize) {
  match tokens.nth(*cursor).map(|t| t.kind) {
    Some(LessThan) => spec.h_align = Some(HorizontalAlignment::Left),
    Some(GreaterThan) => spec.h_align = Some(HorizontalAlignment::Right),
    Some(Caret) => spec.h_align = Some(HorizontalAlignment::Center),
    _ => return,
  }
  *cursor += 1;
}

fn parse_v_align(tokens: &TableTokens, spec: &mut CellSpec, cursor: &mut usize) {
  if !tokens.nth(*cursor).is_len(Dots, 1) {
    return;
  }
  match tokens.nth(*cursor + 1).map(|t| t.kind) {
    Some(GreaterThan) => spec.v_align = Some(VerticalAlignment::Bottom),
    Some(LessThan) => spec.v_align = Some(VerticalAlignment::Top),
    Some(Caret) => spec.v_align = Some(VerticalAlignment::Middle),
    _ => return,
  }
  *cursor += 2;
}

#[cfg(test)]
mod tests {
  use super::*;
  use test_utils::{assert_eq, *};

  #[test]
  fn test_parse_cell_specs() {
    let cases = [
      (b'|', "|foo", "foo", Some((CellSpec::default(), 1))),
      (b'|', "foo", "foo", None),
      (
        b'|',
        "m|foo",
        "foo",
        Some((
          CellSpec {
            style: Some(CellContentStyle::Monospace),
            ..CellSpec::default()
          },
          2,
        )),
      ),
      (
        b'|',
        "3*2.4+>.^s|foo",
        "foo",
        Some((
          CellSpec {
            duplication: Some(3),
            col_span: Some(2),
            row_span: Some(4),
            h_align: Some(HorizontalAlignment::Right),
            v_align: Some(VerticalAlignment::Middle),
            style: Some(CellContentStyle::Strong),
          },
          11,
        )),
      ),
      (b'x', "xfoo", "foo", Some((CellSpec::default(), 1))),
      (
        b'x',
        "3*2.4+>.^sxfoo",
        "foo",
        Some((
          CellSpec {
            duplication: Some(3),
            col_span: Some(2),
            row_span: Some(4),
            h_align: Some(HorizontalAlignment::Right),
            v_align: Some(VerticalAlignment::Middle),
            style: Some(CellContentStyle::Strong),
          },
          11,
        )),
      ),
    ];

    let parser = Parser::new(leaked_bump(), "");
    for (sep, input, remaining, expected) in &cases {
      let mut lexer = Lexer::new(input);
      let line = lexer.consume_line(leaked_bump()).unwrap();
      let mut tokens = vecb![];
      line.drain_into(&mut tokens);
      let mut tokens = TableTokens::new(tokens, lexer.loc_src(0..input.len()));
      let start = parser.consume_cell_start(&mut tokens, *sep);
      assert_eq!(start, *expected, from: input);
      if let Some((_, loc)) = *expected {
        assert_eq!(input.as_bytes().get(loc), remaining.as_bytes().first());
      }
    }
  }

  #[test]
  fn test_parse_col_specs() {
    use ColWidth::*;
    let cases: &[(&str, &[ColSpec])] = &[
      (
        "3*",
        &[ColSpec::default(), ColSpec::default(), ColSpec::default()],
      ),
      ("1", &[ColSpec::default()]),
      ("~", &[ColSpec { width: Auto, ..ColSpec::default() }]),
      (
        ">",
        &[ColSpec {
          h_align: HorizontalAlignment::Right,
          ..ColSpec::default()
        }],
      ),
      (
        ".^",
        &[ColSpec {
          v_align: VerticalAlignment::Middle,
          ..ColSpec::default()
        }],
      ),
      (
        ".>",
        &[ColSpec {
          v_align: VerticalAlignment::Bottom,
          ..ColSpec::default()
        }],
      ),
      (
        "l",
        &[ColSpec {
          style: CellContentStyle::Literal,
          ..ColSpec::default()
        }],
      ),
      (
        "1,2",
        &[
          ColSpec::default(),
          ColSpec {
            width: Proportional(2),
            ..ColSpec::default()
          },
        ],
      ),
      (
        " 1,2  , 1 ", // ignore spaces
        &[
          ColSpec::default(),
          ColSpec {
            width: Proportional(2),
            ..ColSpec::default()
          },
          ColSpec::default(),
        ],
      ),
      // ignore empty colspecs
      ("", &[]),
      (" ", &[]),
      (
        "2*>.>3e,,15%",
        &[
          ColSpec {
            width: Proportional(3),
            h_align: HorizontalAlignment::Right,
            v_align: VerticalAlignment::Bottom,
            style: CellContentStyle::Emphasis,
          },
          ColSpec {
            width: Proportional(3),
            h_align: HorizontalAlignment::Right,
            v_align: VerticalAlignment::Bottom,
            style: CellContentStyle::Emphasis,
          },
          ColSpec::default(),
          ColSpec {
            width: Percentage(15),
            ..ColSpec::default()
          },
        ],
      ),
    ];
    let mut parser = Parser::new(leaked_bump(), "");
    for (input, expected) in cases {
      let cols = parser.parse_col_specs(input);
      assert_eq!(cols, *expected);
    }
  }
}
