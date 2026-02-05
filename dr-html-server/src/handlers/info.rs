//! Server info endpoint

use axum::{extract::State, Json};

use crate::{
    models::response::{InfoResponse, LimitsDto},
    state::AppState,
};

/// Server info endpoint
///
/// Returns server capabilities, available formats, and configuration limits.
pub async fn info(State(state): State<AppState>) -> Json<InfoResponse> {
    Json(InfoResponse {
        version: env!("CARGO_PKG_VERSION"),
        formats: vec!["dr-html", "dr-html-prettier", "html5", "html5-prettier"],
        doctypes: vec!["article", "book", "manpage", "inline"],
        safe_modes: vec!["unsafe", "safe", "server", "secure"],
        limits: LimitsDto {
            max_content_size_bytes: state.config.max_content_size,
            request_timeout_secs: state.config.request_timeout_secs,
        },
        unsafe_mode_enabled: state.config.allow_unsafe,
    })
}
