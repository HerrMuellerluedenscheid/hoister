FROM rust:alpine AS builder

WORKDIR /app
COPY . .

RUN apk add --no-cache \
    musl-dev \
    openssl-dev \
    openssl-libs-static \
    pkgconfig

ENV RUSTFLAGS="-C target-feature=+crt-static"

RUN cargo build --bin hoister --release --target aarch64-unknown-linux-musl

FROM scratch AS runner

ENV WATCH_INTERVAL=120
LABEL maintainer="marius.kriegerowski@gmail.com"
LABEL description="Deploy and roll back docker images"

WORKDIR /app
COPY --from=builder /app/target/aarch64-unknown-linux-musl/release/hoister .

CMD ["/app/hoister", "--watch"]