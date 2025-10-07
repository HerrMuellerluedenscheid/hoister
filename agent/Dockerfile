FROM rust:alpine AS builder


WORKDIR /app
COPY . .
RUN apk add --no-cache musl-dev
RUN cargo build --bin hoister --release

FROM scratch AS runner

ENV WATCH_INTERVAL=120
LABEL maintainer="marius.kriegerowski@gmail.com"
LABEL description="Deploy and roll back docker images"

WORKDIR /app
COPY --from=builder /app/target/release/hoister .

CMD ["/app/hoister", "--watch"]