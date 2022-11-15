FROM lukemathwalker/cargo-chef:latest-rust-1.63.0 AS chef
WORKDIR /app

FROM chef as planner
COPY reporting .
RUN cargo chef prepare --recipe-path recipe.json

####################################################################################################
## Builder
####################################################################################################

FROM chef as builder

# Create appuser
ENV USER=krewetka
ENV UID=10001

COPY reporting ./

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    "${USER}"

WORKDIR /app
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

RUN cargo build --release

####################################################################################################
## Final image reporting
####################################################################################################
FROM debian:bookworm as reporting

# Import from builder.
COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group

WORKDIR /app

# Copy our build
COPY --from=builder /app/target/release/reporting ./reporting

# Use an unprivileged user.
USER krewetka:krewetka

ENV RUST_LOG="debug"
CMD [ "/app/reporting" ]

