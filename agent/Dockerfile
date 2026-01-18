FROM rust:alpine AS builder

WORKDIR /app
COPY . .
RUN apk add --no-cache musl-dev
RUN cargo build --bin hoister --release

FROM busybox:latest as shell-builder
FROM scratch AS runner

ENV HOISTER_SCHEDULE_INTERVAL=120
ENV CONFIG=/config.toml

LABEL maintainer="marius.kriegerowski@gmail.com"
LABEL description="Deploy and roll back docker images"
LABEL io.hoister.container="agent"

WORKDIR /app
COPY --from=shell-builder /bin/sh /bin/sh
COPY --from=shell-builder /bin/cp /bin/cp
COPY --from=builder /app/target/release/hoister .

CMD ["hoister"]
