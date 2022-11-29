# syntax=docker/dockerfile:1.4
FROM rust:1.65 as builder

ENV USER=krewetka
ENV UID=10001

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    "${USER}"

RUN apt-get update && apt-get install -y libzmq3-dev cmake protobuf-compiler libprotobuf-dev cmake

RUN cargo new /app
COPY Cargo.toml Cargo.lock /app/

# Compiles only our dependencies and saves them in a layer
WORKDIR /app

RUN --mount=type=cache,target=/usr/local/cargo/registry cargo build --release

# Copy source code
COPY . /app
ARG PROTO_PATH=/app/proto

RUN --mount=type=cache,target=/usr/local/cargo/registry --mount=type=cache,target=/app/target \
  set -e \
  touch ./src/main.rs && \
  cargo build --release --all && \
  mv /app/target/release/collector /app

FROM debian:bookworm-slim

COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group
USER krewetka:krewetka

COPY --from=builder /app/collector /collector

ENTRYPOINT [ "./collector" ]
