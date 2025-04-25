FROM rust:alpine AS builder

WORKDIR /app
COPY . .
RUN apk add --no-cache musl-dev
RUN cargo build --release

FROM alpine:latest AS runner

ENV WATCH_INTERVAL=60
LABEL maintainer="marius.kriegerowski@gmail.com"
LABEL description="Deploy and roll back docker images"

RUN apk update && \
    apk upgrade && \
    apk add --no-cache ca-certificates tzdata && \
    rm -rf /var/cache/apk/*


RUN rm -rf /bin/ash /bin/bash /usr/bin/curl /usr/bin/wget
WORKDIR /app
COPY --from=builder /app/target/release/deploya .

CMD ["sh", "-c", "/app/deploya --watch 60"]
