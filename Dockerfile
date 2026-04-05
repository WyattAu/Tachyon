FROM rust:1.75-alpine AS planner
WORKDIR /app
RUN apk add --no-cache musl-dev
RUN cargo install cargo-chef
COPY tachyon/Cargo.toml tachyon/Cargo.lock ./
COPY tachyon/crates crates/
RUN cargo chef prepare --recipe-path recipe.json

FROM rust:1.75-alpine AS cacher
WORKDIR /app
RUN apk add --no-cache musl-dev openssl-dev openssl-libs-static pkgconfig cmake
RUN cargo install cargo-chef
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

FROM rust:1.75-alpine AS builder
WORKDIR /app
RUN apk add --no-cache musl-dev openssl-dev openssl-libs-static pkgconfig cmake
COPY tachyon/Cargo.toml tachyon/Cargo.lock ./
COPY tachyon/crates crates/
COPY --from=cacher /app/target target
COPY --from=cacher /usr/local/cargo /usr/local/cargo
RUN cargo build --release -p tachyon-server --locked

FROM gcr.io/distroless/static-debian12:latest AS runtime
COPY --from=builder /app/target/release/tachyon-server /tachyon-server
USER nonroot:nonroot
EXPOSE 8080
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD ["/tachyon-server", "health"] || exit 1
ENV RUST_LOG=info
ENV SERVER_HOST=0.0.0.0
ENV SERVER_PORT=8080
ENTRYPOINT ["/tachyon-server"]
