use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;

use asciidork_parser::includes::*;

use IncludeTarget as Target;
use ResolveError::*;

#[derive(Debug, Clone)]
pub struct CliResolver {
  base_dir: Option<PathBuf>,
  strict: bool,
}

impl IncludeResolver for CliResolver {
  fn resolve(
    &mut self,
    target: IncludeTarget,
    buffer: &mut dyn IncludeBuffer,
  ) -> std::result::Result<usize, ResolveError> {
    match target {
      Target::FilePath(target) => self.resolve_filepath(target, buffer),
      Target::Uri(uri) => match minreq::get(uri).send() {
        Ok(response) => {
          let adoc = response.as_bytes();
          buffer.initialize(adoc.len());
          let bytes = buffer.as_bytes_mut();
          bytes.copy_from_slice(adoc);
          Ok(adoc.len())
        }
        Err(err) => Err(ResolveError::UriRead(err.to_string())),
      },
    }
  }

  fn get_base_dir(&self) -> Option<String> {
    self
      .base_dir
      .clone()
      .map(|pathbuf| pathbuf.to_string_lossy().into())
  }

  fn clone_box(&self) -> Box<dyn IncludeResolver> {
    Box::new(self.clone())
  }
}

impl CliResolver {
  pub const fn new(base_dir: Option<PathBuf>, strict: bool) -> Self {
    Self { base_dir, strict }
  }

  fn resolve_filepath(
    &self,
    path: String,
    buffer: &mut dyn IncludeBuffer,
  ) -> std::result::Result<usize, ResolveError> {
    let pathb = PathBuf::from(path);
    let Ok(pathc) = dunce::canonicalize(&pathb) else {
      return Err(NotFound);
    };

    if self.strict && pathb != pathc {
      match (pathb.file_name(), pathc.file_name()) {
        // only send back the filename so we don't accidentally expose
        // full system filepaths in error messages for security reasons
        (Some(pfn), Some(cfn)) if pfn != cfn => {
          return Err(CaseMismatch(Some(cfn.to_string_lossy().to_string())));
        }
        _ => return Err(CaseMismatch(None)),
      }
    } else if !pathb.exists() {
      return Err(NotFound);
    }

    let file = File::open(pathb)?;
    let len = file.metadata().map(|m| m.len() as usize)?;
    buffer.initialize(len);
    let bytes = buffer.as_bytes_mut();
    Read::read_exact(&mut BufReader::new(file), bytes)?;
    Ok(len)
  }
}
