use std::fs::File;
use std::io::{self, Read};

enum Source {
  File(io::BufReader<File>),
  Bytes { bytes: Vec<u8>, location: usize },
}

pub struct Reader(Source);

impl Reader {
  pub fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
    match self.0 {
      Source::File(ref mut file) => file.read(buf),
      Source::Bytes {
        ref bytes,
        ref mut location,
      } => {
        let bytes = &bytes[*location..];
        let len = bytes.len().min(buf.len());
        buf[..len].copy_from_slice(&bytes[..len]);
        *location += len;
        Ok(len)
      }
    }
  }

  pub fn capacity_hint(&self) -> Option<usize> {
    match self.0 {
      Source::File(ref file) => file.get_ref().metadata().ok().map(|m| m.len() as usize),
      Source::Bytes { ref bytes, .. } => Some(bytes.len()),
    }
  }
}

impl From<File> for Reader {
  fn from(file: File) -> Self {
    Reader(Source::File(io::BufReader::new(file)))
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
