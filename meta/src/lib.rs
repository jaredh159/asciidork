mod attrs;
mod doctype;
mod document_meta;
mod job_attrs;
mod types;

pub use internal::types::*;

mod internal {
  pub(crate) mod types {
    pub use crate::attrs::*;
    pub use crate::doctype::*;
    pub use crate::document_meta::*;
    pub use crate::job_attrs::*;
    pub use crate::types::*;
  }
  pub use types::*;
}
