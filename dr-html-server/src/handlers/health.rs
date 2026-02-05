//! Health check endpoint

use axum::Json;

use crate::models::response::HealthResponse;

/// Health check endpoint
///
/// Returns server health status and version.
pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy",
        version: env!("CARGO_PKG_VERSION"),
    })
}
