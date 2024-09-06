use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;

use asciidork_parser::includes::*;

use IncludeTarget as Target;
use ResolveError::*;

pub struct CliResolver {
  base_dir: Option<PathBuf>,
}

impl IncludeResolver for CliResolver {
  fn resolve(
    &mut self,
    target: IncludeTarget,
    buffer: &mut dyn IncludeBuffer,
  ) -> std::result::Result<usize, ResolveError> {
    match target {
      Target::FilePath(target) => self.resolve_filepath(target, buffer),
      _ => Err(NotFound),
    }
  }

  fn get_base_dir(&self) -> Option<String> {
    self
      .base_dir
      .clone()
      .map(|pathbuf| pathbuf.to_string_lossy().into())
  }
}

impl CliResolver {
  pub const fn new(base_dir: Option<PathBuf>) -> Self {
    Self { base_dir }
  }

  fn resolve_filepath(
    &self,
    path: String,
    buffer: &mut dyn IncludeBuffer,
  ) -> std::result::Result<usize, ResolveError> {
    let path = PathBuf::from(path);
    if path.exists() {
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
