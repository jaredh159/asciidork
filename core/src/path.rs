#[derive(Debug, Clone, PartialEq, Eq)]
enum Component {
  UriScheme(String),
  DrivePrefix(String),
  Root,
  CurrentDir,
  ParentDir,
  Normal(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Path {
  separator: char,
  components: Vec<Component>,
}

impl Path {
  pub fn new_specifying_separator(path: impl Into<String>, separator: char) -> Path {
    Path::from_optional_sep(path, Some(separator))
  }

  pub fn new(path: impl Into<String>) -> Path {
    Path::from_optional_sep(path, None)
  }

  fn from_optional_sep(path: impl Into<String>, separator: Option<char>) -> Path {
    let path: String = path.into();
    let mut path = path.as_str();
    let mut components = Vec::with_capacity(4);
    let inferred_separator = match drive_prefix(path) {
      Some(prefix) => {
        components.push(Component::DrivePrefix(prefix));
        path = &path[2..];
        '\\'
      }
      _ => {
        if path.contains('\\') {
          '\\'
        } else if path.contains('/') {
          '/'
        } else {
          std::path::MAIN_SEPARATOR
        }
      }
    };
    if path.starts_with(inferred_separator) {
      components.push(Component::Root);
      path = &path[1..];
    }
    path.split(inferred_separator).for_each(|s| match s {
      "" => {}
      "." => components.push(Component::CurrentDir),
      ".." => components.push(Component::ParentDir),
      "https:" => components.push(Component::UriScheme("https".to_string())),
      "http:" => components.push(Component::UriScheme("http".to_string())),
      _ => components.push(Component::Normal(s.to_string())),
    });
    Path {
      separator: separator.unwrap_or(inferred_separator),
      components,
    }
  }

  pub fn push(&mut self, other: impl Into<Path>) {
    let other: Path = other.into();
    if other.is_absolute() {
      self.components.clear();
    }
    self.components.extend(other.components);
  }

  pub fn is_absolute(&self) -> bool {
    if self.is_uri() && matches!(self.components.first(), Some(Component::UriScheme(_))) {
      return true;
    }
    self.components.first() == Some(&Component::Root)
      || (matches!(self.components.first(), Some(Component::DrivePrefix(_)))
        && matches!(self.components.get(1), Some(Component::Root)))
  }

  pub fn is_relative(&self) -> bool {
    !self.is_absolute()
  }

  pub fn pop(&mut self) -> bool {
    if self.components.len() > 1 {
      self.components.pop();
      true
    } else {
      false
    }
  }

  pub fn join(&self, other: impl Into<Path>) -> Path {
    let mut joined = self.clone();
    joined.push(other);
    joined
  }

  pub fn file_name(&self) -> &str {
    self._file_name(self.components.len() - 1)
  }

  fn _file_name(&self, idx: usize) -> &str {
    match self.components.get(idx) {
      Some(Component::Normal(s)) => s,
      Some(Component::CurrentDir) => self._file_name(idx - 1),
      _ => "",
    }
  }

  pub fn file_stem(&self) -> &str {
    let file_name = self.file_name();
    file_name
      .rsplit_once('.')
      .map(|(before, _)| before)
      .unwrap_or(file_name)
  }

  pub fn extension(&self) -> &str {
    let filename = self.file_name();
    if let Some(idx) = filename.rfind('.') {
      &filename[idx..]
    } else {
      ""
    }
  }

  pub fn dirname(&self) -> String {
    if self.components.len() == 1 && self.components[0] == Component::Root {
      return self.to_string();
    }
    if self.components.len() == 2
      && matches!(self.components[0], Component::DrivePrefix(_))
      && self.components[1] == Component::Root
    {
      return self.to_string();
    }
    let mut path = String::with_capacity(32);
    for (i, component) in self.components.iter().enumerate() {
      if i == self.components.len() - 1 {
        break;
      }
      match component {
        Component::UriScheme(s) => {
          path.push_str(s);
          path.push(':');
        }
        Component::DrivePrefix(s) => path.push_str(s),
        Component::Root => path.push(self.separator),
        Component::CurrentDir => path.push('.'),
        Component::ParentDir => path.push_str(".."),
        Component::Normal(s) => path.push_str(s),
      }
      if i < self.components.len() - 2
        && component != &Component::Root
        && !matches!(component, Component::DrivePrefix(_))
      {
        path.push(self.separator);
      }
    }
    path
  }

  pub fn is_uri(&self) -> bool {
    matches!(self.components.first(), Some(Component::UriScheme(_)))
  }
}

impl From<std::path::PathBuf> for Path {
  fn from(path: std::path::PathBuf) -> Self {
    Path::new(path.to_string_lossy())
  }
}

impl From<&str> for Path {
  fn from(path: &str) -> Self {
    Path::new(path)
  }
}

impl std::fmt::Display for Path {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let mut path = String::with_capacity(32);
    for (i, component) in self.components.iter().enumerate() {
      match component {
        Component::UriScheme(s) => {
          path.push_str(s);
          path.push_str(":/");
        }
        Component::DrivePrefix(s) => path.push_str(s),
        Component::Root => path.push(self.separator),
        Component::CurrentDir => path.push('.'),
        Component::ParentDir => path.push_str(".."),
        Component::Normal(s) => path.push_str(s),
      }
      if i < self.components.len() - 1
        && component != &Component::Root
        && !matches!(component, Component::DrivePrefix(_))
      {
        path.push(self.separator);
      }
    }
    write!(f, "{path}")
  }
}

fn drive_prefix(path: &str) -> Option<String> {
  if path.len() < 2 {
    return None;
  }
  let bytes = path.as_bytes();
  if bytes[1] == b':' && bytes[0].is_ascii_alphabetic() {
    let prefix = path[..2].to_string();
    Some(prefix)
  } else {
    None
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn path(s: &str) -> Path {
    Path::new(s)
  }

  #[test]
  fn path_new() {
    let path = Path::new("/usr/local");
    assert_eq!(path.separator, '/');
    assert_eq!(path.components.len(), 3);
    assert_eq!(path.components[0], Component::Root);
    assert_eq!(path.components[1], Component::Normal("usr".to_string()));
    assert_eq!(path.components[2], Component::Normal("local".to_string()));
    assert_eq!(path.to_string(), "/usr/local");
    let path = Path::new("/usr/local/");
    assert_eq!(path.to_string(), "/usr/local");
  }

  #[test]
  fn path_new_windows() {
    let mut path = Path::new(r#"c:\windows\foo.dll"#);
    assert_eq!(path.separator, '\\');
    assert_eq!(path.components.len(), 4);
    assert_eq!(path.components[0], Component::DrivePrefix("c:".to_string()));
    assert_eq!(path.components[1], Component::Root);
    assert_eq!(path.components[2], Component::Normal("windows".to_string()));
    assert_eq!(path.components[3], Component::Normal("foo.dll".to_string()));
    assert_eq!(path.to_string(), r#"c:\windows\foo.dll"#);
    path.pop();
    path.push("baz");
    path.push("qux.dll");
    assert_eq!(path.to_string(), r#"c:\windows\baz\qux.dll"#);
    assert!(!path.is_uri());
  }

  #[test]
  fn path_new_uri() {
    let path = Path::new(r#"https://example.com/path"#);
    assert_eq!(path.separator, '/');
    assert_eq!(path.components.len(), 3);
    assert_eq!(
      path.components[0],
      Component::UriScheme("https".to_string())
    );
    assert_eq!(
      path.components[1],
      Component::Normal("example.com".to_string())
    );
    assert_eq!(path.components[2], Component::Normal("path".to_string()));
    assert_eq!(path.to_string(), "https://example.com/path");
    assert!(path.is_uri());
  }

  #[test]
  fn path_is_absolute() {
    assert!(path("/usr/local").is_absolute());
    assert!(!path("usr/local").is_absolute());
    assert!(path(r#"c:\foo"#).is_absolute());
    assert!(path(r#"\foo"#).is_absolute());
    assert!(path(r#"http://foo.com"#).is_absolute());
    assert!(!path(r#"c:foo"#).is_absolute());
  }

  #[test]
  fn path_push_pop() {
    let mut path = Path::new("/usr/local");
    path.push("bin");
    assert_eq!(path.components.len(), 4);
    assert_eq!(path.components[3], Component::Normal("bin".to_string()));
    assert_eq!(path.to_string(), "/usr/local/bin");
    assert!(path.pop());
    assert_eq!(path.to_string(), "/usr/local");
    assert!(path.pop());
    assert_eq!(path.to_string(), "/usr");
    assert!(path.pop());
    assert_eq!(path.to_string(), "/");
    assert!(!path.pop());
    assert_eq!(path.to_string(), "/");
    // pushing an absolute path replaces
    let mut path = Path::new("/usr/local");
    path.push("/bin");
    assert_eq!(path.to_string(), "/bin");
  }

  #[test]
  fn path_join() {
    let path = Path::new("/etc");
    assert_eq!(path.to_string(), "/etc");
    let joined = path.join("passwd");
    assert_eq!(path.to_string(), "/etc");
    assert_eq!(joined.to_string(), "/etc/passwd");
  }

  #[test]
  fn path_dirname() {
    assert_eq!("", &path("foo.txt").dirname());
    assert_eq!("bar", &path("bar/foo.txt").dirname());
    assert_eq!("/", &path("/foo.txt").dirname());
    assert_eq!("/", &path("/").dirname());
    assert_eq!("c:\\foo", path("c:\\foo\\baz.adoc").dirname());
    assert_eq!("c:\\", path("c:\\").dirname());
  }

  #[test]
  fn path_extension() {
    assert_eq!(".txt", path("foo/bar/baz.txt").extension());
    assert_eq!(".asciidoc", path("foo/bar/baz.asciidoc").extension());
    assert_eq!(".txt", path("baz.txt").extension());
    assert_eq!("", path("foo/bar/baz").extension());
    assert_eq!("", path("foo").extension());
    assert_eq!("", path("foo/b.ar/baz").extension());
  }

  #[test]
  fn path_file_name() {
    assert_eq!("bin", path("bin").file_name());
    assert_eq!("bin", path("bin/").file_name());
    assert_eq!("foo.txt", path("tmp/foo.txt").file_name());
    assert_eq!("foo.txt", path("foo.txt/.").file_name());
    assert_eq!("foo.txt", path("foo.txt/./././././.").file_name());
    assert_eq!("foo.txt", path("foo.txt/.//").file_name());
    assert_eq!("", path("foo.txt/..").file_name());
    assert_eq!("", path("/").file_name());
    assert_eq!("", path("c:\\").file_name());
    assert_eq!("foo", path("c:\\foo").file_name());
    assert_eq!("foo", path("\\foo").file_name());
  }

  #[test]
  fn path_file_stem() {
    assert_eq!("bin", path("bin").file_stem());
    assert_eq!("bin", path("bin/").file_stem());
    assert_eq!("foo", path("foo.rs").file_stem());
    assert_eq!("foo", path("/weird.txt/foo.bar/foo.rs").file_stem());
    assert_eq!("foo.tar", path("foo.tar.gz").file_stem());
  }

  #[test]
  fn join_uri_relative() {
    let src = Path::new("https://example.com/foo/bar");
    let dir = Path::new(src.dirname());
    assert_eq!(dir.to_string(), "https://example.com/foo");
    let rel = Path::new("baz");
    let abs = dir.join(rel);
    assert_eq!(abs.to_string(), "https://example.com/foo/baz");
    assert!(abs.is_uri());
  }
}
