use crate::internal::*;

pub fn encode_in<'a>(bytes: &[u8], bump: &'a Bump) -> BumpString<'a> {
  let mut out = BumpString::with_capacity_in(bytes.len().div_ceil(3) * 4, bump);
  let alphabet = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
  let digit = |index: u8| alphabet[index as usize] as char;
  for chunk in bytes.chunks(3) {
    out.push(digit(chunk[0] >> 2));
    match chunk.len() {
      3 => {
        let second = (chunk[0] << 4 | chunk[1] >> 4) & 0b111111;
        let third = (chunk[1] << 2 | chunk[2] >> 6) & 0b111111;
        let fourth = chunk[2] & 0b111111;
        out.extend([digit(second), digit(third), digit(fourth)]);
      }
      2 => {
        let second = (chunk[0] << 4 | chunk[1] >> 4) & 0b111111;
        let third = chunk[1] << 2 & 0b111111;
        out.extend([digit(second), digit(third), '=']);
      }
      _ => {
        let second = chunk[0] << 4 & 0b111111;
        out.extend([digit(second), '=', '=']);
      }
    }
  }
  out
}

#[test]
fn base64_encode() {
  let bump = Bump::new();
  assert_eq!(encode_in(b"", &bump).as_str(), "");
  assert_eq!(encode_in(b"f", &bump).as_str(), "Zg==");
  assert_eq!(encode_in(b"fo", &bump).as_str(), "Zm8=");
  assert_eq!(encode_in(b"foo", &bump).as_str(), "Zm9v");
  assert_eq!(encode_in(b"foob", &bump).as_str(), "Zm9vYg==");
  assert_eq!(encode_in(b"fooba", &bump).as_str(), "Zm9vYmE=");
  assert_eq!(encode_in(b"foobar", &bump).as_str(), "Zm9vYmFy");
  assert_eq!(
    encode_in(b"Hello, World!", &bump).as_str(),
    "SGVsbG8sIFdvcmxkIQ=="
  );
  let invalid_utf8: [u8; 4] = [0xFF, 0x00, 0xAB, 0xCD];
  assert_eq!(encode_in(&invalid_utf8, &bump).as_str(), "/wCrzQ==");
}
