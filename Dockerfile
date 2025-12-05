FROM debian:bookworm-slim

# Install ca-certificates (for TLS) and ensure tools for user creation exist
RUN apt-get update \
		&& apt-get install -y --no-install-recommends ca-certificates adduser \
		&& rm -rf /var/lib/apt/lists/*

# NOTE: This Dockerfile expects a pre-built binary at the specified path.
# Ensure you run `cargo build --release` before building this image.
ARG BINARY=target/release/oxidevault
COPY ${BINARY} /usr/local/bin/oxidevault
RUN chmod +x /usr/local/bin/oxidevault

# Create a non-root user and make the binary owned by that user
RUN addgroup --system app \
		&& adduser --system --ingroup app app \
		&& chown app:app /usr/local/bin/oxidevault

# Note: This HEALTHCHECK only verifies that the binary is executable, not that the
# Discord bot is connected or responsive. Consider implementing a proper health
# endpoint in the application for more meaningful monitoring.
HEALTHCHECK --interval=30s --timeout=3s \
	CMD /bin/sh -c "test -x /usr/local/bin/oxidevault || exit 1"

# Run as non-root user
USER app

ENTRYPOINT ["/usr/local/bin/oxidevault"]
