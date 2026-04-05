# Tachyon Server Dockerfile
# Multi-stage build for optimized production image

# Stage 1: Build environment
FROM rust:1.75-slim-bookworm AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    cmake \
    clang \
    llvm \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy manifests first for better caching
COPY Cargo.toml Cargo.lock ./
COPY crates/*/Cargo.toml ./crates/

# Copy source code
COPY . .

# Build release binary
RUN cargo build --release --workspace --exclude tachyon-testing

# Stage 2: Runtime environment
FROM debian:bookworm-slim AS runtime

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    sqlite3 \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Create non-root user
RUN groupadd -r tachyon && useradd -r -g tachyon tachyon

# Create app directories
RUN mkdir -p /app/data /app/config /app/logs && \
    chown -R tachyon:tachyon /app

# Copy binary from builder
COPY --from=builder /app/target/release/tachyon-server /app/
COPY --from=builder /app/target/release/tachyon /app/ 2>/dev/null || true

# Copy default config if exists
COPY --from=builder /app/config/production.toml /app/config/ 2>/dev/null || true

# Set ownership
RUN chown -R tachyon:tachyon /app

# Switch to non-root user
USER tachyon

# Set working directory
WORKDIR /app

# Environment variables
ENV RUST_LOG=info
ENV DATABASE_PATH=/app/data/tachyon.db
ENV SERVER_HOST=0.0.0.0
ENV SERVER_PORT=8080

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:${SERVER_PORT}/health || exit 1

# Expose port
EXPOSE 8080

# Run server
CMD ["./tachyon-server"]
