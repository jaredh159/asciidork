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

  // <duplication factor><duplication operator><span factor><span operator><horizontal alignment operator><vertical alignment operator><style operator>|<cell’s content>
  // >s
  // 3*2.2+>.^s| (kitchen sink)
  // (3*)(2.2+)(>)(.^)(s)| (kitchen sink)
  // 2.2+>.^s|
  // 2+>s|
  // Digits | Dots | GreaterThan | LessThan | Caret | Star
  fn parse_cell_spec(&self, line: &mut Line, sep: u8) -> Option<CellSpec> {
    let Some(first_token) = line.current_token() else {
      return None;
    };

    // no explicit cell spec, but we're sitting on the sep, so infer default
    if first_token.lexeme.as_bytes() == [sep] {
      line.discard(1);
      return Some(CellSpec::default());
    }

    // todo: handle custom sep that got caught in word token

    // try to pitch cases where we don't need to parse the cell spec
    // should probably benchmark to see if this is worth it
    match &first_token {
      Token {
        kind: Digits | Dots | GreaterThan | LessThan | Caret | Star,
        ..
      } => {}
      Token { kind: Word, lexeme, .. }
        if matches!(
          lexeme.as_bytes().first(),
          Some(b'a' | b'd' | b'e' | b'h' | b'l' | b'm' | b's')
        ) => {}
      _ => return None,
    }

    // speculatively parse a cell spec
    let mut spec = CellSpec::default();
    let mut cursor = 0;
    parse_duplication_factor(line, &mut spec, &mut cursor);
    parse_span_factor(line, &mut spec, &mut cursor);
    parse_h_align(line, &mut spec, &mut cursor);
    parse_v_align(line, &mut spec, &mut cursor);
    parse_style(line, &mut spec, &mut cursor);

    // w/ a valid spec, cursor will be > 0 and line will be on sep
    if cursor == 0 || line.nth_token(cursor).map(|t| t.lexeme.as_bytes()) != Some(&[sep]) {
      None
    } else {
      line.discard(cursor + 1);
      Some(spec)
    }
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

fn parse_style(line: &Line, spec: &mut CellSpec, cursor: &mut usize) {
  let Some(token) = line.nth_token(*cursor) else {
    return;
  };
  if !token.is(Word) {
    return;
  }
  match token.lexeme.as_bytes() {
    [b'a'] => spec.style = CellContentStyle::AsciiDoc,
    [b'd'] => spec.style = CellContentStyle::Default,
    [b'e'] => spec.style = CellContentStyle::Emphasis,
    [b'h'] => spec.style = CellContentStyle::Header,
    [b'l'] => spec.style = CellContentStyle::Literal,
    [b'm'] => spec.style = CellContentStyle::Monospace,
    [b's'] => spec.style = CellContentStyle::Strong,
    _ => return,
  }
  *cursor += 1;
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
