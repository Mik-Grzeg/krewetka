# syntax=docker/dockerfile:1.4

FROM rust:1.65 as builder

RUN cargo new /app
COPY Cargo.toml Cargo.lock /app/

# Compiles only our dependencies and saves them in a layer
WORKDIR /app
RUN --mount=type=cache,target=/usr/local/cargo/registry cargo build --release

# Copy source code
COPY . /app

RUN --mount=type=cache,target=/usr/local/cargo/registry --mount=type=cache,target=/app/target \
  set -e \
  touch src/main.rs && \
  cargo build --release && \
  mv /app/target/release/reporting /app

FROM debian:buster-slim as app
COPY --from=builder /app/reporting /reporting
CMD ["./reporting"]



