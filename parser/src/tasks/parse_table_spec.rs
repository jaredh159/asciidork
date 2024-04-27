use bumpalo::collections::CollectIn;
// use regex::Regex;

use crate::internal::*;
use TokenKind::*;

impl<'bmp, 'src> Parser<'bmp, 'src> {
  pub(super) fn parse_col_specs(&mut self, cols_attr: &str) -> BumpVec<'bmp, ColSpec> {
    cols_attr
      .split(',')
      .map(|col| self.parse_col_spec(col))
      .collect_in(self.bump)
  }

  fn parse_col_spec(&self, col_attr: &str) -> ColSpec {
    ColSpec { width: col_attr.parse().unwrap_or(1) }
  }

  fn parse_cell_spec(&self, line: &mut Line, sep: u8) -> Option<CellSpec> {
    let Some(first_token) = line.current_token_mut() else {
      return None;
    };

    // no explicit cell spec, but we're sitting on the sep, so infer default
    if first_token.lexeme.as_bytes().first() == Some(&sep) {
      if first_token.len() == 1 {
        line.discard(1);
      } else {
        first_token.lexeme = &first_token.lexeme[1..];
        first_token.loc = first_token.loc.incr_start();
        line.src = &line.src[1..];
      }
      return Some(CellSpec::default());
    }

    // speculatively parse a cell spec
    let mut spec = CellSpec::default();
    let mut cursor = 0;
    parse_duplication_factor(line, &mut spec, &mut cursor);
    parse_span_factor(line, &mut spec, &mut cursor);
    parse_h_align(line, &mut spec, &mut cursor);
    parse_v_align(line, &mut spec, &mut cursor);

    // style could be found within a word only if they used a custom separator
    // that wasn't its own token or a word boundary, e.g. `x` in `3*2.4+>.^sx`
    let style_within_word = parse_style(line, &mut spec, &mut cursor);

    if cursor == 0 {
      return None;
    }
    let Some(cursor_token) = line.nth_token(cursor) else {
      return None;
    };
    match (style_within_word, cursor_token.lexeme.as_bytes()) {
      (false, bytes) if bytes == [sep] => {
        line.discard(cursor + 1);
        Some(spec)
      }
      (true, bytes) if bytes.len() == 2 && bytes.get(1) == Some(&sep) => {
        line.discard(cursor + 1);
        Some(spec)
      }
      (true, bytes) if bytes.len() > 2 && bytes.get(1) == Some(&sep) => {
        line.discard(cursor);
        let joined = line.current_token_mut().unwrap();
        joined.lexeme = &joined.lexeme[2..];
        joined.loc.start += 2;
        line.src = &line.src[2..];
        Some(spec)
      }
      _ => None,
    }
  }
}

fn parse_style(line: &Line, spec: &mut CellSpec, cursor: &mut usize) -> bool {
  let Some(token) = line.nth_token(*cursor) else {
    return false;
  };
  if !token.is(Word) {
    return false;
  }
  match token.lexeme.as_bytes().first() {
    Some(b'a') => spec.style = CellContentStyle::AsciiDoc,
    Some(b'd') => spec.style = CellContentStyle::Default,
    Some(b'e') => spec.style = CellContentStyle::Emphasis,
    Some(b'h') => spec.style = CellContentStyle::Header,
    Some(b'l') => spec.style = CellContentStyle::Literal,
    Some(b'm') => spec.style = CellContentStyle::Monospace,
    Some(b's') => spec.style = CellContentStyle::Strong,
    _ => return false,
  }
  if token.len() == 1 {
    *cursor += 1;
    false
  } else {
    true
  }
}

fn parse_duplication_factor(line: &Line, spec: &mut CellSpec, cursor: &mut usize) {
  if line.has_seq_at(&[Digits, Star], *cursor) {
    if let Some(Ok(digits)) = line.nth_token(*cursor).map(|t| t.lexeme.parse::<u8>()) {
      spec.duplication = digits;
      *cursor += 2;
    }
  }
}

fn parse_span_factor(line: &Line, spec: &mut CellSpec, cursor: &mut usize) {
  if line.has_seq_at(&[Digits, Plus], *cursor) {
    if let Some(Ok(digits)) = line.nth_token(*cursor).map(|t| t.lexeme.parse::<u8>()) {
      spec.col_span = digits;
      *cursor += 2;
    }
  } else if line.has_seq_at(&[Dots, Digits, Plus], *cursor) {
    if !line.nth_token(*cursor).is_len(Dots, 1) {
      return;
    }
    if let Some(Ok(digits)) = line.nth_token(*cursor + 1).map(|t| t.lexeme.parse::<u8>()) {
      spec.row_span = digits;
      *cursor += 3;
    }
  } else if line.has_seq_at(&[Digits, Dots, Digits, Plus], *cursor) {
    if !line.nth_token(*cursor + 1).is_len(Dots, 1) {
      return;
    }
    let col = line.nth_token(*cursor).map(|t| t.lexeme.parse::<u8>());
    let row = line.nth_token(*cursor + 2).map(|t| t.lexeme.parse::<u8>());
    if let (Some(Ok(col)), Some(Ok(row))) = (col, row) {
      spec.col_span = col;
      spec.row_span = row;
      *cursor += 4;
    }
  }
}

fn parse_h_align(line: &Line, spec: &mut CellSpec, cursor: &mut usize) {
  match line.nth_token(*cursor).map(|t| t.kind) {
    Some(LessThan) => spec.h_align = HorizontalAlignment::Left,
    Some(GreaterThan) => spec.h_align = HorizontalAlignment::Right,
    Some(Caret) => spec.h_align = HorizontalAlignment::Center,
    _ => return,
  }
  *cursor += 1;
}

fn parse_v_align(line: &Line, spec: &mut CellSpec, cursor: &mut usize) {
  if !line.nth_token(*cursor).is_len(Dots, 1) {
    return;
  }
  match line.nth_token(*cursor + 1).map(|t| t.kind) {
    Some(GreaterThan) => spec.v_align = VerticalAlignment::Top,
    Some(LessThan) => spec.v_align = VerticalAlignment::Bottom,
    Some(Caret) => spec.v_align = VerticalAlignment::Middle,
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
      (b'|', "|foo", "foo", Some(CellSpec::default())),
      (b'|', "foo", "foo", None),
      (
        b'|',
        "3*2.4+>.^s|foo",
        "foo",
        Some(CellSpec {
          duplication: 3,
          col_span: 2,
          row_span: 4,
          h_align: HorizontalAlignment::Right,
          v_align: VerticalAlignment::Middle,
          style: CellContentStyle::Strong,
        }),
      ),
      (b'x', "xfoo", "foo", Some(CellSpec::default())),
      (
        b'x',
        "3*2.4+>.^sxfoo",
        "foo",
        Some(CellSpec {
          duplication: 3,
          col_span: 2,
          row_span: 4,
          h_align: HorizontalAlignment::Right,
          v_align: VerticalAlignment::Middle,
          style: CellContentStyle::Strong,
        }),
      ),
    ];

    let parser = Parser::new(leaked_bump(), "");
    for (sep, input, remaining, expected) in &cases {
      let mut lexer = Lexer::new(input);
      let mut line = lexer.consume_line(leaked_bump()).unwrap();
      let cell_spec = parser.parse_cell_spec(&mut line, *sep);
      assert_eq!(cell_spec, *expected);
      assert_eq!(line.src, *remaining);
    }
  }

  #[test]
  fn test_parse_col_specs() {
    let cases: &[(&str, &[ColSpec])] = &[
      ("1", &[ColSpec { width: 1 }]),
      ("1,2", &[ColSpec { width: 1 }, ColSpec { width: 2 }]),
    ];
    let mut parser = Parser::new(leaked_bump(), "");
    for (input, expected) in cases {
      let cols = parser.parse_col_specs(input);
      assert_eq!(cols, *expected);
    }
  }
}
