FROM alpine:3.19
LABEL version="should-error"

RUN date +%Y%m%d > /build-timestamp
ARG  ERROR_CODE=1

ENV CONTAINER_ERROR_CODE=$ERROR_CODE
ENTRYPOINT ["sh", "-c", "echo \"Intentionally exiting with error code $CONTAINER_ERROR_CODE\"; exit $CONTAINER_ERROR_CODE"]
