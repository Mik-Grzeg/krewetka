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

RUN apt-get update && apt-get install -y libzmq3-dev cmake protobuf-compiler libprotobuf-dev

RUN cargo new /app
COPY Cargo.toml Cargo.lock /app/

# Compiles only our dependencies and saves them in a layer
WORKDIR /app

RUN mkdir ./src/bin && \
	cp ./src/main.rs ./src/bin/migrator.rs && \
	cp ./src/main.rs ./src/bin/processor.rs && \
	touch ./src/lib.rs

RUN --mount=type=cache,target=/usr/local/cargo/registry cargo build --release --all

# Copy source code
COPY . /app
ARG PROTO_PATH=/app/proto

RUN --mount=type=cache,target=/usr/local/cargo/registry --mount=type=cache,target=/app/target \
  set -e \
  touch ./src/lib.rs && \
  touch ./src/bin/migrator.rs && \
  touch ./src/bin/processor.rs && \
  cargo build --release --all && \
  mv /app/target/release/processor /app && \
  mv /app/target/release/migrator /app

FROM debian:bookworm-slim

COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group
USER krewetka:krewetka

COPY --from=builder /app/processor /processor
COPY --from=builder /app/migrator /processor-migrator
COPY --from=builder /app/src/migrator/clickhouse/migrations/ /app/migrations

ENTRYPOINT [ "./processor" ]
