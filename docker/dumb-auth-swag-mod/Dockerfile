# syntax=docker/dockerfile:1

FROM scratch AS build

ARG TARGETARCH

COPY root/ /
COPY --from=binaries --chmod=755 dumb-auth-$TARGETARCH-musl /usr/local/bin/dumb-auth

FROM scratch

COPY --from=build / /
