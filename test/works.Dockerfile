FROM alpine:latest
LABEL version="2"
LABEL version="should-work"

RUN date +%Y%m%d > /build-timestamp

ENTRYPOINT ["top", "-b"]