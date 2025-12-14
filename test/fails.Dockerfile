FROM alpine:3.19 as runner
LABEL version="should-error"

RUN date +%Y%m%d > /build-timestamp
ARG  ERROR_CODE=1

VOLUME "/data"

ENV CONTAINER_ERROR_CODE=$ERROR_CODE
ENTRYPOINT ["sh", "-c", "rm -r /data/*; echo \"Intentionally exiting with error code $CONTAINER_ERROR_CODE\"; exit $CONTAINER_ERROR_CODE"]
