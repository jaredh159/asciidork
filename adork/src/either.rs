#[derive(Debug, PartialEq, Eq)]
pub enum Either<A, B> {
  Left(A),
  Right(B),
}

impl<A, B> Either<A, B> {
  pub fn is_left(&self) -> bool {
    match self {
      Either::Left(_) => true,
      Either::Right(_) => false,
    }
  }

  pub fn is_right(&self) -> bool {
    match self {
      Either::Left(_) => false,
      Either::Right(_) => true,
    }
  }

  pub fn left(&self) -> Option<&A> {
    match self {
      Either::Left(a) => Some(a),
      Either::Right(_) => None,
    }
  }

  pub fn right(&self) -> Option<&B> {
    match self {
      Either::Left(_) => None,
      Either::Right(b) => Some(b),
    }
  }

  pub fn take_right(self) -> Option<B> {
    match self {
      Either::Left(_) => None,
      Either::Right(b) => Some(b),
    }
  }

  pub fn take_left(self) -> Option<A> {
    match self {
      Either::Left(a) => Some(a),
      Either::Right(_) => None,
    }
  }
}
