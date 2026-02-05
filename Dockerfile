# Build stage
FROM rust:1.93-slim-bookworm AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy the entire workspace
COPY . .

# Build the CLI in release mode
RUN cargo build --release --package asciidork-cli

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies (SSL for https support in minreq)
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy the built binary from the builder stage
COPY --from=builder /app/target/release/asciidork /usr/local/bin/asciidork

# Set the entrypoint
ENTRYPOINT ["asciidork"]
