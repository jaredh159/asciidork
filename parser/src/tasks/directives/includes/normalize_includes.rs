use std::ops::Range;

use bumpalo::collections::CollectIn;

use crate::internal::*;

impl<'arena> Parser<'arena> {
  pub(super) fn normalize_include_bytes(
    &mut self,
    path: &Path,
    include_attrs: &AttrList,
    bytes: &mut BumpVec<'arena, u8>,
  ) -> std::result::Result<(), &'static str> {
    self.normalize_encoding(include_attrs.named("encoding"), bytes)?;
    self.normalize_asciidoc(path, bytes);
    Ok(())
  }

  pub(super) fn select_lines(
    &mut self,
    include_attrs: &AttrList,
    src_path: &Path,
    bytes: &mut BumpVec<'arena, u8>,
  ) -> Result<()> {
    // NB: selecting lines takes precedence over tags
    if let Some(lines) = include_attrs.named("lines") {
      self.select_line_ranges(lines, bytes);
      Ok(())
    } else {
      self.select_tagged_lines(include_attrs, src_path, bytes)
    }
  }

  fn normalize_asciidoc(&mut self, path: &Path, bytes: &mut BumpVec<'arena, u8>) {
    let adoc_exts = [".adoc", ".asciidoc", ".ad", ".asc", ".txt"];
    if !adoc_exts.contains(&path.extension()) {
      return;
    }
    let mut dest = BumpVec::with_capacity_in(bytes.len(), self.bump);
    for (i, b) in bytes.iter().enumerate() {
      if *b == b'\n' {
        trim_trailing_whitespace(&mut dest);
        dest.push(b'\n');
      } else if *b == b'\r' && bytes.get(i + 1) == Some(&b'\n') {
        continue;
      } else {
        dest.push(*b);
      }
    }
    std::mem::swap(bytes, &mut dest);
  }

  fn select_line_ranges(&mut self, lines: &str, bytes: &mut BumpVec<'arena, u8>) {
    let ranges = parse_line_ranges(lines);
    if ranges.is_empty() {
      return;
    }

    let mut selected = BumpVec::with_capacity_in(bytes.len(), self.bump);
    for (i, line) in bytes.split(|&b| b == b'\n').enumerate() {
      if ranges.iter().any(|range| range.contains(&(i + 1))) {
        selected.extend(line);
        selected.push(b'\n');
      } else {
        selected.extend_from_slice(b"asciidorkinclude::[false]\n");
      }
    }
    std::mem::swap(bytes, &mut selected);
  }

  fn normalize_encoding(
    &mut self,
    encoding: Option<&str>,
    bytes: &mut BumpVec<u8>,
  ) -> std::result::Result<(), &'static str> {
    if let Some("utf-16" | "utf16" | "UTF-16" | "UTF16") = encoding {
      return self.convert_utf16_le(bytes);
    }

    // UTF-8 BOM
    if bytes.len() >= 3 && bytes[0..3] == [0xEF, 0xBB, 0xBF] {
      bytes.drain(0..3);
      return Ok(());
    }

    // UTF-16 BOM, little endian
    if bytes.len() >= 2 && bytes[0..2] == [0xFF, 0xFE] {
      bytes.drain(0..2);
      return self.convert_utf16_le(bytes);
    }

    // UTF-16 BOM, big endian
    if bytes.len() >= 2 && bytes[0..2] == [0xFE, 0xFF] {
      bytes.drain(0..2);
      if !bytes.len().is_multiple_of(2) {
        bytes.push(0x00);
      }

      let utf16 = bytes
        .chunks_exact(2)
        .map(|chunk| u16::from_be_bytes([chunk[0], chunk[1]]))
        .collect_in::<BumpVec<_>>(self.bump);

      if from_utf16_in(utf16, bytes, self.bump) {
        return Ok(());
      } else {
        return Err("Invalid UTF-16 (BE)");
      }
    }

    // TODO: doctor supports iso-8859-1, + all encodings ruby supports
    // we could use encoding_rs for this, but might not be any demand
    if std::str::from_utf8(bytes).is_err() {
      return Err("Invalid UTF-8");
    }

    Ok(())
  }

  fn convert_utf16_le(&self, bytes: &mut BumpVec<u8>) -> std::result::Result<(), &'static str> {
    if !bytes.len().is_multiple_of(2) {
      bytes.push(0x00);
    }
    let utf16: BumpVec<u16> = bytes
      .chunks_exact(2)
      .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
      .collect_in(self.bump);

    if from_utf16_in(utf16, bytes, self.bump) {
      Ok(())
    } else {
      Err("Invalid UTF-16 (LE)")
    }
  }
}

fn trim_trailing_whitespace(bytes: &mut BumpVec<u8>) {
  let mut i = bytes.len();
  while i > 0 && (bytes[i - 1] == b' ' || bytes[i - 1] == b'\t') {
    i -= 1;
  }
  bytes.truncate(i);
}

fn from_utf16_in(utf16: BumpVec<u16>, dest: &mut BumpVec<u8>, bump: &Bump) -> bool {
  match BumpString::from_utf16_in(&utf16, bump) {
    Ok(string) => {
      dest.clear();
      dest.extend_from_slice(string.as_bytes());
      true
    }
    Err(_) => false,
  }
}

fn parse_line_ranges(s: &str) -> Vec<Range<usize>> {
  let mut ranges = Vec::new();
  s.split([',', ';']).for_each(|part| {
    let part = part.trim();
    if let Some((low, high)) = part.split_once("..") {
      let Ok(low_n) = low.parse::<usize>() else {
        return;
      };
      if high == "-1" || high.is_empty() {
        ranges.push(low_n..usize::MAX);
      } else if let Ok(high_n) = high.parse::<usize>()
        && low_n <= high_n
      {
        ranges.push(low_n..high_n + 1);
      }
    } else if let Ok(n) = part.parse::<usize>() {
      ranges.push(n..n + 1);
    }
  });
  ranges
}

#[cfg(test)]
mod tests {
  use super::*;
  use test_utils::*;

  fn p(s: &str) -> Path {
    Path::new(s)
  }

  #[test]
  fn strips_utf8_bom() {
    let mut parser = test_parser!("");
    let mut bytes = vecb![0xEF, 0xBB, 0xBF, 0x68, 0x69];
    parser.normalize_encoding(None, &mut bytes).unwrap();
    assert_eq!(bytes.as_slice(), b"hi");
  }

  #[test]
  fn converts_little_endian_utf16_to_utf8() {
    let mut parser = test_parser!("");
    let mut bytes = vecb![0xFF, 0xFE, 0x68, 0x00, 0x69, 0x00];
    parser.normalize_encoding(None, &mut bytes).unwrap();
    assert_eq!(bytes.as_slice(), b"hi");
  }

  #[test]
  fn converts_big_endian_utf16_to_utf8() {
    let mut parser = test_parser!("");
    let mut bytes = vecb![0xFE, 0xFF, 0x00, 0x68, 0x00, 0x69];
    parser.normalize_encoding(None, &mut bytes).unwrap();
    assert_eq!(bytes.as_slice(), b"hi");
  }

  #[test]
  fn test_parse_line_ranges() {
    assert_eq!(parse_line_ranges("1"), vec![1..2]);
    assert_eq!(parse_line_ranges("1;2"), vec![1..2, 2..3]);
    assert_eq!(parse_line_ranges("1 ; 2"), vec![1..2, 2..3]);
    assert_eq!(parse_line_ranges(" 1 ,  2  "), vec![1..2, 2..3]);
    assert_eq!(parse_line_ranges("1..3"), vec![1..4]);
    assert_eq!(parse_line_ranges("17..-1"), vec![17..usize::MAX]);
    assert_eq!(parse_line_ranges("17..-1,howdy"), vec![17..usize::MAX]);
    assert_eq!(parse_line_ranges("17.."), vec![17..usize::MAX]);
    assert_eq!(parse_line_ranges("1..3;5..7"), vec![1..4, 5..8]);
    assert_eq!(parse_line_ranges("1..3,5..7"), vec![1..4, 5..8]);
    assert_eq!(
      parse_line_ranges("1;3..4;6..-1"),
      vec![1..2, 3..5, 6..usize::MAX]
    );
    assert!(parse_line_ranges("17..15").is_empty()); // invalid range
    assert!(parse_line_ranges("hello").is_empty());
  }
}
