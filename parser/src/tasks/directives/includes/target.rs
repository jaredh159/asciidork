use crate::internal::*;
use IncludeTarget as Target;
use ResolveError as Err;
use SourceFile as Src;

pub fn prepare(
  target_str: &str,
  target_is_uri: bool,
  src_file: &SourceFile,
  src_is_primary: bool,
  base_dir: Option<Path>,
  _safe_mode: SafeMode,
) -> std::result::Result<IncludeTarget, ResolveError> {
  let target = Path::new(target_str);
  if target_is_uri {
    // TODO: handle URI
    return Ok(Target::Uri(target_str.to_string()));
  }
  if src_is_primary && target.is_relative() {
    let Some(base_dir) = base_dir else {
      return Err(Err::BaseDirRequired);
    };
    let abspath = base_dir.join(target);
    return Ok(Target::FilePath(abspath.to_string()));
  }
  match (src_file, base_dir) {
    (Src::Path(src), _) => {
      let abspath = if target.is_relative() {
        let dir = Path::new(src.dirname());
        dir.join(target)
      } else {
        target
      };
      Ok(Target::FilePath(abspath.to_string()))
    }
    _ => todo!(),
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use IncludeTarget::*;

  struct TestCase {
    name: &'static str,
    target_str: &'static str,
    target_is_uri: bool,
    src_file: SourceFile,
    src_is_primary: bool,
    safe_mode: SafeMode,
    base_dir: Option<Path>,
    expected: std::result::Result<IncludeTarget, ResolveError>,
  }

  impl Default for TestCase {
    fn default() -> Self {
      Self {
        name: "",
        target_str: "",
        target_is_uri: false,
        src_file: SourceFile::Tmp,
        src_is_primary: true,
        safe_mode: SafeMode::Unsafe,
        base_dir: Some(Path::new("/basedir")),
        expected: Err(ResolveError::NotFound),
      }
    }
  }
  #[test]
  fn resolve_target_filepaths() {
    let cases = vec![
      TestCase {
        name: "relative includes from primary doc are resolved relative to basedir",
        src_is_primary: true, // <-- primary doc
        base_dir: Some(Path::new("/basedir")),
        target_str: "b.adoc",
        src_file: SourceFile::Path(Path::new("/basedir/subdir/src.adoc")),
        expected: Ok(FilePath("/basedir/b.adoc".to_string())), // <-- basedir used
        ..TestCase::default()
      },
      TestCase {
        name: "relative include from primary doc targeting basedir",
        target_str: "b.adoc",
        src_file: SourceFile::Path(Path::new("/basedir/src.adoc")),
        expected: Ok(FilePath("/basedir/b.adoc".to_string())),
        ..TestCase::default()
      },
      TestCase {
        name: "non-primary relative include",
        src_is_primary: false, // <-- not primary, so basedir not consulted
        target_str: "other.adoc",
        src_file: SourceFile::Path(Path::new("/basedir/sub/src.adoc")),
        expected: Ok(FilePath("/basedir/sub/other.adoc".to_string())),
        ..TestCase::default()
      },
      TestCase {
        name: "basic absolute include",
        target_str: "/abs/other.adoc",
        src_file: SourceFile::Path(Path::new("/path/to/src.adoc")),
        expected: Ok(FilePath("/abs/other.adoc".to_string())),
        ..TestCase::default()
      },
      TestCase {
        name: "relative canonicalized",
        src_is_primary: false,
        target_str: "../other.adoc",
        src_file: SourceFile::Path(Path::new("/d1/d2/src.adoc")),
        expected: Ok(FilePath("/d1/d2/../other.adoc".to_string())),
        ..TestCase::default()
      },
    ];

    for case in cases {
      let actual = prepare(
        case.target_str,
        case.target_is_uri,
        &case.src_file,
        case.src_is_primary,
        case.base_dir,
        case.safe_mode,
      );
      assert_eq!(actual, case.expected, "TestCase.name: {:?}", case.name);
    }
  }
}
