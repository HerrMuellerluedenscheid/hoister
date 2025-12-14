FROM alpine:latest
LABEL version="2"
LABEL version="should-work"
VOLUME "/data"

RUN date +%Y%m%d%H%M%S > /build-timestamp

ENTRYPOINT ["sh", "-c", "date +%Y%m%d%H%M%S >> /data/timestamps.txt; top -b"]
