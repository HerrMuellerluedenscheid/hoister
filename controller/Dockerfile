FROM rust:alpine AS builder

WORKDIR /app
COPY . .
RUN apk add --no-cache musl-dev
RUN cargo build --bin controller --release

FROM alpine AS runner

LABEL maintainer="marius.kriegerowski@gmail.com"
LABEL description="Deploy and roll back docker images (controller)"

WORKDIR /app
COPY --from=builder /app/target/release/controller .

CMD ["/app/controller"]
