####################################################################################################
## Builder
####################################################################################################
FROM rust:latest AS builder
RUN set -x && \
    apk add --no-cache musl-dev openssl-dev openssl-libs-static

# statically link against openssl
ENV OPENSSL_STATIC=1 
RUN rustup target add aarch64-unknown-linux-musl
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