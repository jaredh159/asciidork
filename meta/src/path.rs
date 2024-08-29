use std::convert::AsRef;
use std::fmt;
use std::path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Path(String);

impl Path {
  pub fn new(path: impl Into<String>) -> Path {
    Self(path.into())
  }

  pub fn parent(self) -> Path {
    if let Some(p) = self.0.rsplit_once('/').map(|(before, _)| before) {
      return Path(p.into());
    }
    self
  }

  pub fn join(self, other: Path) -> Path {
    Path(format!("{}/{}", self.0.trim_end_matches('/'), other.0))
  }

  pub fn to_str(&self) -> &str {
    &self.0
  }

  pub fn into_string(self) -> String {
    self.0
  }

  pub fn basename(&self) -> &str {
    self
      .0
      .rsplit_once('/')
      .map(|(_, after)| after)
      .unwrap_or(&self.0)
  }

  pub fn basename_no_ext(&self) -> &str {
    self
      .basename()
      .rsplit_once('.')
      .map(|(before, _)| before)
      .unwrap_or(self.basename())
  }

  pub fn is_absolute(&self) -> bool {
    self.0.starts_with('/') // TODO: Windows
  }

  pub fn is_relative(&self) -> bool {
    !self.is_absolute()
  }

  pub fn ext(&self) -> &str {
    if let Some(idx) = self.0.rfind('.') {
      let ext = &self.0[idx..];
      if !ext.contains('/') {
        return ext;
      }
    }
    ""
  }

  pub fn path_buf(&self) -> path::PathBuf {
    path::PathBuf::from(&self.0)
  }

  pub fn exists_fs(&self) -> bool {
    self.path_buf().exists()
  }
}

impl AsRef<std::path::Path> for Path {
  fn as_ref(&self) -> &std::path::Path {
    self.0.as_ref()
  }
}

impl AsRef<str> for Path {
  fn as_ref(&self) -> &str {
    &self.0
  }
}

impl From<std::path::PathBuf> for Path {
  fn from(path: std::path::PathBuf) -> Self {
    Path(path.to_string_lossy().into())
  }
}

impl fmt::Display for Path {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_join() {
    let path = Path::new("foo/bar");
    assert_eq!(path.join(Path::new("baz")).to_str(), "foo/bar/baz");
    let path = Path::new("foo/bar/");
    assert_eq!(path.join(Path::new("baz")).to_str(), "foo/bar/baz");
    let path = Path::new("foo/bar");
    assert_eq!(path.join(Path::new("baz/")).to_str(), "foo/bar/baz/");
  }

  #[test]
  fn test_path_ext() {
    let path = Path::new("foo/bar/baz.txt");
    assert_eq!(path.ext(), ".txt");
    let path = Path::new("foo/bar/baz.asciidoc");
    assert_eq!(path.ext(), ".asciidoc");
    let path = Path::new("baz.txt");
    assert_eq!(path.ext(), ".txt");
    let path = Path::new("foo/bar/baz");
    assert_eq!(path.ext(), "");
    let path = Path::new("foo");
    assert_eq!(path.ext(), "");
    let path = Path::new("foo/b.ar/baz");
    assert_eq!(path.ext(), "");
  }

  #[test]
  fn test_path_parent() {
    let path = Path::new("foo/bar/baz");
    assert_eq!(path.parent().to_str(), "foo/bar");
    let path = Path::new("foo");
    assert_eq!(path.parent().to_str(), "foo");
  }

  #[test]
  fn test_path_basename() {
    let path = Path::new("foo/bar/baz.txt");
    assert_eq!(path.basename(), "baz.txt");
    let path = Path::new("baz.txt");
    assert_eq!(path.basename(), "baz.txt");
    let path = Path::new("foo/bar/baz");
    assert_eq!(path.basename(), "baz");
    let path = Path::new("foo");
    assert_eq!(path.basename(), "foo");
  }
}
