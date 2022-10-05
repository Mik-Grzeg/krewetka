FROM lukemathwalker/cargo-chef:latest-rust-1.63.0 AS chef
WORKDIR /app

FROM chef as planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

####################################################################################################
## Cacher
####################################################################################################

FROM chef as cacher

# Create appuser
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

RUN apt-get update && apt-get install -y libzmq3-dev cmake

WORKDIR /app
COPY --from=planner /app/recipe.json recipe.json
# Build the dependencies (and add to docker's caching layer)
# This caches the dependency files similar to how @ckaserer's solution
# does, but is handled solely through the `cargo-chef` library.
RUN cargo chef cook --release --recipe-path recipe.json

####################################################################################################
## Builder
####################################################################################################

FROM rust as builder
WORKDIR /app

RUN apt-get update && apt-get install -y protobuf-compiler libprotobuf-dev cmake

COPY . .
COPY --from=cacher /app/target target

RUN cargo build --release

####################################################################################################
## Final image processor
####################################################################################################
FROM debian:bookworm-slim as processor

# Import from builder.
COPY --from=cacher /etc/passwd /etc/passwd
COPY --from=cacher /etc/group /etc/group

WORKDIR /app

# Copy our build
COPY --from=builder /app/target/release/krewetka-processor ./processor

# Use an unprivileged user.
USER krewetka:krewetka

ENV RUST_LOG="debug"
CMD [ "/app/processor" ]

####################################################################################################
## Final image migrator 
####################################################################################################
FROM debian:bookworm-slim as migrator 

# Import from builder.
COPY --from=cacher /etc/passwd /etc/passwd
COPY --from=cacher /etc/group /etc/group


WORKDIR /app

COPY --from=builder /app/src/migrator/clickhouse/migrations/ ./migrations

# Copy our build
COPY --from=builder /app/target/release/krewetka-migrator ./migrator

# Use an unprivileged user.
USER krewetka:krewetka

ENV RUST_LOG="debug"
CMD [ "/app/migrator", "apply", "./migrations" ]