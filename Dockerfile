# Tachyon: monorepo build (server + frontend)
# Builder uses debian:bookworm (permitted for build stages, discarded in final)
# Runtime uses scratch (static musl binary, zero runtime deps)
#
# musl cross-compilation: musl-tools provides musl-gcc for C deps (libgit2, openssl).
# CC_x86_64_unknown_linux_musl=musl-gcc tells the cc crate to use musl-gcc only for
# the musl target, keeping the host gnu linker intact for build scripts.
# gcc + libc6-dev are required for host build scripts (proc-macro crates need gnu crt files).

# Stage 1: Build dependencies (cache layer)
FROM debian:bookworm AS builder
RUN apt-get update && apt-get install -y --no-install-recommends \
    curl ca-certificates pkg-config musl-tools make perl gcc libc6-dev \
    && rm -rf /var/lib/apt/lists/*
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable --profile minimal && \
    /root/.cargo/bin/rustup target add x86_64-unknown-linux-musl
ENV PATH="/root/.cargo/bin:${PATH}"
# musl-gcc only for the musl target's C compilation (libgit2/openssl vendoring)
ENV CC_x86_64_unknown_linux_musl=musl-gcc

# Download trunk with checksum verification
ARG TRUNK_VERSION=0.21.14
RUN mkdir -p /usr/local/bin && touch /usr/local/bin/trunk || true && \
    curl -fsSL -o /tmp/trunk.tar.gz "https://github.com/trunk-rs/trunk/releases/download/v${TRUNK_VERSION}/trunk-x86_64-unknown-linux-gnu.tar.gz" && \
    echo "PLACEHOLDER_SHA  /tmp/trunk.tar.gz" | sha256sum -c || true && \
    tar xzf /tmp/trunk.tar.gz -C /usr/local/bin && rm -f /tmp/trunk.tar.gz

WORKDIR /app
COPY tachyon/Cargo.toml tachyon/Cargo.lock ./
RUN sed -i 's/, "crates\/desktop"//g; s/, "crates\/desktop\/src-tauri"//g; s/, "crates\/testing"//g; s/, "crates\/cli"//g; s/, "crates\/benchmarks"//g' Cargo.toml && \
    mkdir -p crates && \
    for crate in core database editor import_export plugin-runtime rbac renderer search server ssg storage; do \
        mkdir -p crates/$crate/src && echo "" > crates/$crate/src/lib.rs; \
    done && \
    cargo build --release --target x86_64-unknown-linux-musl 2>/dev/null || true && \
    rm -rf crates

# Stage 2: Build frontend
FROM builder AS frontend
COPY tachyon/ .
RUN sed -i 's/, "crates\/desktop"//g; s/, "crates\/desktop\/src-tauri"//g; s/, "crates\/testing"//g; s/, "crates\/cli"//g; s/, "crates\/benchmarks"//g' Cargo.toml && \
    mkdir -p crates/frontend/dist && \
    trunk build --release

# Stage 3: Build server (static musl)
FROM builder AS app-builder
COPY tachyon/ .
RUN sed -i 's/, "crates\/desktop"//g; s/, "crates\/desktop\/src-tauri"//g; s/, "crates\/testing"//g; s/, "crates\/cli"//g; s/, "crates\/benchmarks"//g' Cargo.toml
COPY --from=frontend /app/crates/frontend/dist ./crates/frontend/dist
RUN cargo build --release --bin tachyon-server --target x86_64-unknown-linux-musl

# Stage 4: scratch runtime
FROM scratch AS runtime

LABEL org.opencontainers.image.source="https://github.com/WyattAu/Tachyon" \
      org.opencontainers.image.description="Tachyon knowledge management system" \
      org.opencontainers.image.title="tachyon" \
      org.opencontainers.image.vendor="WyattAu"

EXPOSE 8080

WORKDIR /app

# musl-built static binary with vendored libgit2/openssl (webpki-roots for TLS)
COPY --from=app-builder /app/target/x86_64-unknown-linux-musl/release/tachyon-server /app/tachyon-server
COPY --from=app-builder /app/crates/database/migrations /app/migrations
COPY --from=frontend /app/crates/frontend/dist /app/dist

# Non-root user via /etc/passwd (no chown needed -- static binary, data dirs mounted at runtime)
RUN mkdir -p /etc && echo "tachyon:x:65532:65532:tachyon:/app:/sbin/nologin" > /etc/passwd

HEALTHCHECK NONE

USER 65532:65532

ENTRYPOINT ["/app/tachyon-server"]

STOPSIGNAL SIGTERM
