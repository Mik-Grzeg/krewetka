FROM lukemathwalker/cargo-chef:latest-rust-1.63.0 AS chef
WORKDIR /app

FROM chef as planner
COPY ./krewetka-collector .
RUN cargo chef prepare --recipe-path recipe.json

####################################################################################################
## Builder 
####################################################################################################

FROM chef as builder 

# Create appuser
ENV USER=krewetka
ENV UID=10001

COPY ./krewetka-collector ./

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    "${USER}"

RUN apt-get update && apt-get install -y libzmq3-dev cmake protobuf-compiler libprotobuf-dev cmake

WORKDIR /app
COPY --from=planner /app/recipe.json recipe.json

RUN cargo chef cook --release --recipe-path recipe.json

COPY ./krewetka-collector ./
COPY proto ./proto

RUN cargo build --release

####################################################################################################
## Final image collector
####################################################################################################
FROM debian:bookworm-slim as collector

# Import from builder.
COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group

# to remove
RUN apt-get update && apt-get install -yy curl
#

WORKDIR /app

# Copy our build
COPY --from=builder /app/target/release/krewetka-collector ./collector

# Use an unprivileged user.
USER krewetka:krewetka

ENV RUST_LOG="debug"
CMD [ "/app/collector" ]
