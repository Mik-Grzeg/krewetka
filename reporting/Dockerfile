# syntax=docker/dockerfile:1.4

FROM rust:1.65 as builder

# Capture dependencies
COPY Cargo.toml Cargo.lock /app/

RUN cargo new /app/reporting
COPY ./Cargo.toml /app/reporting

# Compiles only our dependencies and saves them in a layer
WORKDIR /app/reporting
RUN --mount=type=cache,target=/usr/local/cargo/registry cargo build --release

# Copy source code
COPY . /app

RUN --mount=type=cache,target=/usr/local/cargo/registry --mount=type=cache,target=/app/target \
  set -e \
  cargo build --release
  # mv /app/reporting/target/release/reporting /app

FROM debian:buster-slim as app
COPY --from=builder /app/reporting/target/release/reporting /reporting
CMD ["./reporting"]



