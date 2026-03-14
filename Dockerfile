# Multi-stage Dockerfile for sigc

# Stage 1: Build
FROM rust:1.75-bookworm AS builder

WORKDIR /usr/src/sigc

# Install build dependencies
RUN apt-get update && apt-get install -y \
    cmake \
    pkg-config \
    libssl-dev \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy Cargo files first for dependency caching
COPY Cargo.toml Cargo.lock ./
COPY crates/sig_types/Cargo.toml crates/sig_types/
COPY crates/sig_cache/Cargo.toml crates/sig_cache/
COPY crates/sig_compiler/Cargo.toml crates/sig_compiler/
COPY crates/sig_runtime/Cargo.toml crates/sig_runtime/
COPY crates/sigc/Cargo.toml crates/sigc/
COPY crates/pysigc/Cargo.toml crates/pysigc/

# Create dummy source files to build dependencies
RUN mkdir -p crates/sig_types/src && echo "pub fn dummy() {}" > crates/sig_types/src/lib.rs && \
    mkdir -p crates/sig_cache/src && echo "pub fn dummy() {}" > crates/sig_cache/src/lib.rs && \
    mkdir -p crates/sig_compiler/src && echo "pub fn dummy() {}" > crates/sig_compiler/src/lib.rs && \
    mkdir -p crates/sig_runtime/src && echo "pub fn dummy() {}" > crates/sig_runtime/src/lib.rs && \
    mkdir -p crates/sigc/src && echo "fn main() {}" > crates/sigc/src/main.rs && \
    mkdir -p crates/pysigc/src && echo "pub fn dummy() {}" > crates/pysigc/src/lib.rs

# Build dependencies
RUN cargo build --release || true

# Copy actual source code
COPY crates ./crates
COPY examples ./examples

# Touch files to invalidate cache
RUN touch crates/*/src/*.rs

# Build the actual application
RUN cargo build --release --bin sigc

# Stage 2: Runtime
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libpq5 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 sigc

# Create directories
RUN mkdir -p /app/data /app/config /app/logs /app/cache && \
    chown -R sigc:sigc /app

WORKDIR /app

# Copy binary from builder
COPY --from=builder /usr/src/sigc/target/release/sigc /usr/local/bin/sigc

# Copy example files
COPY --from=builder /usr/src/sigc/examples ./examples

USER sigc

# Environment variables
ENV SIGC_DATA_DIR=/app/data \
    SIGC_CONFIG_DIR=/app/config \
    SIGC_LOG_DIR=/app/logs \
    SIGC_CACHE_DIR=/app/cache \
    RUST_LOG=info

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD sigc --version || exit 1

# Expose ports
EXPOSE 8080 9090

# Default command
ENTRYPOINT ["sigc"]
CMD ["--help"]
