# Build stage
FROM rust:latest AS builder
WORKDIR /app
COPY . .

# Build the release binary
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/aisopod /usr/local/bin/aisopod

# Create non-root user
RUN groupadd -g 1000 aisopod && useradd -u 1000 -g aisopod -m aisopod

# Persistent data volume
RUN mkdir -p /data && chown aisopod:aisopod /data
VOLUME ["/data"]

USER 1000:1000

EXPOSE 18789

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD ["aisopod", "health"] || exit 1

CMD ["aisopod", "gateway", "--allow-unconfigured"]
