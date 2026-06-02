# Tachyon: server-only build (same as Dockerfile.server, kept for docker compose)
#
# NOTE: WASM frontend build is not yet supported in this Dockerfile.
# The frontend (crates/frontend) depends on tachyon-editor which uses
# tree-sitter -- a native-only dependency that won't compile to
# wasm32-unknown-unknown. Frontend must be built separately or served
# via CDN. The server embeds frontend dist via rust-embed if available.
#
# Builder uses debian:bookworm (permitted for build stages, discarded in final)
# Runtime uses scratch (static musl binary, zero runtime deps)

FROM debian:bookworm AS builder
RUN apt-get update && apt-get install -y --no-install-recommends \
    curl ca-certificates pkg-config musl-tools make perl patch gcc libc6-dev \
    && rm -rf /var/lib/apt/lists/*
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable --profile minimal && \
    /root/.cargo/bin/rustup target add x86_64-unknown-linux-musl
ENV PATH="/root/.cargo/bin:${PATH}"
ENV CC_x86_64_unknown_linux_musl=musl-gcc

# Pre-create passwd file for scratch runtime (no shell available there)
RUN echo "tachyon:x:65532:65532:tachyon:/app:/sbin/nologin" > /etc/passwd.tachyon

WORKDIR /app

# Dependency cache layer
COPY tachyon/Cargo.toml tachyon/Cargo.lock ./
RUN sed -i 's/, "crates\/desktop"//g; s/, "crates\/desktop\/src-tauri"//g; s/, "crates\/testing"//g; s/, "crates\/cli"//g; s/, "crates\/benchmarks"//g; s/, "crates\/frontend"//g' Cargo.toml && \
    mkdir -p crates && \
    for crate in core database editor import_export plugin-runtime rbac renderer search server ssg storage; do \
        mkdir -p crates/$crate/src && echo "" > crates/$crate/src/lib.rs; \
    done && \
    cargo build --release --bin tachyon-server --target x86_64-unknown-linux-musl 2>/dev/null || true && \
    rm -rf crates

# Full build (server-only, no frontend)
COPY tachyon/ .
RUN sed -i 's/, "crates\/desktop"//g; s/, "crates\/desktop\/src-tauri"//g; s/, "crates\/testing"//g; s/, "crates\/cli"//g; s/, "crates\/benchmarks"//g; s/, "crates\/frontend"//g' Cargo.toml && \
    cargo build --release --bin tachyon-server --target x86_64-unknown-linux-musl

FROM scratch

LABEL org.opencontainers.image.source="https://github.com/WyattAu/Tachyon" \
      org.opencontainers.image.description="Tachyon knowledge management system" \
      org.opencontainers.image.title="tachyon" \
      org.opencontainers.image.vendor="WyattAu"

EXPOSE 8080

WORKDIR /app

COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/tachyon-server /app/tachyon-server
COPY tachyon/crates/database/migrations /app/migrations

COPY --from=builder /etc/passwd.tachyon /etc/passwd

HEALTHCHECK NONE

USER 65532:65532

ENTRYPOINT ["/app/tachyon-server"]

STOPSIGNAL SIGTERM
