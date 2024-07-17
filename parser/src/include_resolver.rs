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
    self.reserve(len + 1);
    self.resize(len, 0);
  }

  fn as_bytes_mut(&mut self) -> &mut [u8] {
    self
  }
}

impl<'a> IncludeBuffer for BumpVec<'a, u8> {
  fn initialize(&mut self, len: usize) {
    self.reserve(len + 1);
    self.resize(len, 0);
  }

  fn as_bytes_mut(&mut self) -> &mut [u8] {
    self
  }
}

#[derive(Debug)]
pub enum ResolveError {
  NotFound,
  Io(std::io::Error),
}
