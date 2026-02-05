//! Application state

use std::sync::Arc;

use crate::config::Config;

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    /// Server configuration
    pub config: Arc<Config>,
}
