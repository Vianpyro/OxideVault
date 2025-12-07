FROM rust:1-bookworm as builder

WORKDIR /build

# Layer caching
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

COPY . .
RUN touch src/main.rs  # Force rebuild of main crate only
RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update \
        && apt-get install -y --no-install-recommends ca-certificates \
        && rm -rf /var/lib/apt/lists/*

COPY --from=builder /build/target/release/oxidevault /usr/local/bin/oxidevault
RUN chmod +x /usr/local/bin/oxidevault

RUN addgroup --system app \
        && adduser --system --ingroup app app \
        && chown app:app /usr/local/bin/oxidevault

RUN mkdir -p /data \
        && chown -R app:app /data

HEALTHCHECK --interval=30s --timeout=3s \
    CMD /bin/sh -c "test -x /usr/local/bin/oxidevault || exit 1"

USER app
WORKDIR /data

ENTRYPOINT ["/usr/local/bin/oxidevault"]
