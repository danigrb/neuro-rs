####################################################################################################
## Builder
####################################################################################################
FROM rust:latest AS builder

RUN rustup target add aarch64-unknown-linux-musl
RUN apt update && apt install -y  musl-dev openssl-dev openssl-libs-static
RUN update-ca-certificates

# Create appuser
ENV OPENSSL_STATIC=1
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