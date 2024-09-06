use std::fmt;

use crate::internal::*;

#[derive(Debug, Eq, PartialEq)]
pub enum IncludeTarget {
  FilePath(String),
  Uri(String),
}

impl IncludeTarget {
  pub const fn is_path(&self) -> bool {
    matches!(self, IncludeTarget::FilePath(_))
  }

  pub const fn is_uri(&self) -> bool {
    matches!(self, IncludeTarget::Uri(_))
  }
}

pub trait IncludeResolver {
  fn resolve(
    &mut self,
    target: IncludeTarget,
    buffer: &mut dyn IncludeBuffer,
  ) -> std::result::Result<usize, ResolveError>;

  fn get_base_dir(&self) -> Option<String> {
    None
  }
}

pub trait IncludeBuffer {
  fn as_bytes_mut(&mut self) -> &mut [u8];
  fn initialize(&mut self, len: usize);
}

impl IncludeBuffer for Vec<u8> {
  fn initialize(&mut self, len: usize) {
    self.reserve(len + 1); // for possible extra newline
    self.resize(len, 0);
  }

  fn as_bytes_mut(&mut self) -> &mut [u8] {
    self
  }
}

impl<'a> IncludeBuffer for BumpVec<'a, u8> {
  fn initialize(&mut self, len: usize) {
    self.reserve(len + 1); // for possible extra newline
    self.resize(len, 0);
  }

  fn as_bytes_mut(&mut self) -> &mut [u8] {
    self
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolveError {
  NotFound,
  Io(String),
  UriReadNotSupported,
  BaseDirRequired,
}

impl fmt::Display for ResolveError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      ResolveError::NotFound => write!(f, "File not found"),
      ResolveError::Io(e) => write!(f, "I/O error: {}", e),
      ResolveError::UriReadNotSupported => write!(f, "URI read not supported"),
      ResolveError::BaseDirRequired => {
        write!(
          f,
          "Include resolvers must supply a base_dir for relative includes from primary document"
        )
      }
    }
  }
}

impl From<std::io::Error> for ResolveError {
  fn from(e: std::io::Error) -> Self {
    ResolveError::Io(e.to_string())
  }
}

// test helpers

#[cfg(test)]
pub struct ConstResolver(pub Vec<u8>);

#[cfg(test)]
impl IncludeResolver for ConstResolver {
  fn resolve(
    &mut self,
    _: IncludeTarget,
    buffer: &mut dyn IncludeBuffer,
  ) -> std::result::Result<usize, ResolveError> {
    buffer.initialize(self.0.len());
    let bytes = buffer.as_bytes_mut();
    bytes.copy_from_slice(&self.0);
    Ok(self.0.len())
  }

  fn get_base_dir(&self) -> Option<String> {
    Some("/".to_string())
  }
}

#[cfg(test)]
pub struct ErrorResolver(pub ResolveError);

#[cfg(test)]
impl IncludeResolver for ErrorResolver {
  fn resolve(
    &mut self,
    _: IncludeTarget,
    _: &mut dyn IncludeBuffer,
  ) -> std::result::Result<usize, ResolveError> {
    Err(self.0.clone())
  }
}