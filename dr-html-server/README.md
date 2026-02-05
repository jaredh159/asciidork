# asciidork-server

REST API server for converting Asciidoc documents to HTML using the asciidork parser.

## Installation

```bash
cargo install asciidork-server
```

Or build from source:

```bash
cargo build --release -p asciidork-server
```

## Usage

Start the server:

```bash
asciidork-server
```

The server listens on `127.0.0.1:3000` by default.

## Configuration

Configure via environment variables (prefixed with `ASCIIDORK_`):

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

## API Endpoints

### POST /api/v1/convert

Convert Asciidoc content to HTML.

**Request:**

```json
{
  "content": "= Document Title\n\nHello *world*!",
  "options": {
    "format": "dr-html",
    "doctype": "article",
    "embedded": false,
    "safe_mode": "secure",
    "strict": false,
    "include_timings": true,
    "attributes": {
      "author": "John Doe"
    }
  }
}
```

**Response:**

```json
{
  "html": "<!DOCTYPE html>...",
  "diagnostics": [],
  "timings": {
    "parse_ms": 1.234,
    "convert_ms": 0.567,
    "total_ms": 1.801,
    "input_bytes": 42
  }
}
```

### POST /api/v1/convert/multipart

Convert an uploaded Asciidoc file.

**Request:** `multipart/form-data`
- `file`: The .adoc file
- `options`: JSON options (same as above)

### GET /api/v1/health

Health check endpoint.

```json
{
  "status": "healthy",
  "version": "0.33.0"
}
```

### GET /api/v1/info

Server capabilities and configuration.

```json
{
  "version": "0.33.0",
  "formats": ["dr-html", "dr-html-prettier", "html5", "html5-prettier"],
  "doctypes": ["article", "book", "manpage", "inline"],
  "safe_modes": ["unsafe", "safe", "server", "secure"],
  "limits": {
    "max_content_size_bytes": 10485760,
    "request_timeout_secs": 30
  },
  "unsafe_mode_enabled": false
}
```

## Options Reference

### format

Output format for HTML rendering:

- `dr-html` (default) - Asciidoctor-compatible HTML
- `dr-html-prettier` - Asciidoctor-compatible HTML, formatted with prettier
- `html5` - Semantic HTML5 (jirutka-based)
- `html5-prettier` - Semantic HTML5, formatted with prettier

### doctype

Document type:

- `article` (default) - Standard article
- `book` - Book document with parts/chapters
- `manpage` - Unix man page
- `inline` - Inline content only

### safe_mode

Security level:

- `unsafe` - No restrictions (must be enabled on server)
- `safe` - Minimal restrictions
- `server` - Server-oriented restrictions
- `secure` (default) - Maximum restrictions

### embedded

When `true`, suppresses enclosing document structure (`<html>`, `<head>`, `<body>` tags).

### strict

When `true`, fails on any parsing warnings (treats them as errors).

### attributes

Document attributes as key-value pairs. Supports:

- String values: `"author": "John Doe"`
- Boolean flags: `"toc": true`
- Modifiable values: `"version": {"value": "1.0", "modifiable": true}`

## Examples

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
      "include_timings": true
    }
  }'

# File upload
curl -X POST http://localhost:3000/api/v1/convert/multipart \
  -F "file=@document.adoc" \
  -F 'options={"format": "dr-html"}'
```

### HTTPie

```bash
http POST :3000/api/v1/convert content="= Hello\n\nWorld!"
```

## License

MIT OR Apache-2.0
