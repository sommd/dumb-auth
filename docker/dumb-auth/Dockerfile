# syntax=docker/dockerfile:1

FROM gcr.io/distroless/cc-debian12

ARG TARGETARCH

COPY --from=binaries --chmod=755 dumb-auth-$TARGETARCH-gnu /usr/local/bin/dumb-auth

ENTRYPOINT ["/usr/local/bin/dumb-auth"]
