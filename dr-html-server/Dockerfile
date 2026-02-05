# =============================================================================
# Asciidork Server Dockerfile
# =============================================================================
# Multi-stage build for minimal final image size
#
# Build: docker build -t asciidork-server .
# Run:   docker run -p 3000:3000 asciidork-server
# =============================================================================

# -----------------------------------------------------------------------------
# Stage 1: Build
# -----------------------------------------------------------------------------
FROM rust:1.93-slim-trixie AS builder

WORKDIR /build

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace files (all members required for Cargo to parse workspace)
COPY Cargo.toml Cargo.lock ./
COPY ast ./ast
COPY backend ./backend
COPY backend-html5s ./backend-html5s
COPY cli ./cli
COPY core ./core
COPY dr-html-backend ./dr-html-backend
COPY dr-html-server ./dr-html-server
COPY dr-html-wasm ./dr-html-wasm
COPY eval ./eval
COPY parser ./parser
COPY tck ./tck
COPY test-utils ./test-utils

# Build release binary
RUN cargo build --release -p asciidork-server

# Strip debug symbols for smaller binary
RUN strip /build/target/release/asciidork-server

# -----------------------------------------------------------------------------
# Stage 2: Runtime
# -----------------------------------------------------------------------------
FROM debian:trixie-slim AS runtime

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -r -s /bin/false -u 1000 asciidork

# Copy binary from builder
COPY --from=builder /build/target/release/asciidork-server /usr/local/bin/asciidork-server

# Set ownership
RUN chown asciidork:asciidork /usr/local/bin/asciidork-server

# Switch to non-root user
USER asciidork

# Default environment
ENV ASCIIDORK_HOST=0.0.0.0
ENV ASCIIDORK_PORT=3000
ENV ASCIIDORK_LOG_LEVEL=info

# Expose port
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/api/v1/health || exit 1

# Run server
ENTRYPOINT ["asciidork-server"]

