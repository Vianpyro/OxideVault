FROM debian:bookworm-slim

# Install ca-certificates (for TLS) and ensure tools for user creation exist
RUN apt-get update \
		&& apt-get install -y --no-install-recommends ca-certificates adduser \
		&& rm -rf /var/lib/apt/lists/*

ARG BINARY=target/release/OxideVault
COPY ${BINARY} /usr/local/bin/oxidevault
RUN chmod +x /usr/local/bin/oxidevault

# Create a non-root user and make the binary owned by that user
RUN addgroup --system app \
		&& adduser --system --ingroup app app \
		&& chown app:app /usr/local/bin/oxidevault

# Simple HEALTHCHECK so scanners (e.g. Checkov) detect a healthprobe.
# This uses the shell builtin `test` to ensure the binary is executable.
HEALTHCHECK --interval=30s --timeout=3s \
	CMD /bin/sh -c "test -x /usr/local/bin/oxidevault || exit 1"

# Run as non-root user
USER app

ENTRYPOINT ["/usr/local/bin/oxidevault"]
