pub mod bump {
  pub use bumpalo::collections::BumpString;
  pub use bumpalo::collections::Vec;
  pub use bumpalo::vec as bvec;
  pub use bumpalo::Bump;
  pub use std::vec::Vec as StdVec;
}
