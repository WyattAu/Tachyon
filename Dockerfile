FROM rust:1.86-bookworm AS chef
RUN cargo install cargo-chef
WORKDIR /app

FROM chef AS planner
COPY tachyon/ .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

FROM rust:1.86-bookworm AS frontend
RUN rustup target add wasm32-unknown-unknown && \
    curl -sL https://github.com/trunk-rs/trunk/releases/download/v0.21.14/trunk-x86_64-unknown-linux-gnu.tar.gz | tar xz -C /usr/local/bin
WORKDIR /app
COPY tachyon/Cargo.toml tachyon/Cargo.lock ./
COPY tachyon/crates ./crates
COPY --from=builder /app/target /app/target
COPY --from=builder /usr/local/cargo /usr/local/cargo
WORKDIR /app/crates/frontend
RUN TRUNK_TOOLS_WASM_OPT= trunk build --release

FROM builder AS app-builder
COPY tachyon/ .
COPY --from=frontend /app/crates/frontend/dist ./crates/frontend/dist
RUN cargo build --release --bin tachyon-server

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
