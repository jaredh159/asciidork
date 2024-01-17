use std::path::Path;

pub fn file_ext(path: &str) -> Option<&str> {
  Path::new(path).extension().and_then(|s| s.to_str())
}
