####################################################################################################
## Builder
####################################################################################################
FROM rust:1.76.0-alpine3.19 AS builder


RUN apk update
RUN apk add pkgconfig openssl openssl-dev musl-dev

RUN rustup target add aarch64-unknown-linux-musl
RUN rustup toolchain install stable-aarch64-unknown-linux-musl

RUN update-ca-certificates

# Create appuser
ENV USER=neuro-rs
ENV UID=10001

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    "${USER}"


WORKDIR /neuro-rs

COPY ./ .

RUN cargo build --target aarch64-unknown-linux-musl --release

####################################################################################################
## Intermediate stage to handle .env file
####################################################################################################
FROM alpine:3.19 AS intermediate

WORKDIR /neuro-rs

# Copy the binary from builder
COPY --from=builder /neuro-rs/target/aarch64-unknown-linux-musl/release/neuro-rs ./

# Attempt to copy the .env file. If it doesn't exist, create an empty one.
COPY .env .env || echo "" > .env

####################################################################################################
## Final image
####################################################################################################
FROM scratch

# Import from builder.
COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group

WORKDIR /neuro-rs

# Copy our build and potentially the .env file from the intermediate stage
COPY --from=intermediate /neuro-rs ./

# Use an unprivileged user.
USER neuro-rs:neuro-rs

CMD ["/neuro-rs/neuro-rs"]