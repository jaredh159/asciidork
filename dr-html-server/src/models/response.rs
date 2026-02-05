//! Response DTOs

use asciidork_parser::Diagnostic;
use serde::Serialize;

/// Successful conversion response
#[derive(Debug, Serialize)]
pub struct ConvertResponse {
    /// The converted HTML output
    pub html: String,

    /// Any diagnostics (warnings) from parsing
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub diagnostics: Vec<DiagnosticDto>,

    /// Timing information (if requested)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timings: Option<TimingsDto>,
}

/// Diagnostic information (warning or error)
#[derive(Debug, Clone, Serialize)]
pub struct DiagnosticDto {
    /// The full source line where the issue occurred
    pub line: String,

    /// The diagnostic message
    pub message: String,

    /// Line number (1-indexed)
    pub line_num: u32,

    /// Column start position (0-indexed)
    pub column_start: u32,

    /// Column end position (0-indexed)
    pub column_end: u32,

    /// Severity level
    pub severity: Severity,

    /// Source file name (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_file: Option<String>,
}

impl From<Diagnostic> for DiagnosticDto {
    fn from(d: Diagnostic) -> Self {
        Self {
            line: d.line,
            message: d.message,
            line_num: d.line_num,
            column_start: d.underline_start,
            column_end: d.underline_start + d.underline_width,
            severity: Severity::Warning,
            source_file: Some(d.source_file.file_name().to_string()),
        }
    }
}

/// Diagnostic severity level
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
    Info,
}

/// Timing information for performance analysis
#[derive(Debug, Serialize)]
pub struct TimingsDto {
    /// Time spent parsing (milliseconds)
    pub parse_ms: f64,

    /// Time spent converting to HTML (milliseconds)
    pub convert_ms: f64,

    /// Total processing time (milliseconds)
    pub total_ms: f64,

    /// Input size in bytes
    pub input_bytes: usize,
}

/// Error response format
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    /// Error code (machine-readable)
    pub error: String,

    /// Human-readable error message
    pub message: String,

    /// Associated diagnostics (if any)
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub diagnostics: Vec<DiagnosticDto>,
}

/// Health check response
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    /// Health status
    pub status: &'static str,

    /// Server version
    pub version: &'static str,
}

/// Server capabilities and configuration
#[derive(Debug, Serialize)]
pub struct InfoResponse {
    /// Server version
    pub version: &'static str,

    /// Available output formats
    pub formats: Vec<&'static str>,

    /// Available document types
    pub doctypes: Vec<&'static str>,

    /// Available safe modes
    pub safe_modes: Vec<&'static str>,

    /// Server limits
    pub limits: LimitsDto,

    /// Whether unsafe mode is enabled
    pub unsafe_mode_enabled: bool,
}

/// Server limits information
#[derive(Debug, Serialize)]
pub struct LimitsDto {
    /// Maximum content size in bytes
    pub max_content_size_bytes: usize,

    /// Request timeout in seconds
    pub request_timeout_secs: u64,
}
