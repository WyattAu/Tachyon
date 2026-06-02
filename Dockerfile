# syntax=docker/dockerfile:1

# NOTE on libquickjs-sys: build.rs panics if `patch` binary is missing (installed above).
# NOTE on tree-sitter: feature-gated behind `native-tree-sitter` in tachyon-editor.
# Frontend compiles to wasm32-unknown-unknown without tree-sitter (regex-only highlighting).

# Builder uses debian:bookworm (permitted for build stages, discarded in final)
# Runtime uses scratch (static musl binary, zero runtime deps)

FROM debian:bookworm AS base
ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update && apt-get install -y --no-install-recommends \
    curl ca-certificates pkg-config musl-tools make perl patch gcc libc6-dev \
    && rm -rf /var/lib/apt/lists/*
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable --profile minimal
ENV PATH="/root/.cargo/bin:${PATH}"

# Pre-create passwd file for scratch runtime (no shell available there)
RUN echo "tachyon:x:65532:65532:tachyon:/app:/sbin/nologin" > /etc/passwd.tachyon

# --- Stage 1: Server binary (musl static) ---
FROM base AS server-builder
COPY tachyon/ /build/tachyon/
WORKDIR /build/tachyon

RUN rustup target add x86_64-unknown-linux-musl

# Dependency cache layer: build with stub crate sources first
RUN sed -i 's/, "crates\/desktop"//g; s/, "crates\/desktop\/src-tauri"//g; s/, "crates\/testing"//g; s/, "crates\/cli"//g; s/, "crates\/benchmarks"//g; s/, "crates\/frontend"//g' Cargo.toml && \
    mkdir -p crates && \
    for crate in core database editor import_export plugin-runtime rbac renderer search server ssg storage; do \
        mkdir -p crates/$crate/src && echo "" > crates/$crate/src/lib.rs; \
    done && \
    cargo build --release --bin tachyon-server --target x86_64-unknown-linux-musl 2>/dev/null || true && \
    rm -rf crates

# Full build (server-only, no frontend)
RUN sed -i 's/, "crates\/desktop"//g; s/, "crates\/desktop\/src-tauri"//g; s/, "crates\/testing"//g; s/, "crates\/cli"//g; s/, "crates\/benchmarks"//g; s/, "crates\/frontend"//g' Cargo.toml && \
    CC_x86_64_unknown_linux_musl=musl-gcc \
    CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER=musl-gcc \
    cargo build --release --bin tachyon-server --target x86_64-unknown-linux-musl && \
    cp target/x86_64-unknown-linux-musl/release/tachyon-server /build/tachyon-server

# --- Stage 2: Frontend WASM (trunk) ---
FROM base AS frontend-builder
RUN rustup target add wasm32-unknown-unknown \
    && cargo install trunk --locked --version 0.21.6

COPY tachyon/ /build/tachyon/
WORKDIR /build/tachyon/crates/frontend
RUN trunk build --release

# --- Stage 3: Scratch runtime ---
FROM scratch

LABEL org.opencontainers.image.source="https://github.com/WyattAu/Tachyon" \
      org.opencontainers.image.title="Tachyon" \
      org.opencontainers.image.description="Self-hosted collaborative knowledge management" \
      org.opencontainers.image.vendor="WyattAu"

WORKDIR /app

COPY --from=server-builder /build/tachyon-server /app/tachyon-server
COPY --from=server-builder /build/tachyon/crates/database/migrations /app/migrations

COPY --from=frontend-builder /build/tachyon/crates/frontend/dist/ /app/dist/

COPY --from=server-builder /etc/passwd.tachyon /etc/passwd

HEALTHCHECK NONE

USER 65532:65532

ENV TACHYON_STATIC_DIR=/app/dist
ENV TACHYON_MIGRATIONS_DIR=/app/migrations

EXPOSE 8080

ENTRYPOINT ["/app/tachyon-server"]

STOPSIGNAL SIGTERM
