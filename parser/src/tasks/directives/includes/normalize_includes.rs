use bumpalo::collections::CollectIn;

use crate::internal::*;

impl<'arena> Parser<'arena> {
  pub(super) fn normalize_include_bytes(
    &mut self,
    path: &Path,
    bytes: &mut BumpVec<'arena, u8>,
  ) -> std::result::Result<(), &'static str> {
    self.normalize_byte_order(bytes)?;
    self.normalize_asciidoc(path, bytes);
    Ok(())
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

  fn normalize_byte_order(
    &mut self,
    bytes: &mut BumpVec<u8>,
  ) -> std::result::Result<(), &'static str> {
    // UTF-8 BOM
    if bytes.len() >= 3 && bytes[0..3] == [0xEF, 0xBB, 0xBF] {
      bytes.drain(0..3);
      return Ok(());
    }

    // UTF-16 BOM, little endian
    if bytes.len() >= 2 && bytes[0..2] == [0xFF, 0xFE] {
      bytes.drain(0..2);

      // SAFETY: because we ensure the len is even, it's fine to transmute to u16
      // because we're going to check that it's valid going back to utf8 anyway
      let utf16: BumpVec<u16> = unsafe {
        if bytes.len() % 2 != 0 {
          bytes.push(0x00);
        }
        let ptr = bytes.as_ptr() as *const u16;
        let len = bytes.len() / 2;
        BumpVec::from_raw_parts_in(ptr as *mut u16, len, len, self.bump)
      };

      if from_utf16_in(utf16, bytes, self.bump) {
        return Ok(());
      } else {
        return Err("Invalid UTF-16 (LE)");
      }
    }

    // UTF-16 BOM, big endian
    if bytes.len() >= 2 && bytes[0..2] == [0xFE, 0xFF] {
      bytes.drain(0..2);
      if bytes.len() % 2 != 0 {
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

    Ok(())
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
    parser.normalize_byte_order(&mut bytes).unwrap();
    assert_eq!(bytes.as_slice(), b"hi");
  }

  #[test]
  fn converts_little_endian_utf16_to_utf8() {
    let mut parser = test_parser!("");
    let mut bytes = vecb![0xFF, 0xFE, 0x68, 0x00, 0x69, 0x00];
    parser.normalize_byte_order(&mut bytes).unwrap();
    assert_eq!(bytes.as_slice(), b"hi");
  }

  #[test]
  fn converts_big_endian_utf16_to_utf8() {
    let mut parser = test_parser!("");
    let mut bytes = vecb![0xFE, 0xFF, 0x00, 0x68, 0x00, 0x69];
    parser.normalize_byte_order(&mut bytes).unwrap();
    assert_eq!(bytes.as_slice(), b"hi");
  }
}
