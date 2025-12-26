use std::fmt;

use crate::internal::*;

/// A Vec-like data structure that allows efficient popping from front
/// by replacing the popped item with a default value and maintaining
/// an internal position. It does not use a real ring buffer, so pushing
/// to the front should only occur when we have a position to overwrite
#[derive(Clone)]
pub struct Deq<'arena, T> {
  pub bump: &'arena Bump,
  buf: BumpVec<'arena, T>,
  pos: usize,
}

impl<'arena, T> Deq<'arena, T> {
  pub fn new(bump: &'arena Bump) -> Self {
    Deq {
      bump,
      buf: BumpVec::new_in(bump),
      pos: 0,
    }
  }

  pub fn with_capacity(capacity: usize, bump: &'arena Bump) -> Self {
    Deq {
      bump,
      buf: BumpVec::with_capacity_in(capacity, bump),
      pos: 0,
    }
  }

  pub const fn from_vec(bump: &'arena Bump, buf: BumpVec<'arena, T>) -> Self {
    Deq { bump, buf, pos: 0 }
  }

  pub fn clear(&mut self) {
    self.buf.clear();
    self.pos = 0;
  }

  pub fn extend(&mut self, other: impl IntoIterator<Item = T>) {
    self.buf.extend(other);
  }

  pub fn get(&self, index: usize) -> Option<&T> {
    self.buf.get(index + self.pos)
  }

  pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
    self.buf.get_mut(index + self.pos)
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

  pub fn truncate(&mut self, len: usize) {
    self.buf.truncate(len);
  }

  pub fn is_empty(&self) -> bool {
    self.buf.len() - self.pos == 0
  }

  pub fn len(&self) -> usize {
    self.buf.len() - self.pos
  }

  pub fn first(&self) -> Option<&T> {
    self.buf.get(self.pos)
  }

  pub fn last(&self) -> Option<&T> {
    if self.is_empty() {
      return None;
    }
    self.buf.last()
  }

  pub fn last_mut(&mut self) -> Option<&mut T> {
    if self.is_empty() {
      return None;
    }
    self.buf.last_mut()
  }

  pub fn iter(&self) -> impl ExactSizeIterator<Item = &T> {
    self.buf.iter().skip(self.pos)
  }

  pub fn iter_mut(&mut self) -> impl ExactSizeIterator<Item = &mut T> {
    self.buf.iter_mut().skip(self.pos)
  }

  pub fn into_iter(self) -> impl ExactSizeIterator<Item = T> + 'arena {
    self.buf.into_iter().skip(self.pos)
  }

  pub fn reserve(&mut self, additional: usize) {
    self.buf.reserve(additional + self.pos);
  }

  pub const fn remove_first(&mut self) {
    self.pos += 1;
  }

  // this is not meant to be a general-purpose method, rather
  // should only be called when we know we have a slot to overwrite
  // which is why there is a assert! to catch misuse
  pub fn restore_front(&mut self, item: T) {
    assert!(self.pos != 0, "unexpected O(n) push_front in Deq");
    self.pos -= 1;
    self.buf[self.pos] = item;
  }

  pub fn slowly_push_front(&mut self, item: T) {
    if self.pos == 0 {
      self.buf.insert(0, item);
    } else {
      self.pos -= 1;
      self.buf[self.pos] = item;
    }
  }

  pub fn as_slice(&self) -> &[T] {
    &self.buf[self.pos..]
  }

  pub fn into_vec(self) -> BumpVec<'arena, T> {
    BumpVec::from_iter_in(self.buf.into_iter().skip(self.pos), self.bump)
  }
}

pub trait DefaultIn<'a> {
  fn default_in(bump: &'a Bump) -> Self;
}

impl<'arena, T: DefaultIn<'arena>> Deq<'arena, T> {
  #[must_use]
  pub fn pop_front(&mut self) -> Option<T> {
    if self.is_empty() {
      return None;
    }
    let mut item = T::default_in(self.bump);
    std::mem::swap(&mut self.buf[self.pos], &mut item);
    self.pos += 1;
    Some(item)
  }
}

impl<T: fmt::Debug> fmt::Debug for Deq<'_, T> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(
      f,
      "{:?}",
      self.buf.iter().skip(self.pos).collect::<Vec<_>>()
    )
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use test_utils::*;

  #[test]
  fn deq_impl() {
    let mut deq = Deq::new(leaked_bump());
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

  impl DefaultIn<'static> for &'static str {
    fn default_in(_: &Bump) -> Self {
      "_"
    }
  }

  fn buf(deq: &Deq<&'static str>) -> String {
    let mut s = String::new();
    deq.buf.iter().for_each(|str| s.push_str(str));
    s
  }
}
