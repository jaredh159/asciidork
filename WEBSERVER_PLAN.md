# Asciidork Web Server Implementation Plan

## Overview

This plan outlines the implementation of a REST API web server that exposes all asciidork CLI functionality as HTTP endpoints. The server will be implemented as a new workspace crate using idiomatic Rust patterns.

## Architecture

### New Crate: `asciidork-server`

```
dr-html-server/
├── Cargo.toml
├── src/
│   ├── lib.rs              # Library exports for embedding
│   ├── main.rs             # Binary entry point
│   ├── config.rs           # Configuration (env vars, CLI args)
│   ├── error.rs            # Error types and responses
│   ├── handlers/
│   │   ├── mod.rs
│   │   ├── convert.rs      # Main conversion endpoint
│   │   ├── health.rs       # Health check endpoint
│   │   └── info.rs         # Version/capabilities endpoint
│   ├── models/
│   │   ├── mod.rs
│   │   ├── request.rs      # Request DTOs
│   │   └── response.rs     # Response DTOs
│   └── state.rs            # Application state
```

### Technology Choices

| Component | Choice | Rationale |
|-----------|--------|-----------|
| Web Framework | **axum** | Most idiomatic, async-first, tower ecosystem, excellent ergonomics |
| Runtime | **tokio** | De facto standard async runtime |
| Serialization | **serde** | Already used in asciidork |
| Validation | **validator** | Derive-based validation |
| Docs | **utoipa** | OpenAPI generation |
| Config | **figment** | Layered configuration |
| Tracing | **tracing** | Structured logging |

---

## API Design

### Endpoints

#### `POST /api/v1/convert`

Primary conversion endpoint. Accepts Asciidoc content and returns rendered output.

**Request Body (JSON):**
```json
{
  "content": "= Document Title\n\nHello *world*!",
  "options": {
    "format": "dr-html",
    "doctype": "article",
    "embedded": false,
    "safe_mode": "secure",
    "strict": false,
    "attributes": {
      "author": "John Doe",
      "version": "1.0"
    }
  }
}
```

**Response (Success - 200):**
```json
{
  "html": "<!DOCTYPE html>...",
  "diagnostics": [],
  "timings": {
    "parse_ms": 1.234,
    "convert_ms": 0.567,
    "total_ms": 1.801
  }
}
```

**Response (With Warnings - 200):**
```json
{
  "html": "<!DOCTYPE html>...",
  "diagnostics": [
    {
      "line": "include::missing.adoc[]",
      "message": "include file not found",
      "line_num": 5,
      "column_start": 0,
      "column_end": 22,
      "severity": "warning"
    }
  ],
  "timings": {...}
}
```

**Response (Error - 400/422):**
```json
{
  "error": "parsing_failed",
  "message": "Document parsing failed in strict mode",
  "diagnostics": [...]
}
```

#### `POST /api/v1/convert/multipart`

File upload endpoint for larger documents or when source file path matters.

**Request:** `multipart/form-data`
- `file`: The .adoc file
- `options`: JSON options (same as above)

#### `GET /api/v1/health`

Health check endpoint for load balancers and orchestration.

```json
{
  "status": "healthy",
  "version": "0.33.0"
}
```

#### `GET /api/v1/info`

Capabilities and configuration information.

```json
{
  "version": "0.33.0",
  "formats": ["dr-html", "dr-html-prettier", "html5", "html5-prettier"],
  "doctypes": ["article", "book", "manpage", "inline"],
  "safe_modes": ["unsafe", "safe", "server", "secure"],
  "limits": {
    "max_content_size_bytes": 10485760,
    "max_include_depth": 10
  }
}
```

---

## Request/Response Models

### Request DTOs

```rust
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct ConvertRequest {
    #[validate(length(min = 1, max = 10_485_760))]
    pub content: String,

    #[serde(default)]
    pub options: ConvertOptions,
}

#[derive(Debug, Default, Deserialize)]
pub struct ConvertOptions {
    /// Output format: dr-html (default), dr-html-prettier, html5, html5-prettier
    #[serde(default)]
    pub format: OutputFormat,

    /// Document type: article (default), book, manpage, inline
    #[serde(default)]
    pub doctype: DocType,

    /// Suppress enclosing document structure
    #[serde(default)]
    pub embedded: bool,

    /// Safe mode: unsafe, safe, server, secure (default)
    #[serde(default)]
    pub safe_mode: SafeMode,

    /// Fail on any parsing errors
    #[serde(default)]
    pub strict: bool,

    /// Include timing information in response
    #[serde(default)]
    pub include_timings: bool,

    /// Document attributes (key-value pairs)
    #[serde(default)]
    pub attributes: HashMap<String, AttributeValue>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OutputFormat {
    #[default]
    DrHtml,
    DrHtmlPrettier,
    Html5,
    Html5Prettier,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DocType {
    #[default]
    Article,
    Book,
    Manpage,
    Inline,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SafeMode {
    Unsafe,
    Safe,
    Server,
    #[default]
    Secure,
}

/// Attribute value with optional modifier
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum AttributeValue {
    /// Simple value (readonly)
    Value(String),
    /// Boolean flag
    Flag(bool),
    /// Value with modifier
    WithModifier {
        value: String,
        modifiable: bool,
    },
}
```

### Response DTOs

```rust
#[derive(Debug, Serialize)]
pub struct ConvertResponse {
    pub html: String,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub diagnostics: Vec<DiagnosticDto>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub timings: Option<TimingsDto>,
}

#[derive(Debug, Serialize)]
pub struct DiagnosticDto {
    pub line: String,
    pub message: String,
    pub line_num: u32,
    pub column_start: u32,
    pub column_end: u32,
    pub severity: Severity,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_file: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Serialize)]
pub struct TimingsDto {
    pub parse_ms: f64,
    pub convert_ms: f64,
    pub total_ms: f64,
    pub input_bytes: usize,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub diagnostics: Vec<DiagnosticDto>,
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub version: &'static str,
}

#[derive(Debug, Serialize)]
pub struct InfoResponse {
    pub version: &'static str,
    pub formats: Vec<&'static str>,
    pub doctypes: Vec<&'static str>,
    pub safe_modes: Vec<&'static str>,
    pub limits: LimitsDto,
}

#[derive(Debug, Serialize)]
pub struct LimitsDto {
    pub max_content_size_bytes: usize,
    pub max_include_depth: u8,
}
```

---

## Configuration

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `ASCIIDORK_HOST` | `127.0.0.1` | Bind address |
| `ASCIIDORK_PORT` | `3000` | Bind port |
| `ASCIIDORK_MAX_CONTENT_SIZE` | `10485760` | Max request body (10MB) |
| `ASCIIDORK_DEFAULT_SAFE_MODE` | `secure` | Default safe mode |
| `ASCIIDORK_ALLOW_UNSAFE` | `false` | Allow unsafe mode |
| `ASCIIDORK_CORS_ORIGINS` | `*` | CORS allowed origins |
| `ASCIIDORK_REQUEST_TIMEOUT_SECS` | `30` | Request timeout |
| `ASCIIDORK_LOG_LEVEL` | `info` | Logging level |
| `ASCIIDORK_PRETTIER_PATH` | `prettier` | Path to prettier binary |

### Configuration Struct

```rust
use figment::{Figment, providers::{Env, Serialized}};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub max_content_size: usize,
    pub default_safe_mode: SafeMode,
    pub allow_unsafe: bool,
    pub cors_origins: Vec<String>,
    pub request_timeout_secs: u64,
    pub log_level: String,
    pub prettier_path: Option<String>,
}

impl Config {
    pub fn load() -> Result<Self, figment::Error> {
        Figment::new()
            .merge(Serialized::defaults(Config::default()))
            .merge(Env::prefixed("ASCIIDORK_"))
            .extract()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".into(),
            port: 3000,
            max_content_size: 10 * 1024 * 1024,
            default_safe_mode: SafeMode::Secure,
            allow_unsafe: false,
            cors_origins: vec!["*".into()],
            request_timeout_secs: 30,
            log_level: "info".into(),
            prettier_path: None,
        }
    }
}
```

---

## Error Handling

### Custom Error Type

```rust
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Invalid request: {0}")]
    BadRequest(String),

    #[error("Validation failed: {0}")]
    Validation(#[from] validator::ValidationErrors),

    #[error("Content too large")]
    PayloadTooLarge,

    #[error("Parsing failed")]
    ParsingFailed(Vec<DiagnosticDto>),

    #[error("Unsafe mode not allowed")]
    UnsafeModeDisabled,

    #[error("Format not available: {0}")]
    FormatUnavailable(String),

    #[error("Request timeout")]
    Timeout,

    #[error("Internal error: {0}")]
    Internal(String),
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
            AppError::Validation(errs) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "validation_failed",
                errs.to_string(),
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
            AppError::Timeout => (
                StatusCode::REQUEST_TIMEOUT,
                "timeout",
                "Request processing timed out".into(),
                vec![],
            ),
            AppError::Internal(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
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
```

---

## Handler Implementation

### Convert Handler

```rust
use axum::{extract::State, Json};
use bumpalo::Bump;
use std::time::Instant;

use crate::{
    config::Config,
    error::AppError,
    models::{ConvertRequest, ConvertResponse, DiagnosticDto, TimingsDto},
    state::AppState,
};

pub async fn convert(
    State(state): State<AppState>,
    Json(req): Json<ConvertRequest>,
) -> Result<Json<ConvertResponse>, AppError> {
    // Validate request
    req.validate()?;

    // Check safe mode restrictions
    if matches!(req.options.safe_mode, SafeMode::Unsafe) && !state.config.allow_unsafe {
        return Err(AppError::UnsafeModeDisabled);
    }

    // Process in blocking task (parser is not async)
    let result = tokio::task::spawn_blocking(move || {
        convert_document(&req, &state.config)
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;

    Ok(Json(result))
}

fn convert_document(
    req: &ConvertRequest,
    config: &Config,
) -> Result<ConvertResponse, AppError> {
    let start = Instant::now();
    let input_bytes = req.content.len();

    // Create arena allocator
    let bump = Bump::new();

    // Build job settings from options
    let settings = build_job_settings(&req.options);

    // Create parser
    let source_file = asciidork_parser::prelude::SourceFile::Stdin {
        cwd: std::path::Path::new("."),
    };

    let mut parser = asciidork_parser::prelude::Parser::from_str(
        &req.content,
        source_file,
        &bump,
    );

    parser.apply_job_settings(settings);

    // Parse document
    let parse_start = Instant::now();
    let parse_result = parser.parse();
    let parse_duration = parse_start.elapsed();

    // Handle parse result
    let (document, diagnostics) = match parse_result {
        Ok(result) => (result.document, result.diagnostics),
        Err(errors) if req.options.strict => {
            let diags = errors.into_iter().map(Into::into).collect();
            return Err(AppError::ParsingFailed(diags));
        }
        Err(errors) => {
            // In non-strict mode, this shouldn't happen, but handle it
            let diags = errors.into_iter().map(Into::into).collect();
            return Err(AppError::ParsingFailed(diags));
        }
    };

    // Check for errors in strict mode
    if req.options.strict && !diagnostics.is_empty() {
        let diags = diagnostics.into_iter().map(Into::into).collect();
        return Err(AppError::ParsingFailed(diags));
    }

    // Convert to HTML
    let convert_start = Instant::now();
    let html = match req.options.format {
        OutputFormat::DrHtml | OutputFormat::DrHtmlPrettier => {
            asciidork_dr_html_backend::convert(document)
                .map_err(|e| AppError::Internal(e.to_string()))?
        }
        OutputFormat::Html5 | OutputFormat::Html5Prettier => {
            asciidork_backend_html5s::convert(document)
                .map_err(|e| AppError::Internal(e.to_string()))?
        }
    };
    let convert_duration = convert_start.elapsed();

    // Apply prettier if requested
    let html = if matches!(
        req.options.format,
        OutputFormat::DrHtmlPrettier | OutputFormat::Html5Prettier
    ) {
        apply_prettier(&html, config).unwrap_or(html)
    } else {
        html
    };

    let total_duration = start.elapsed();

    Ok(ConvertResponse {
        html,
        diagnostics: diagnostics.into_iter().map(Into::into).collect(),
        timings: req.options.include_timings.then(|| TimingsDto {
            parse_ms: parse_duration.as_secs_f64() * 1000.0,
            convert_ms: convert_duration.as_secs_f64() * 1000.0,
            total_ms: total_duration.as_secs_f64() * 1000.0,
            input_bytes,
        }),
    })
}

fn build_job_settings(options: &ConvertOptions) -> asciidork_parser::prelude::JobSettings {
    use asciidork_core::*;

    let mut settings = asciidork_parser::prelude::JobSettings::default();

    settings.doctype = Some(match options.doctype {
        DocType::Article => DocType::Article,
        DocType::Book => DocType::Book,
        DocType::Manpage => DocType::Manpage,
        DocType::Inline => DocType::Inline,
    });

    settings.safe_mode = match options.safe_mode {
        SafeMode::Unsafe => SafeMode::Unsafe,
        SafeMode::Safe => SafeMode::Safe,
        SafeMode::Server => SafeMode::Server,
        SafeMode::Secure => SafeMode::Secure,
    };

    settings.embedded = options.embedded;
    settings.strict = options.strict;

    // Apply attributes
    for (key, value) in &options.attributes {
        let attr = match value {
            AttributeValue::Value(v) => JobAttr::Readonly(v.clone().into()),
            AttributeValue::Flag(true) => JobAttr::Readonly("".into()),
            AttributeValue::Flag(false) => JobAttr::Readonly(AttrValue::Bool(false)),
            AttributeValue::WithModifier { value, modifiable: true } => {
                JobAttr::Modifiable(value.clone().into())
            }
            AttributeValue::WithModifier { value, modifiable: false } => {
                JobAttr::Readonly(value.clone().into())
            }
        };
        settings.job_attrs.insert(key.clone(), attr);
    }

    settings
}

fn apply_prettier(html: &str, config: &Config) -> Option<String> {
    let prettier_path = config.prettier_path.as_deref().unwrap_or("prettier");

    std::process::Command::new(prettier_path)
        .args(["--parser", "html"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .ok()
        .and_then(|mut child| {
            use std::io::Write;
            child.stdin.take()?.write_all(html.as_bytes()).ok()?;
            let output = child.wait_with_output().ok()?;
            output.status.success().then(|| {
                String::from_utf8(output.stdout).ok()
            })?
        })
}
```

---

## Application Setup

### Main Entry Point

```rust
use axum::{
    routing::{get, post},
    Router,
};
use tower_http::{
    cors::CorsLayer,
    timeout::TimeoutLayer,
    trace::TraceLayer,
};
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod error;
mod handlers;
mod models;
mod state;

use config::Config;
use state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load configuration
    let config = Config::load()?;

    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(&config.log_level))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Create app state
    let state = AppState {
        config: Arc::new(config.clone()),
    };

    // Build CORS layer
    let cors = CorsLayer::permissive(); // Customize based on config

    // Build router
    let app = Router::new()
        .route("/api/v1/convert", post(handlers::convert::convert))
        .route("/api/v1/convert/multipart", post(handlers::convert::convert_multipart))
        .route("/api/v1/health", get(handlers::health::health))
        .route("/api/v1/info", get(handlers::info::info))
        .layer(TraceLayer::new_for_http())
        .layer(TimeoutLayer::new(Duration::from_secs(config.request_timeout_secs)))
        .layer(cors)
        .with_state(state);

    // Start server
    let addr = SocketAddr::from((
        config.host.parse::<std::net::IpAddr>()?,
        config.port,
    ));

    tracing::info!("Starting server on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
```

### Application State

```rust
use std::sync::Arc;
use crate::config::Config;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
}
```

---

## Cargo.toml

```toml
[package]
name = "asciidork-server"
version = "0.33.0"
edition = "2021"
license = "MIT OR Apache-2.0"
description = "REST API server for asciidork document conversion"
repository = "https://github.com/jaredh159/asciidork"

[[bin]]
name = "asciidork-server"
path = "src/main.rs"

[lib]
name = "asciidork_server"
path = "src/lib.rs"

[dependencies]
# Workspace dependencies
asciidork-ast = { path = "../ast" }
asciidork-core = { path = "../core" }
asciidork-parser = { path = "../parser" }
asciidork-dr-html-backend = { path = "../dr-html-backend" }
asciidork-backend-html5s = { path = "../backend-html5s" }

# Web framework
axum = { version = "0.7", features = ["multipart"] }
tokio = { version = "1", features = ["full"] }
tower-http = { version = "0.5", features = ["cors", "timeout", "trace"] }

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# Validation
validator = { version = "0.18", features = ["derive"] }

# Configuration
figment = { version = "0.10", features = ["env"] }

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Memory allocator (same as CLI)
bumpalo = "3"

# Error handling
thiserror = "1"
anyhow = "1"

# OpenAPI (optional)
utoipa = { version = "4", features = ["axum_extras"], optional = true }
utoipa-swagger-ui = { version = "7", features = ["axum"], optional = true }

[features]
default = []
openapi = ["utoipa", "utoipa-swagger-ui"]

[dev-dependencies]
axum-test = "15"
```

---

## Implementation Tasks

### Phase 1: Core Infrastructure
1. [ ] Create new `dr-html-server` crate in workspace
2. [ ] Add crate to workspace `Cargo.toml` members
3. [ ] Implement configuration loading (`config.rs`)
4. [ ] Implement error types (`error.rs`)
5. [ ] Define request/response models (`models/`)
6. [ ] Set up application state (`state.rs`)

### Phase 2: Handlers
1. [ ] Implement health check handler
2. [ ] Implement info/capabilities handler
3. [ ] Implement main convert handler (JSON body)
4. [ ] Implement multipart convert handler (file upload)
5. [ ] Add attribute parsing and validation

### Phase 3: Integration
1. [ ] Wire up router with all endpoints
2. [ ] Add CORS configuration
3. [ ] Add request timeout handling
4. [ ] Add request body size limiting
5. [ ] Add tracing/logging

### Phase 4: Testing
1. [ ] Unit tests for models and validation
2. [ ] Integration tests for handlers
3. [ ] End-to-end API tests

### Phase 5: Documentation & Polish
1. [ ] OpenAPI specification (optional `openapi` feature)
2. [ ] Update workspace README
3. [ ] Add Docker support
4. [ ] Add example requests (curl, httpie)

---

## CLI Mapping Reference

| CLI Option | API Equivalent | Location |
|------------|----------------|----------|
| `-i, --input <PATH>` | `content` field or multipart file | Request body |
| `-o, --output <PATH>` | N/A (returns in response) | Response body |
| `-f, --format <FORMAT>` | `options.format` | Request body |
| `-d, --doctype <DOCTYPE>` | `options.doctype` | Request body |
| `-e, --embedded` | `options.embedded` | Request body |
| `-a, --attribute <ATTR>` | `options.attributes` | Request body |
| `-s, --safe-mode <MODE>` | `options.safe_mode` | Request body |
| `-B, --base-dir <PATH>` | N/A (server-controlled) | Config |
| `--strict` | `options.strict` | Request body |
| `--json-errors` | Always JSON | Response format |
| `-t, --print-timings` | `options.include_timings` | Request body |

---

## Security Considerations

1. **Safe Mode Enforcement**: Server defaults to `secure` mode; `unsafe` mode must be explicitly enabled via config
2. **Input Size Limits**: Configurable max content size to prevent DoS
3. **Request Timeouts**: Prevent long-running conversions from consuming resources
4. **No File System Access**: API mode doesn't support file includes by default (secure mode)
5. **CORS**: Configurable origins for browser-based clients

---

## Example Usage

### cURL

```bash
# Basic conversion
curl -X POST http://localhost:3000/api/v1/convert \
  -H "Content-Type: application/json" \
  -d '{"content": "= Hello\n\nWorld!"}'

# With options
curl -X POST http://localhost:3000/api/v1/convert \
  -H "Content-Type: application/json" \
  -d '{
    "content": "= Document\n\nHello *world*!",
    "options": {
      "format": "html5",
      "embedded": true,
      "include_timings": true,
      "attributes": {
        "author": "Jane Doe"
      }
    }
  }'

# File upload
curl -X POST http://localhost:3000/api/v1/convert/multipart \
  -F "file=@document.adoc" \
  -F 'options={"format": "dr-html", "embedded": true}'

# Health check
curl http://localhost:3000/api/v1/health

# Server info
curl http://localhost:3000/api/v1/info
```

### HTTPie

```bash
# Basic conversion
http POST :3000/api/v1/convert content="= Hello\n\nWorld!"

# With options
http POST :3000/api/v1/convert \
  content="= Document\n\nHello *world*!" \
  options:='{"format": "html5", "embedded": true}'
```

---

## Future Enhancements

1. **Streaming Response**: For very large documents
2. **Batch Conversion**: Multiple documents in one request
3. **WebSocket Support**: Real-time conversion for editor integrations
4. **Caching**: Cache converted documents by content hash
5. **Rate Limiting**: Per-client request limits
6. **Metrics**: Prometheus endpoint for monitoring
7. **Authentication**: Optional API key or JWT authentication
