pub mod bump {
  pub use bumpalo::collections::String;
  pub use bumpalo::collections::Vec;
  pub use bumpalo::vec as bvec;
  pub use bumpalo::Bump;
  pub use std::string::String as StdString;
  pub use std::vec::Vec as StdVec;
}
