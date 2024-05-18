mod attrs;
mod doctype;
mod document_attrs;
mod task_attrs;
mod types;

pub use internal::types::*;

mod internal {
  pub(crate) mod types {
    pub use crate::attrs::*;
    pub use crate::doctype::*;
    pub use crate::document_attrs::*;
    pub use crate::task_attrs::*;
    pub use crate::types::*;
  }
  pub use types::*;
}
