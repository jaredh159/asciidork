mod attrs;
mod doctype;
mod document_meta;
pub mod file;
mod job_attrs;
mod job_settings;
mod path;
mod special_sect;
mod types;
mod validate;

pub use internal::types::*;

#[macro_export]
macro_rules! iff {
  ($condition:expr,  $_true:expr, $_false:expr) => {
    if $condition { $_true } else { $_false }
  };
}

mod internal {
  pub(crate) mod types {
    pub use crate::attrs::*;
    pub use crate::doctype::*;
    pub use crate::document_meta::*;
    pub use crate::job_attrs::*;
    pub use crate::job_settings::*;
    pub use crate::path::*;
    pub use crate::special_sect::*;
    pub use crate::types::*;
  }
  pub use types::*;
}
