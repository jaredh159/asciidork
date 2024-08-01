use std::fmt;

use crate::internal::*;

pub trait IncludeResolver {
  fn resolve(
    &mut self,
    path: &str,
    buffer: &mut dyn IncludeBuffer,
  ) -> std::result::Result<usize, ResolveError>;
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

#[derive(Debug, Clone)]
pub enum ResolveError {
  NotFound,
  Io(String),
  UriReadNotSupported,
}

impl fmt::Display for ResolveError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      ResolveError::NotFound => write!(f, "File not found"),
      ResolveError::Io(e) => write!(f, "I/O error: {}", e),
      ResolveError::UriReadNotSupported => write!(f, "URI read not supported"),
    }
  }
}

// test helpers

#[cfg(test)]
pub struct ErrorResolver(pub ResolveError);

#[cfg(test)]
impl IncludeResolver for ErrorResolver {
  fn resolve(
    &mut self,
    _: &str,
    _: &mut dyn IncludeBuffer,
  ) -> std::result::Result<usize, ResolveError> {
    Err(self.0.clone())
  }
}
