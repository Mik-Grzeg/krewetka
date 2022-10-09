FROM lukemathwalker/cargo-chef:latest-rust-1.63.0 AS chef
WORKDIR /app

FROM chef as planner
COPY krewetka-processor .
RUN cargo chef prepare --recipe-path recipe.json

####################################################################################################
## Builder 
####################################################################################################

FROM chef as builder 

# Create appuser
ENV USER=krewetka
ENV UID=10001

COPY krewetka-processor ./

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    "${USER}"

RUN apt-get update && apt-get install -y libzmq3-dev cmake protobuf-compiler libprotobuf-dev

WORKDIR /app
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

COPY krewetka-processor ./
COPY proto ./proto

RUN cargo build --release

####################################################################################################
## Final image processor
####################################################################################################
FROM debian:bookworm-slim as processor

# Import from builder.
COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group

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
COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group


WORKDIR /app

COPY --from=builder /app/src/migrator/clickhouse/migrations/ ./migrations

# Copy our build
COPY --from=builder /app/target/release/krewetka-migrator ./migrator

# Use an unprivileged user.
USER krewetka:krewetka

ENV RUST_LOG="debug"
CMD [ "/app/migrator", "apply", "./migrations" ]
