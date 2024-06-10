use crate::internal::*;

pub trait DefaultIn {
  fn default_in(bmp: &Bump) -> Self;
}

// NB: if we need push_front, switch to a ring buffer
#[derive(Debug)]
pub struct Deq<'bmp, T> {
  bmp: &'bmp Bump,
  buf: BumpVec<'bmp, T>,
  pos: usize,
}

impl<'bmp, T> Deq<'bmp, T> {
  pub fn new(bmp: &'bmp Bump) -> Self {
    Deq {
      bmp,
      buf: BumpVec::new_in(bmp),
      pos: 0,
    }
  }

  pub fn with_capacity(bmp: &'bmp Bump, capacity: usize) -> Self {
    Deq {
      bmp,
      buf: BumpVec::with_capacity_in(capacity, bmp),
      pos: 0,
    }
  }

  pub fn push(&mut self, item: T) {
    self.buf.push(item);
  }

  pub fn pop(&mut self) -> Option<T> {
    if self.is_empty() {
      return None;
    }
    self.buf.pop()
  }

  pub fn is_empty(&self) -> bool {
    self.buf.len() - self.pos == 0
  }

  pub fn len(&self) -> usize {
    self.buf.len() - self.pos
  }

  pub fn iter(&self) -> impl ExactSizeIterator<Item = &T> {
    self.buf.iter().skip(self.pos)
  }

  pub fn iter_mut(&mut self) -> impl ExactSizeIterator<Item = &mut T> {
    self.buf.iter_mut().skip(self.pos)
  }
}

impl<'bmp, T: DefaultIn> Deq<'bmp, T> {
  pub fn pop_front(&mut self) -> Option<T> {
    if self.is_empty() {
      return None;
    }
    let mut item = T::default_in(self.bmp);
    std::mem::swap(&mut self.buf[self.pos], &mut item);
    self.pos += 1;
    Some(item)
  }
}

mod tests {
  use super::*;

  #[test]
  fn deq_impl() {
    let bump = Bump::new();
    let mut deq = Deq::new(&bump);
    assert!(deq.is_empty());
    assert_eq!(deq.len(), 0);
    assert_eq!(deq.pop(), None);
    assert_eq!(deq.pop_front(), None);

    deq.push("X");
    assert!(!deq.is_empty());
    assert_eq!(deq.len(), 1);
    assert_eq!("X", buf(&deq));

    deq.push("X");
    assert_eq!(deq.len(), 2);
    assert_eq!("XX", buf(&deq));

    assert_eq!(deq.pop(), Some("X"));
    assert_eq!("X", buf(&deq));

    deq.push("X");
    assert_eq!("XX", buf(&deq));

    deq.push("X");
    deq.push("X");
    assert_eq!("XXXX", buf(&deq));

    assert_eq!(deq.pop_front(), Some("X"));
    assert_eq!("_XXX", buf(&deq));

    assert_eq!(deq.pop_front(), Some("X"));
    assert_eq!("__XX", buf(&deq));

    deq.push("X");
    assert_eq!("__XXX", buf(&deq));

    assert_eq!(deq.pop(), Some("X"));
    assert_eq!("__XX", buf(&deq));

    assert_eq!(deq.pop_front(), Some("X"));
    assert_eq!("___X", buf(&deq));

    deq.push("1");
    assert_eq!("___X1", buf(&deq));
    assert_eq!(deq.len(), 2);

    deq.push("2");
    assert_eq!("___X12", buf(&deq));

    let mut iter = deq.iter();
    assert_eq!(iter.len(), 3);
    assert_eq!(iter.next(), Some(&"X"));
    assert_eq!(iter.next(), Some(&"1"));
    assert_eq!(iter.next(), Some(&"2"));
    assert_eq!(iter.next(), None);
  }

  impl DefaultIn for &'static str {
    fn default_in(_bmp: &Bump) -> Self {
      "_"
    }
  }

  fn buf(deq: &Deq<&'static str>) -> String {
    let mut s = String::new();
    deq.buf.iter().for_each(|str| s.push_str(str));
    s
  }
}
