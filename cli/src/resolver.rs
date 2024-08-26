use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;

use asciidork_parser::include_resolver::*;
use asciidork_parser::prelude::*;

use IncludeTarget as Target;
use ResolveError::*;
use SourceFile as Src;

pub struct CliResolver {
  base_dir: Option<PathBuf>,
}

impl IncludeResolver for CliResolver {
  fn resolve(
    &mut self,
    target: IncludeTarget,
    source: &SourceFile,
    buffer: &mut dyn IncludeBuffer,
  ) -> std::result::Result<usize, ResolveError> {
    match (source, target, self.base_dir.as_ref()) {
      // not sure if this is correct, possibly should use cwd...
      (Src::Stdin { .. }, Target::FilePath(_), None) => Err(BaseDirRequired),
      (Src::Stdin { .. }, Target::FilePath(target), Some(base_dir)) => {
        let pathbuf = base_dir.join(target);
        self.resolve_path(pathbuf, buffer)
      }
      (Src::Path(src), Target::FilePath(target), _base_dir) => {
        let abspath = if target.is_relative() {
          src.clone().parent().join(target)
        } else {
          target
        };
        self.resolve_path(abspath, buffer)
      }
      _ => Err(NotFound),
    }
  }
}

impl CliResolver {
  pub const fn new(base_dir: Option<PathBuf>) -> Self {
    Self { base_dir }
  }

  fn resolve_path(
    &self,
    path: impl Into<Path>,
    buffer: &mut dyn IncludeBuffer,
  ) -> std::result::Result<usize, ResolveError> {
    let path: Path = path.into();
    if path.exists_fs() {
      let file = File::open(path)?;
      let len = file.metadata().map(|m| m.len() as usize)?;
      buffer.initialize(len);
      let bytes = buffer.as_bytes_mut();
      Read::read_exact(&mut BufReader::new(file), bytes)?;
      Ok(len)
    } else {
      Err(NotFound)
    }
  }
}
