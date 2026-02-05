//! Error types and HTTP response handling

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use thiserror::Error;

use crate::models::response::{DiagnosticDto, ErrorResponse};

/// Application error type
#[derive(Debug, Error)]
pub enum AppError {
    #[error("Invalid request: {0}")]
    BadRequest(String),

    #[error("Content too large")]
    PayloadTooLarge,

    #[error("Parsing failed")]
    ParsingFailed(Vec<DiagnosticDto>),

    #[error("Unsafe mode not allowed")]
    UnsafeModeDisabled,

    #[error("Format not available: {0}")]
    FormatUnavailable(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Invalid attribute: {0}")]
    InvalidAttribute(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_code, message, diagnostics) = match &self {
            AppError::BadRequest(msg) => (
                StatusCode::BAD_REQUEST,
                "bad_request",
                msg.clone(),
                vec![],
            ),
            AppError::PayloadTooLarge => (
                StatusCode::PAYLOAD_TOO_LARGE,
                "payload_too_large",
                "Content exceeds maximum size".into(),
                vec![],
            ),
            AppError::ParsingFailed(diags) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "parsing_failed",
                "Document parsing failed".into(),
                diags.clone(),
            ),
            AppError::UnsafeModeDisabled => (
                StatusCode::FORBIDDEN,
                "unsafe_mode_disabled",
                "Unsafe mode is disabled on this server".into(),
                vec![],
            ),
            AppError::FormatUnavailable(fmt) => (
                StatusCode::BAD_REQUEST,
                "format_unavailable",
                format!("Format '{}' is not available", fmt),
                vec![],
            ),
            AppError::Internal(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                msg.clone(),
                vec![],
            ),
            AppError::InvalidAttribute(msg) => (
                StatusCode::BAD_REQUEST,
                "invalid_attribute",
                msg.clone(),
                vec![],
            ),
        };

        let body = ErrorResponse {
            error: error_code.into(),
            message,
            diagnostics,
        };

        (status, Json(body)).into_response()
    }
}
