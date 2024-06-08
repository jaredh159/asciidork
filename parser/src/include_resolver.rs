use crate::internal::*;

pub trait IncludeBuffer {
  // fn set_capacity(&mut self, capacity: usize);
  fn as_bytes_mut(&mut self) -> &mut [u8];
  fn initialize(&mut self, len: usize);
}

#[derive(Debug)]
pub enum ResolveError {
  NotFound,
  Io(std::io::Error),
}

impl IncludeBuffer for Vec<u8> {
  // fn set_capacity(&mut self, capacity: usize) {
  //   let additional = capacity.saturating_sub(self.capacity());
  //   self.reserve(additional);
  // }

  fn initialize(&mut self, len: usize) {
    self.resize(len, 0);
  }

  fn as_bytes_mut(&mut self) -> &mut [u8] {
    self
  }
}

impl<'a> IncludeBuffer for BumpVec<'a, u8> {
  // fn set_capacity(&mut self, capacity: usize) {
  //   let additional = capacity.saturating_sub(self.capacity());
  //   self.reserve(additional);
  // }

  fn initialize(&mut self, len: usize) {
    self.resize(len, 0);
  }

  fn as_bytes_mut(&mut self) -> &mut [u8] {
    self
  }
}

pub trait IncludeResolver {
  fn resolve(
    &mut self,
    path: &str,
    buffer: &mut dyn IncludeBuffer,
  ) -> std::result::Result<usize, ResolveError>;
}

pub struct LolResolver;

struct XockResolver(pub Vec<u8>);
impl IncludeResolver for XockResolver {
  fn resolve(
    &mut self,
    _path: &str,
    buffer: &mut dyn IncludeBuffer,
  ) -> std::result::Result<usize, ResolveError> {
    buffer.initialize(self.0.len());
    let bytes = buffer.as_bytes_mut();
    bytes.copy_from_slice(&self.0);
    Ok(self.0.len())
  }
}

impl IncludeResolver for LolResolver {
  // fn resolve(
  //   &mut self,
  //   _path: &str,
  //   buffer: &mut dyn IncludeBuffer,
  // ) -> std::result::Result<usize, ResolveError> {
  //   buffer.initialize(3);
  //   let bytes = buffer.as_bytes_mut();
  //   bytes[0] = b'L';
  //   bytes[1] = b'o';
  //   bytes[2] = b'l';
  //   // bytes.copy_from_slice(b"Hello, lol");
  //   // bytes.copy_within(src, dest)
  //   Ok(3)
  // }

  fn resolve(
    &mut self,
    _path: &str,
    buffer: &mut dyn IncludeBuffer,
  ) -> std::result::Result<usize, ResolveError> {
    use std::io::Read;
    let mut file =
      std::fs::File::open("/Users/jared/lol.adoc").map_err(|_| ResolveError::NotFound)?;

    // if let Ok(meta) = file.metadata() {
    //   buffer.set_capacity(meta.len() as usize);
    // } else {
    //   buffer.set_capacity(1024);
    // }
    let metadata = file.metadata().map_err(ResolveError::Io)?;
    buffer.initialize(metadata.len() as usize);

    let mut bytes_written = 0;
    let bytes = buffer.as_bytes_mut();

    loop {
      let n = file.read(bytes).map_err(ResolveError::Io)?;
      if n == 0 {
        break;
      }
      bytes_written += n;
    }

    Ok(bytes_written)
  }
}

struct FileResolver;

impl IncludeResolver for FileResolver {
  fn resolve(
    &mut self,
    path: &str,
    buffer: &mut dyn IncludeBuffer,
  ) -> std::result::Result<usize, ResolveError> {
    use std::io::Read;
    let mut file = std::fs::File::open(path).map_err(|_| ResolveError::NotFound)?;

    // if let Ok(meta) = file.metadata() {
    //   buffer.set_capacity(meta.len() as usize);
    // } else {
    //   buffer.set_capacity(1024);
    // }
    let metadata = file.metadata().map_err(ResolveError::Io)?;
    buffer.initialize(metadata.len() as usize);

    let mut bytes_written = 0;
    let bytes = buffer.as_bytes_mut();

    loop {
      let n = file.read(bytes).map_err(ResolveError::Io)?;
      if n == 0 {
        break;
      }
      bytes_written += n;
    }

    Ok(bytes_written)
  }
}

fn bump_foo(mut resolver: impl IncludeResolver) {
  let bump = Bump::new();
  let mut bump_vec = BumpVec::new_in(&bump);
  let n = resolver.resolve("test.adoc", &mut bump_vec).unwrap();
  println!("{}", n);
}

fn foo(mut resolver: impl IncludeResolver) {
  let mut vec = Vec::new();
  // vec.tru
  let n = resolver.resolve("test.adoc", &mut vec).unwrap();
  println!("{}", n);
}
