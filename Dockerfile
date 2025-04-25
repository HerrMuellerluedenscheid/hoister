FROM rust:alpine AS builder

WORKDIR /app
COPY . .
RUN apk add --no-cache musl-dev
RUN cargo build --release

FROM alpine:latest AS runner

LABEL maintainer="marius.kriegerowski@gmail.com"
LABEL description="Deploy and roll back docker images"

RUN apk update && \
    apk upgrade && \
    apk add --no-cache ca-certificates tzdata && \
    rm -rf /var/cache/apk/*

RUN addgroup -S appgroup && adduser -S appuser -G appgroup

RUN rm -rf /bin/ash /bin/sh /bin/bash /usr/bin/curl /usr/bin/wget
COPY --from=builder --chown=appuser:appgroup /app/target/release/deploya .

USER appuser
WORKDIR /app

CMD ["/app/deploya"]
