//! Server configuration

use figment::{providers::Env, Figment};
use serde::Deserialize;

/// Server configuration loaded from environment variables.
///
/// All environment variables are prefixed with `ASCIIDORK_`.
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    /// Host address to bind to (default: 127.0.0.1)
    #[serde(default = "default_host")]
    pub host: String,

    /// Port to bind to (default: 3000)
    #[serde(default = "default_port")]
    pub port: u16,

    /// Maximum content size in bytes (default: 10MB)
    #[serde(default = "default_max_content_size")]
    pub max_content_size: usize,

    /// Default safe mode for requests (default: secure)
    #[serde(default)]
    pub default_safe_mode: SafeModeConfig,

    /// Allow unsafe mode (default: false)
    #[serde(default)]
    pub allow_unsafe: bool,

    /// CORS allowed origins (default: ["*"])
    #[serde(default = "default_cors_origins")]
    pub cors_origins: Vec<String>,

    /// Request timeout in seconds (default: 30)
    #[serde(default = "default_request_timeout")]
    pub request_timeout_secs: u64,

    /// Log level (default: info)
    #[serde(default = "default_log_level")]
    pub log_level: String,

    /// Path to prettier binary for prettier output formats
    #[serde(default)]
    pub prettier_path: Option<String>,
}

fn default_host() -> String {
    "127.0.0.1".into()
}

fn default_port() -> u16 {
    3000
}

fn default_max_content_size() -> usize {
    10 * 1024 * 1024 // 10MB
}

fn default_cors_origins() -> Vec<String> {
    vec!["*".into()]
}

fn default_request_timeout() -> u64 {
    30
}

fn default_log_level() -> String {
    "info".into()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
            max_content_size: default_max_content_size(),
            default_safe_mode: SafeModeConfig::default(),
            allow_unsafe: false,
            cors_origins: default_cors_origins(),
            request_timeout_secs: default_request_timeout(),
            log_level: default_log_level(),
            prettier_path: None,
        }
    }
}

impl Config {
    /// Load configuration from environment variables.
    ///
    /// Environment variables are prefixed with `ASCIIDORK_`, e.g.:
    /// - `ASCIIDORK_HOST`
    /// - `ASCIIDORK_PORT`
    /// - `ASCIIDORK_MAX_CONTENT_SIZE`
    pub fn load() -> Result<Self, figment::Error> {
        Figment::new()
            .merge(Env::prefixed("ASCIIDORK_").split("_"))
            .extract()
    }
}

/// Safe mode configuration value
#[derive(Debug, Clone, Copy, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SafeModeConfig {
    Unsafe,
    Safe,
    Server,
    #[default]
    Secure,
}

impl From<SafeModeConfig> for asciidork_core::SafeMode {
    fn from(config: SafeModeConfig) -> Self {
        match config {
            SafeModeConfig::Unsafe => asciidork_core::SafeMode::Unsafe,
            SafeModeConfig::Safe => asciidork_core::SafeMode::Safe,
            SafeModeConfig::Server => asciidork_core::SafeMode::Server,
            SafeModeConfig::Secure => asciidork_core::SafeMode::Secure,
        }
    }
}
