# Asciidork Server

A REST API server for converting Asciidoc documents to HTML using the asciidork parser.

## Table of Contents

- [Installation](#installation)
- [Quick Start](#quick-start)
- [Configuration](#configuration)
- [API Reference](#api-reference)
- [Examples](#examples)
- [Docker](#docker)
- [Security](#security)

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/jaredh159/asciidork.git
cd asciidork

# Build release binary
cargo build --release -p asciidork-server

# Binary located at target/release/asciidork-server
```

### Using Cargo

```bash
cargo install asciidork-server
```

## Quick Start

```bash
# Start server with defaults (localhost:3000)
asciidork-server

# Start on custom port
ASCIIDORK_PORT=8080 asciidork-server

# Start with debug logging
ASCIIDORK_LOG_LEVEL=debug asciidork-server
```

Test the server:

```bash
# Health check
curl http://localhost:3000/api/v1/health

# Convert a document
curl -X POST http://localhost:3000/api/v1/convert \
  -H "Content-Type: application/json" \
  -d '{"content": "= Hello World\n\nThis is *Asciidoc*!"}'
```

## Configuration

All configuration is done via environment variables with the `ASCIIDORK_` prefix.

### Environment Variables

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `ASCIIDORK_HOST` | string | `127.0.0.1` | IP address to bind to |
| `ASCIIDORK_PORT` | integer | `3000` | Port to listen on |
| `ASCIIDORK_MAX_CONTENT_SIZE` | integer | `10485760` | Maximum request body size in bytes (default: 10MB) |
| `ASCIIDORK_REQUEST_TIMEOUT_SECS` | integer | `30` | Request timeout in seconds |
| `ASCIIDORK_LOG_LEVEL` | string | `info` | Log level: `trace`, `debug`, `info`, `warn`, `error` |
| `ASCIIDORK_CORS_ORIGINS` | string | `*` | Comma-separated list of allowed CORS origins |
| `ASCIIDORK_ALLOW_UNSAFE` | boolean | `false` | Allow unsafe mode (enables file includes) |
| `ASCIIDORK_DEFAULT_SAFE_MODE` | string | `secure` | Default safe mode: `unsafe`, `safe`, `server`, `secure` |
| `ASCIIDORK_PRETTIER_PATH` | string | `prettier` | Path to prettier binary (for prettier output formats) |

### Configuration Examples

#### Development

```bash
export ASCIIDORK_HOST=127.0.0.1
export ASCIIDORK_PORT=3000
export ASCIIDORK_LOG_LEVEL=debug
export ASCIIDORK_ALLOW_UNSAFE=true
```

#### Production

```bash
export ASCIIDORK_HOST=0.0.0.0
export ASCIIDORK_PORT=8080
export ASCIIDORK_LOG_LEVEL=info
export ASCIIDORK_ALLOW_UNSAFE=false
export ASCIIDORK_CORS_ORIGINS=https://example.com,https://app.example.com
export ASCIIDORK_REQUEST_TIMEOUT_SECS=60
export ASCIIDORK_MAX_CONTENT_SIZE=52428800  # 50MB
```

## API Reference

### Base URL

```
http://localhost:3000/api/v1
```

### Endpoints

#### POST /convert

Convert Asciidoc content to HTML.

**Request Headers:**
- `Content-Type: application/json`

**Request Body:**

```json
{
  "content": "string (required)",
  "options": {
    "format": "dr-html | dr-html-prettier | html5 | html5-prettier",
    "doctype": "article | book | manpage | inline",
    "embedded": "boolean",
    "safe_mode": "unsafe | safe | server | secure",
    "strict": "boolean",
    "include_timings": "boolean",
    "attributes": {
      "key": "value"
    }
  }
}
```

**Request Fields:**

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `content` | string | Yes | - | Asciidoc source content |
| `options.format` | string | No | `dr-html` | Output format |
| `options.doctype` | string | No | `article` | Document type |
| `options.embedded` | boolean | No | `false` | Omit HTML wrapper (no html/head/body) |
| `options.safe_mode` | string | No | `secure` | Security level |
| `options.strict` | boolean | No | `false` | Fail on warnings |
| `options.include_timings` | boolean | No | `false` | Include timing metrics |
| `options.attributes` | object | No | `{}` | Document attributes |

**Response (200 OK):**

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
      "severity": "warning",
      "source_file": "stdin"
    }
  ],
  "timings": {
    "parse_ms": 1.234,
    "convert_ms": 0.567,
    "total_ms": 1.801,
    "input_bytes": 42
  }
}
```

**Error Response (4xx/5xx):**

```json
{
  "error": "error_code",
  "message": "Human readable message",
  "diagnostics": []
}
```

**Error Codes:**

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `bad_request` | 400 | Invalid request format |
| `invalid_attribute` | 400 | Invalid document attribute |
| `format_unavailable` | 400 | Requested format not available |
| `unsafe_mode_disabled` | 403 | Unsafe mode not allowed on this server |
| `parsing_failed` | 422 | Document parsing failed (strict mode) |
| `payload_too_large` | 413 | Content exceeds max size |
| `internal_error` | 500 | Internal server error |

---

#### POST /convert/multipart

Convert an uploaded Asciidoc file.

**Request Headers:**
- `Content-Type: multipart/form-data`

**Form Fields:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `file` | file | Yes | The .adoc file to convert |
| `options` | string | No | JSON string with conversion options |

**Response:** Same as `/convert`

---

#### GET /health

Health check endpoint for load balancers and orchestration.

**Response (200 OK):**

```json
{
  "status": "healthy",
  "version": "0.34.0"
}
```

---

#### GET /info

Server capabilities and configuration.

**Response (200 OK):**

```json
{
  "version": "0.34.0",
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

## Examples

### cURL

```bash
# Basic conversion
curl -X POST http://localhost:3000/api/v1/convert \
  -H "Content-Type: application/json" \
  -d '{"content": "= Title\n\nParagraph with *bold* text."}'

# Embedded HTML (no wrapper)
curl -X POST http://localhost:3000/api/v1/convert \
  -H "Content-Type: application/json" \
  -d '{
    "content": "Hello *world*!",
    "options": {"embedded": true}
  }'

# HTML5 format with timings
curl -X POST http://localhost:3000/api/v1/convert \
  -H "Content-Type: application/json" \
  -d '{
    "content": "= Document\n\nContent here.",
    "options": {
      "format": "html5",
      "include_timings": true
    }
  }'

# With document attributes
curl -X POST http://localhost:3000/api/v1/convert \
  -H "Content-Type: application/json" \
  -d '{
    "content": "= {doctitle}\n\nBy {author}",
    "options": {
      "attributes": {
        "doctitle": "My Document",
        "author": "Jane Doe"
      }
    }
  }'

# File upload
curl -X POST http://localhost:3000/api/v1/convert/multipart \
  -F "file=@document.adoc" \
  -F 'options={"format": "html5", "embedded": true}'

# Strict mode (fail on warnings)
curl -X POST http://localhost:3000/api/v1/convert \
  -H "Content-Type: application/json" \
  -d '{
    "content": "= Title\n\nContent",
    "options": {"strict": true}
  }'
```

### HTTPie

```bash
# Basic conversion
http POST :3000/api/v1/convert content="= Hello\n\nWorld"

# With options
http POST :3000/api/v1/convert \
  content="= Doc\n\nText" \
  options:='{"format": "html5", "embedded": true}'
```

### Python

```python
import requests

response = requests.post(
    "http://localhost:3000/api/v1/convert",
    json={
        "content": "= My Document\n\nHello *world*!",
        "options": {
            "format": "html5",
            "embedded": True,
            "include_timings": True
        }
    }
)

result = response.json()
print(result["html"])
print(f"Parsed in {result['timings']['parse_ms']}ms")
```

### JavaScript/Node.js

```javascript
const response = await fetch("http://localhost:3000/api/v1/convert", {
  method: "POST",
  headers: { "Content-Type": "application/json" },
  body: JSON.stringify({
    content: "= My Document\n\nHello *world*!",
    options: {
      format: "html5",
      embedded: true
    }
  })
});

const { html, diagnostics } = await response.json();
console.log(html);
```

### Go

```go
package main

import (
    "bytes"
    "encoding/json"
    "net/http"
)

func main() {
    payload := map[string]interface{}{
        "content": "= Hello\n\nWorld",
        "options": map[string]interface{}{
            "embedded": true,
        },
    }

    body, _ := json.Marshal(payload)
    resp, _ := http.Post(
        "http://localhost:3000/api/v1/convert",
        "application/json",
        bytes.NewBuffer(body),
    )
    defer resp.Body.Close()

    var result map[string]interface{}
    json.NewDecoder(resp.Body).Decode(&result)
}
```

## Docker

### Building the Docker Container

The Dockerfile uses a multi-stage build. The build context **must** be the
workspace root (not the `dr-html-server/` directory) because the build copies
all workspace crates required by Cargo.

```bash
# From the workspace root:
docker build -t asciidork-server -f dr-html-server/Dockerfile .

# Run the container
docker run -p 3000:3000 asciidork-server

# Run with custom configuration
docker run -p 8080:8080 \
  -e ASCIIDORK_PORT=8080 \
  -e ASCIIDORK_LOG_LEVEL=debug \
  asciidork-server
```

### Docker Compose

A `docker-compose.yml` is provided in the `dr-html-server/` directory. It
already sets the correct build context (`..`, i.e. the workspace root):

```bash
# From the workspace root:
docker compose -f dr-html-server/docker-compose.yml up -d

# Or from the dr-html-server directory:
cd dr-html-server
docker compose up -d

# Stop the server:
docker compose -f dr-html-server/docker-compose.yml down
```

Override settings with environment variables or a `.env` file:

```bash
ASCIIDORK_LOG_LEVEL=debug ASCIIDORK_PORT=8080 \
  docker compose -f dr-html-server/docker-compose.yml up -d
```

### Docker Compose (Custom)

```yaml
services:
  asciidork:
    build:
      context: .           # workspace root
      dockerfile: dr-html-server/Dockerfile
    ports:
      - "3000:3000"
    environment:
      - ASCIIDORK_HOST=0.0.0.0
      - ASCIIDORK_PORT=3000
      - ASCIIDORK_LOG_LEVEL=info
      - ASCIIDORK_MAX_CONTENT_SIZE=52428800
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3000/api/v1/health"]
      interval: 30s
      timeout: 10s
      retries: 3
    restart: unless-stopped
```

### Kubernetes

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: asciidork-server
spec:
  replicas: 3
  selector:
    matchLabels:
      app: asciidork-server
  template:
    metadata:
      labels:
        app: asciidork-server
    spec:
      containers:
      - name: asciidork-server
        image: ghcr.io/jaredh159/asciidork-server:latest
        ports:
        - containerPort: 3000
        env:
        - name: ASCIIDORK_HOST
          value: "0.0.0.0"
        - name: ASCIIDORK_PORT
          value: "3000"
        livenessProbe:
          httpGet:
            path: /api/v1/health
            port: 3000
          initialDelaySeconds: 5
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /api/v1/health
            port: 3000
          initialDelaySeconds: 5
          periodSeconds: 5
        resources:
          requests:
            memory: "64Mi"
            cpu: "100m"
          limits:
            memory: "256Mi"
            cpu: "500m"
---
apiVersion: v1
kind: Service
metadata:
  name: asciidork-server
spec:
  selector:
    app: asciidork-server
  ports:
  - port: 80
    targetPort: 3000
  type: ClusterIP
```

## Security

### Safe Modes

The server supports four safe modes that control what operations are allowed:

| Mode | Description |
|------|-------------|
| `unsafe` | No restrictions. Allows file includes and external resources. **Must be explicitly enabled.** |
| `safe` | Minimal restrictions. |
| `server` | Server-appropriate restrictions. |
| `secure` | Maximum restrictions (default). No file access, no external resources. |

### Enabling Unsafe Mode

Unsafe mode must be explicitly enabled on the server:

```bash
ASCIIDORK_ALLOW_UNSAFE=true asciidork-server
```

Even when enabled, clients must explicitly request it per-request:

```json
{
  "content": "include::chapter1.adoc[]",
  "options": {
    "safe_mode": "unsafe"
  }
}
```

### CORS

Configure allowed origins for browser-based clients:

```bash
# Allow all origins (default, not recommended for production)
ASCIIDORK_CORS_ORIGINS=*

# Specific origins
ASCIIDORK_CORS_ORIGINS=https://example.com,https://app.example.com
```

### Request Limits

Protect against denial of service:

```bash
# Maximum content size (bytes)
ASCIIDORK_MAX_CONTENT_SIZE=10485760  # 10MB

# Request timeout (seconds)
ASCIIDORK_REQUEST_TIMEOUT_SECS=30
```

### Recommendations

1. **Never enable unsafe mode in production** unless absolutely necessary
2. Set specific CORS origins instead of `*`
3. Use appropriate request size limits
4. Run behind a reverse proxy (nginx, traefik) for TLS termination
5. Use rate limiting at the proxy level

## Output Formats

### dr-html (default)

Asciidoctor-compatible HTML output. Includes full CSS styling and produces output similar to the standard Asciidoctor toolchain.

### html5

Semantic HTML5 output based on the jirutka HTML5 backend. Produces cleaner, more semantic markup without embedded CSS.

### Prettier Variants

`dr-html-prettier` and `html5-prettier` formats run the output through prettier for formatted, readable HTML. Requires `prettier` to be installed and accessible.

```bash
# Install prettier
npm install -g prettier

# Or specify path
ASCIIDORK_PRETTIER_PATH=/usr/local/bin/prettier asciidork-server
```

## License

MIT OR Apache-2.0
