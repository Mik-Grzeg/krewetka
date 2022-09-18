FROM lukemathwalker/cargo-chef:latest-rust-1.63.0 AS chef
WORKDIR /app

FROM chef as planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

####################################################################################################
## Builder
####################################################################################################

FROM chef as builder

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

RUN apt-get update && apt-get install -y libzmq3-dev cmake protobuf-compiler libprotobuf-dev

WORKDIR /app
COPY --from=planner /app/recipe.json recipe.json
# Build the dependencies (and add to docker's caching layer)
# This caches the dependency files similar to how @ckaserer's solution
# does, but is handled solely through the `cargo-chef` library.
RUN cargo chef cook --release --recipe-path recipe.json
# Build the application
COPY . .

# We no longer need to use the x86_64-unknown-linux-musl target
RUN cargo build


####################################################################################################
## Final image processor 
####################################################################################################
FROM debian:bookworm-slim as processor 

# Import from builder.
COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group

WORKDIR /app

# Copy our build
COPY --from=builder /app/target/debug/krewetka-processor ./processor

# Use an unprivileged user.
USER krewetka:krewetka

ENV RUST_LOG="debug"
CMD [ "/app/processor" ]