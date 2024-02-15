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
## Final image
####################################################################################################
FROM scratch

# Import from builder.
COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group

WORKDIR /neuro-rs

# Copy our build
COPY --from=builder /neuro-rs/target/aarch64-unknown-linux-musl/release/neuro-rs ./

# Use an unprivileged user.
USER neuro-rs:neuro-rs

CMD ["/neuro-rs/neuro-rs"]