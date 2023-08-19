use std::fs::File;
use std::io::{self, Read};

enum Source {
  File {
    file: io::BufReader<File>,
    path: Option<String>,
  },
  Bytes {
    bytes: Vec<u8>,
    location: usize,
  },
  Collection(Vec<Source>),
}

pub struct Reader(Source);

impl Source {
  pub fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
    match self {
      Source::File { ref mut file, .. } => file.read(buf),
      Source::Bytes { ref bytes, ref mut location } => {
        let bytes = &bytes[*location..];
        let len = bytes.len().min(buf.len());
        buf[..len].copy_from_slice(&bytes[..len]);
        *location += len;
        Ok(len)
      }
      Source::Collection(ref mut sources) => {
        let Some(source) = sources.last_mut() else {
          return Ok(0);
        };
        match source.read(buf)? {
          0 => {
            sources.pop();
            self.read(buf)
          }
          n => Ok(n),
        }
      }
    }
  }

  pub fn capacity_hint(&self) -> Option<usize> {
    match &self {
      Source::File { ref file, .. } => file.get_ref().metadata().ok().map(|m| m.len() as usize),
      Source::Bytes { ref bytes, .. } => Some(bytes.len()),
      Source::Collection(sources) => sources.iter().map(|s| s.capacity_hint()).sum(),
    }
  }
}

impl Reader {
  pub fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
    self.0.read(buf)
  }

  pub fn capacity_hint(&self) -> Option<usize> {
    self.0.capacity_hint()
  }

  pub fn from_file(file: File, path: Option<impl Into<String>>) -> Self {
    Reader(Source::File {
      file: io::BufReader::new(file),
      path: path.map(Into::into),
    })
  }
}

impl From<String> for Reader {
  fn from(string: String) -> Self {
    Reader(Source::Bytes {
      bytes: string.into_bytes(),
      location: 0,
    })
  }
}

impl From<&'static str> for Reader {
  fn from(string: &'static str) -> Self {
    Reader(Source::Bytes {
      bytes: string.as_bytes().to_vec(),
      location: 0,
    })
  }
}
