//! Request DTOs

use std::collections::HashMap;

use serde::Deserialize;

/// Request body for the convert endpoint
#[derive(Debug, Deserialize)]
pub struct ConvertRequest {
    /// The Asciidoc content to convert
    pub content: String,

    /// Conversion options (all optional with defaults)
    #[serde(default)]
    pub options: ConvertOptions,
}

/// Conversion options matching CLI arguments
#[derive(Debug, Default, Deserialize)]
pub struct ConvertOptions {
    /// Output format: dr-html (default), dr-html-prettier, html5, html5-prettier
    #[serde(default)]
    pub format: OutputFormat,

    /// Document type: article (default), book, manpage, inline
    #[serde(default)]
    pub doctype: DocTypeOption,

    /// Suppress enclosing document structure (no html/head/body tags)
    #[serde(default)]
    pub embedded: bool,

    /// Safe mode: unsafe, safe, server, secure (default)
    #[serde(default)]
    pub safe_mode: SafeModeOption,

    /// Fail on any parsing errors (default: false, warnings only)
    #[serde(default)]
    pub strict: bool,

    /// Include timing information in response
    #[serde(default)]
    pub include_timings: bool,

    /// Document attributes (key-value pairs)
    #[serde(default)]
    pub attributes: HashMap<String, AttributeValue>,
}

/// Output format selection
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OutputFormat {
    /// Asciidoctor-compatible HTML (default)
    #[default]
    DrHtml,
    /// Asciidoctor-compatible HTML, formatted with prettier
    DrHtmlPrettier,
    /// Semantic HTML5
    Html5,
    /// Semantic HTML5, formatted with prettier
    Html5Prettier,
}

/// Document type selection
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DocTypeOption {
    #[default]
    Article,
    Book,
    Manpage,
    Inline,
}

impl From<DocTypeOption> for asciidork_core::DocType {
    fn from(opt: DocTypeOption) -> Self {
        match opt {
            DocTypeOption::Article => asciidork_core::DocType::Article,
            DocTypeOption::Book => asciidork_core::DocType::Book,
            DocTypeOption::Manpage => asciidork_core::DocType::Manpage,
            DocTypeOption::Inline => asciidork_core::DocType::Inline,
        }
    }
}

/// Safe mode selection
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SafeModeOption {
    /// No restrictions (dangerous - must be explicitly enabled on server)
    Unsafe,
    /// Minimal restrictions
    Safe,
    /// Server-oriented restrictions
    Server,
    /// Maximum restrictions (default)
    #[default]
    Secure,
}

impl From<SafeModeOption> for asciidork_core::SafeMode {
    fn from(opt: SafeModeOption) -> Self {
        match opt {
            SafeModeOption::Unsafe => asciidork_core::SafeMode::Unsafe,
            SafeModeOption::Safe => asciidork_core::SafeMode::Safe,
            SafeModeOption::Server => asciidork_core::SafeMode::Server,
            SafeModeOption::Secure => asciidork_core::SafeMode::Secure,
        }
    }
}

/// Attribute value with optional modifier
///
/// Supports multiple formats:
/// - Simple string value: `"value"` (readonly)
/// - Boolean flag: `true` or `false`
/// - Object with modifiable flag: `{"value": "x", "modifiable": true}`
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum AttributeValue {
    /// Simple string value (readonly by default)
    String(String),
    /// Boolean flag
    Bool(bool),
    /// Value with explicit modifiable setting
    WithModifier {
        value: String,
        #[serde(default)]
        modifiable: bool,
    },
}

impl AttributeValue {
    /// Convert to a JobAttr for the parser
    pub fn to_job_attr(&self) -> asciidork_core::JobAttr {
        match self {
            AttributeValue::String(v) => asciidork_core::JobAttr::readonly(v.as_str()),
            AttributeValue::Bool(true) => asciidork_core::JobAttr::readonly(true),
            AttributeValue::Bool(false) => asciidork_core::JobAttr::readonly(false),
            AttributeValue::WithModifier {
                value,
                modifiable: true,
            } => asciidork_core::JobAttr::modifiable(value.as_str()),
            AttributeValue::WithModifier {
                value,
                modifiable: false,
            } => asciidork_core::JobAttr::readonly(value.as_str()),
        }
    }
}

/// Options for multipart file upload
#[derive(Debug, Default, Deserialize)]
pub struct MultipartOptions {
    /// Conversion options (same as JSON body)
    #[serde(flatten)]
    pub options: ConvertOptions,
}
