# Stage 1: Build dependencies (cache layer)
FROM rust:bookworm AS builder
RUN rustup target add wasm32-unknown-unknown && \
    curl -sL https://github.com/trunk-rs/trunk/releases/download/v0.21.14/trunk-x86_64-unknown-linux-gnu.tar.gz | tar xz -C /usr/local/bin
WORKDIR /app

# Copy manifests first for dependency caching
COPY tachyon/Cargo.toml tachyon/Cargo.lock ./
# Remove desktop/testing/cli/benchmarks from workspace members (require GTK/tauri)
RUN sed -i 's/, "crates\/desktop"//g; s/, "crates\/desktop\/src-tauri"//g; s/, "crates\/testing"//g; s/, "crates\/cli"//g; s/, "crates\/benchmarks"//g' Cargo.toml
RUN mkdir -p crates && \
    for crate in core database editor import_export plugin-runtime rbac renderer search server ssg storage; do \
        mkdir -p crates/$crate/src && echo "" > crates/$crate/src/lib.rs; \
    done && \
    cargo build --release 2>/dev/null || true && \
    rm -rf crates

# Stage 2: Build frontend
FROM builder AS frontend
COPY tachyon/ .
# Re-apply workspace member removal (COPY overwrites our modified Cargo.toml)
RUN sed -i 's/, "crates\/desktop"//g; s/, "crates\/desktop\/src-tauri"//g; s/, "crates\/testing"//g; s/, "crates\/cli"//g; s/, "crates\/benchmarks"//g' Cargo.toml
RUN mkdir -p crates/frontend/dist
WORKDIR /app/crates/frontend
RUN trunk build --release

# Stage 3: Build server
FROM builder AS app-builder
COPY tachyon/ .
# Re-apply workspace member removal
RUN sed -i 's/, "crates\/desktop"//g; s/, "crates\/desktop\/src-tauri"//g; s/, "crates\/testing"//g; s/, "crates\/cli"//g; s/, "crates\/benchmarks"//g' Cargo.toml
COPY --from=frontend /app/crates/frontend/dist ./crates/frontend/dist
RUN cargo build --release --bin tachyon-server

# Stage 4: Runtime
FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y \
    ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app

COPY --from=app-builder /app/target/release/tachyon-server /app/tachyon-server
COPY --from=app-builder /app/crates/database/migrations /app/migrations
COPY --from=frontend /app/crates/frontend/dist /app/dist

RUN groupadd -r tachyon && useradd -r -g tachyon -d /app tachyon && \
    chown -R tachyon:tachyon /app

USER tachyon

EXPOSE 8080
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD ["/app/tachyon-server", "health"] || exit 1

ENV RUST_LOG=info
ENV DATABASE_URL=postgres://tachyon:tachyon@db:5432/tachyon
ENV JWT_SECRET=change-me-in-production
ENTRYPOINT ["/app/tachyon-server"]
