//! Document conversion endpoints

use std::{
    io::Write,
    process::{Command, Stdio},
    time::Instant,
};

use axum::{
    extract::{Multipart, State},
    Json,
};
use bumpalo::Bump;

use asciidork_core::{JobAttrs, JobSettings, SafeMode};
use asciidork_dr_html_backend::{AsciidoctorHtml, Backend};
use asciidork_parser::{prelude::{Parser, SourceFile}, Diagnostic};

use crate::{
    config::Config,
    error::AppError,
    models::{
        request::{ConvertOptions, ConvertRequest, OutputFormat, SafeModeOption},
        response::{ConvertResponse, DiagnosticDto, Severity, TimingsDto},
    },
    state::AppState,
};

/// Convert Asciidoc content to HTML (JSON body)
///
/// Accepts Asciidoc content and conversion options, returns HTML.
pub async fn convert(
    State(state): State<AppState>,
    Json(req): Json<ConvertRequest>,
) -> Result<Json<ConvertResponse>, AppError> {
    // Validate content
    if req.content.is_empty() {
        return Err(AppError::BadRequest("Content cannot be empty".into()));
    }

    // Check safe mode restrictions
    if matches!(req.options.safe_mode, SafeModeOption::Unsafe) && !state.config.allow_unsafe {
        return Err(AppError::UnsafeModeDisabled);
    }

    // Process in blocking task (parser is not async)
    let config = state.config.clone();
    let result = tokio::task::spawn_blocking(move || convert_document(&req.content, &req.options, &config))
        .await
        .map_err(|e| AppError::Internal(e.to_string()))??;

    Ok(Json(result))
}

/// Convert Asciidoc content to HTML (multipart file upload)
///
/// Accepts a file upload and optional JSON options.
pub async fn convert_multipart(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<ConvertResponse>, AppError> {
    let mut content: Option<String> = None;
    let mut options = ConvertOptions::default();

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(e.to_string()))?
    {
        let name = field.name().unwrap_or_default().to_string();

        match name.as_str() {
            "file" => {
                let bytes = field
                    .bytes()
                    .await
                    .map_err(|e| AppError::BadRequest(e.to_string()))?;
                content = Some(
                    String::from_utf8(bytes.to_vec())
                        .map_err(|e| AppError::BadRequest(format!("Invalid UTF-8: {}", e)))?,
                );
            }
            "options" => {
                let text = field
                    .text()
                    .await
                    .map_err(|e| AppError::BadRequest(e.to_string()))?;
                options = serde_json::from_str(&text)
                    .map_err(|e| AppError::BadRequest(format!("Invalid options JSON: {}", e)))?;
            }
            _ => {
                // Ignore unknown fields
            }
        }
    }

    let content = content.ok_or_else(|| AppError::BadRequest("No file provided".into()))?;

    if content.is_empty() {
        return Err(AppError::BadRequest("File content cannot be empty".into()));
    }

    // Check safe mode restrictions
    if matches!(options.safe_mode, SafeModeOption::Unsafe) && !state.config.allow_unsafe {
        return Err(AppError::UnsafeModeDisabled);
    }

    // Process in blocking task
    let config = state.config.clone();
    let result = tokio::task::spawn_blocking(move || convert_document(&content, &options, &config))
        .await
        .map_err(|e| AppError::Internal(e.to_string()))??;

    Ok(Json(result))
}

/// Perform the actual document conversion
fn convert_document(
    content: &str,
    options: &ConvertOptions,
    config: &Config,
) -> Result<ConvertResponse, AppError> {
    let start = Instant::now();
    let input_bytes = content.len();

    // Create arena allocator
    let bump = Bump::with_capacity(content.len() * 2);

    // Build job settings from options
    let job_settings = build_job_settings(options)?;

    // Create parser
    let source_file = SourceFile::Stdin {
        cwd: asciidork_core::Path::new("."),
    };

    let mut parser = Parser::from_str(content, source_file, &bump);

    // Apply settings
    let mut settings = job_settings;
    AsciidoctorHtml::set_job_attrs(&mut settings.job_attrs);
    parser.apply_job_settings(settings);

    // Parse document
    let parse_start = Instant::now();
    let result = parser.parse();
    let parse_duration = parse_start.elapsed();

    // Handle parse result
    let (document, warnings) = match result {
        Ok(parse_result) => (parse_result.document, parse_result.warnings),
        Err(errors) => {
            // In strict mode or on fatal errors, return error
            let diags: Vec<DiagnosticDto> = errors
                .into_iter()
                .map(|d: Diagnostic| {
                    let mut dto: DiagnosticDto = d.into();
                    dto.severity = Severity::Error;
                    dto
                })
                .collect();
            return Err(AppError::ParsingFailed(diags));
        }
    };

    // Check for warnings in strict mode (treat as errors)
    if options.strict && !warnings.is_empty() {
        let diags: Vec<DiagnosticDto> = warnings
            .into_iter()
            .map(|d: Diagnostic| {
                let mut dto: DiagnosticDto = d.into();
                dto.severity = Severity::Error;
                dto
            })
            .collect();
        return Err(AppError::ParsingFailed(diags));
    }

    // Convert to HTML
    let convert_start = Instant::now();
    let html = match options.format {
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
        options.format,
        OutputFormat::DrHtmlPrettier | OutputFormat::Html5Prettier
    ) {
        format_html(&html, config).unwrap_or(html)
    } else {
        html
    };

    let total_duration = start.elapsed();

    Ok(ConvertResponse {
        html,
        diagnostics: warnings.into_iter().map(Into::into).collect(),
        timings: options.include_timings.then(|| TimingsDto {
            parse_ms: parse_duration.as_secs_f64() * 1000.0,
            convert_ms: convert_duration.as_secs_f64() * 1000.0,
            total_ms: total_duration.as_secs_f64() * 1000.0,
            input_bytes,
        }),
    })
}

/// Build JobSettings from request options
fn build_job_settings(options: &ConvertOptions) -> Result<JobSettings, AppError> {
    let mut job_attrs = JobAttrs::empty();

    // Apply custom attributes
    for (key, value) in &options.attributes {
        let job_attr = value.to_job_attr();
        job_attrs
            .insert(key.clone(), job_attr)
            .map_err(|e| AppError::InvalidAttribute(e))?;
    }

    Ok(JobSettings {
        doctype: Some(options.doctype.into()),
        safe_mode: SafeMode::from(options.safe_mode),
        job_attrs,
        embedded: options.embedded,
        strict: options.strict,
    })
}

/// Format HTML using prettier
fn format_html(html: &str, config: &Config) -> Option<String> {
    let prettier_path = config.prettier_path.as_deref().unwrap_or("prettier");

    let mut child = Command::new(prettier_path)
        .args(["--parser", "html", "--html-whitespace-sensitivity", "ignore"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;

    let stdin = child.stdin.as_mut()?;
    stdin.write_all(html.as_bytes()).ok()?;

    let output = child.wait_with_output().ok()?;
    if output.status.success() {
        String::from_utf8(output.stdout).ok()
    } else {
        None
    }
}
