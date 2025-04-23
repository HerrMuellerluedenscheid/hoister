FROM alpine:latest
LABEL version="2"
RUN date +%Y%m%d > /build-timestamp
ENTRYPOINT ["top", "-b"]
