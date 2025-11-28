FROM rust:alpine3.22

RUN apk add --no-cache \
    build-base \
    musl-dev \
    linux-headers \
    pkgconf \
    bash

RUN addgroup -S vscode && adduser -S -G vscode -s /bin/bash vscode

USER vscode

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD /bin/sh -c "rustc --version >/dev/null 2>&1 || exit 1"

CMD ["/bin/bash", "-lc", "rustc --version && cargo --version"]
