# Tachyon: inner workspace build
# Builder uses debian:bookworm (permitted for build stages)
# Runtime uses scratch (static musl binary, zero runtime deps)
#
# musl-tools + CC target override for vendored C deps (libgit2/openssl).
# gcc + libc6-dev required for host build script linking (proc-macro crt files).

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

# Dependency cache layer: copy all workspace manifests + create dummy sources
COPY Cargo.toml Cargo.lock ./
COPY crates/core/Cargo.toml ./crates/core/
COPY crates/server/Cargo.toml ./crates/server/
COPY crates/database/Cargo.toml ./crates/database/
COPY crates/renderer/Cargo.toml ./crates/renderer/
COPY crates/rbac/Cargo.toml ./crates/rbac/
COPY crates/search/Cargo.toml ./crates/search/
COPY crates/storage/Cargo.toml ./crates/storage/
COPY crates/import-export/Cargo.toml ./crates/import-export/
COPY crates/ssg/Cargo.toml ./crates/ssg/
COPY crates/plugin-runtime/Cargo.toml ./crates/plugin-runtime/
COPY crates/editor/Cargo.toml ./crates/editor/
COPY crates/frontend/Cargo.toml ./crates/frontend/

# Remove non-server workspace members (desktop/testing/cli/benchmarks require GTK/tauri)
RUN sed -i 's/, "crates\/desktop"//g; s/, "crates\/desktop\/src-tauri"//g; s/, "crates\/testing"//g; s/, "crates\/cli"//g; s/, "crates\/benchmarks"//g' Cargo.toml && \
    mkdir -p crates/server/src && echo "pub fn dummy() {}" > crates/server/src/lib.rs && \
    echo "fn main() {}" > crates/server/src/main.rs && \
    for crate in core database renderer rbac search storage import-export ssg plugin-runtime editor frontend; do \
        mkdir -p crates/$crate/src && echo "pub fn dummy() {}" > crates/$crate/src/lib.rs; \
    done && \
    mkdir -p crates/database/migrations && echo "" > crates/database/migrations/.gitkeep && \
    mkdir -p crates/frontend/src && \
    cargo build --release --target x86_64-unknown-linux-musl -p tachyon-server --lib 2>/dev/null || true && \
    rm -rf crates

# Full build
COPY crates ./crates
RUN find target/x86_64-unknown-linux-musl/release/.fingerprint -name 'tachyon-*' -type d -exec rm -rf {} + 2>/dev/null || true
RUN cargo build --release --target x86_64-unknown-linux-musl -p tachyon-server

FROM scratch

LABEL org.opencontainers.image.source="https://github.com/WyattAu/Tachyon" \
      org.opencontainers.image.description="Tachyon server (inner workspace build)" \
      org.opencontainers.image.title="tachyon-server" \
      org.opencontainers.image.vendor="WyattAu"

EXPOSE 8080

WORKDIR /app

COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/tachyon-server /usr/local/bin/tachyon-server
COPY crates/database/migrations /app/migrations

# /etc/passwd for USER directive (file created in builder)
COPY --from=builder /etc/passwd.tachyon /etc/passwd

HEALTHCHECK NONE

USER 65532:65532

ENTRYPOINT ["tachyon-server"]

STOPSIGNAL SIGTERM
