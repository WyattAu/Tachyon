# Multi-stage build for Tachyon Backend Server
# Optimized for small image size and fast builds

# Build stage
FROM rust:alpine AS builder

# Install build dependencies
RUN apk add --no-cache \
    musl-dev \
    g++ \
    pkgconfig \
    openssl-dev \
    postgresql-dev \
    git \
    patch \
    perl \
    make \
    cmake \
    curl

WORKDIR /app

# Copy workspace manifests first for dependency caching
COPY Cargo.toml Cargo.lock ./
COPY crates/core/Cargo.toml ./crates/core/
COPY crates/server/Cargo.toml ./crates/server/
COPY crates/database/Cargo.toml ./crates/database/
COPY crates/renderer/Cargo.toml ./crates/renderer/
COPY crates/rbac/Cargo.toml ./crates/rbac/
COPY crates/search/Cargo.toml ./crates/search/
COPY crates/desktop/Cargo.toml ./crates/desktop/
COPY crates/desktop/src-tauri/Cargo.toml ./crates/desktop/src-tauri/
COPY crates/testing/Cargo.toml ./crates/testing/
COPY crates/cli/Cargo.toml ./crates/cli/
COPY crates/frontend/Cargo.toml ./crates/frontend/

# Create dummy source files for ALL workspace members
# Must match the module structure expected by each crate's Cargo.toml
RUN mkdir -p crates/server/src && \
    echo "pub fn dummy() {}" > crates/server/src/lib.rs && \
    echo "fn main() {}" > crates/server/src/main.rs && \
    mkdir -p crates/core/src && \
    echo "pub fn dummy() {}" > crates/core/src/lib.rs && \
    mkdir -p crates/database/src && \
    echo "pub fn dummy() {}" > crates/database/src/lib.rs && \
    mkdir -p crates/database/migrations && \
    echo "" > crates/database/migrations/.gitkeep && \
    mkdir -p crates/renderer/src && \
    echo "pub fn dummy() {}" > crates/renderer/src/lib.rs && \
    mkdir -p crates/rbac/src && \
    echo "pub fn dummy() {}" > crates/rbac/src/lib.rs && \
    mkdir -p crates/search/src && \
    echo "pub fn dummy() {}" > crates/search/src/lib.rs && \
    mkdir -p crates/desktop/src && \
    echo "pub fn dummy() {}" > crates/desktop/src/lib.rs && \
    mkdir -p crates/desktop/src-tauri/src && \
    echo "pub fn dummy() {}" > crates/desktop/src-tauri/src/lib.rs && \
    echo "fn main() {}" > crates/desktop/src-tauri/src/main.rs && \
    mkdir -p crates/desktop/src-tauri/icons && \
    mkdir -p crates/desktop/src-tauri/capabilities && \
    echo "" > crates/desktop/src-tauri/tauri.conf.json && \
    mkdir -p crates/testing/src && \
    echo "pub fn dummy() {}" > crates/testing/src/lib.rs && \
    mkdir -p crates/testing/src/fuzz && \
    echo "fn main() {}" > crates/testing/src/fuzz/harness.rs && \
    mkdir -p crates/testing/benches && \
    echo "fn main() {}" > crates/testing/benches/repository_bench.rs && \
    echo "fn main() {}" > crates/testing/benches/search_bench.rs && \
    echo "fn main() {}" > crates/testing/benches/rbac_bench.rs && \
    echo "fn main() {}" > crates/testing/benches/database_bench.rs && \
    mkdir -p crates/cli/src && \
    echo "pub fn dummy() {}" > crates/cli/src/lib.rs && \
    mkdir -p crates/frontend/src && \
    echo "pub fn dummy() {}" > crates/frontend/src/lib.rs

# Build dependencies only (this layer is cached until Cargo.toml/lock changes)
# Use --lib to skip binary targets and avoid module resolution issues
RUN cargo build --release -p tachyon-server --lib

# Copy actual source code (invalidates cache only when source changes)
COPY crates ./crates

# Remove incremental compilation fingerprints for workspace crates
# so Cargo detects the source file changes
RUN find target/release/.fingerprint -name 'tachyon-*' -type d -exec rm -rf {} + 2>/dev/null || true

# Build the actual binary with real source code
RUN cargo build --release -p tachyon-server

# Runtime stage
FROM alpine:3.19

# Install runtime dependencies
RUN apk add --no-cache \
    ca-certificates \
    libgcc \
    postgresql-libs \
    && addgroup -g 1000 tachyon \
    && adduser -D -u 1000 -G tachyon tachyon

WORKDIR /app

# Copy the binary from builder
COPY --from=builder /app/target/release/tachyon-server /usr/local/bin/tachyon-server

# Copy migration files
COPY --chown=tachyon:tachyon crates/database/migrations /app/migrations

# Create data directory
RUN mkdir -p /app/data && chown tachyon:tachyon /app/data

USER tachyon

# Expose port
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://127.0.0.1:3000/health || exit 1

# Run the binary
ENTRYPOINT ["tachyon-server"]
