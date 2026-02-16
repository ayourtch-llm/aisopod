# Issue 153: Create Multi-Stage Dockerfile for aisopod

## Summary
Create a multi-stage Dockerfile at the repository root that produces a minimal runtime image for aisopod with non-root execution, health checks, and persistent storage support.

## Location
- Crate: `aisopod` (workspace root)
- File: `Dockerfile`

## Current Behavior
No Dockerfile exists in the repository. There is no containerized way to build or run aisopod.

## Expected Behavior
A multi-stage Dockerfile at the repo root that:
- Uses `rust:latest` as the build stage to compile the release binary.
- Uses `debian:bookworm-slim` as the runtime stage with only `ca-certificates` installed.
- Runs as a non-root user (1000:1000).
- Exposes port 18789 for the gateway.
- Includes a health check command.
- Mounts `/data` as a volume for persistent storage.

## Impact
Enables containerized deployment of aisopod, which is the foundation for Docker Compose, Fly.io, Render, and CI/CD release pipelines.

## Suggested Implementation

Create `Dockerfile` at the repository root:

```dockerfile
# Build stage
FROM rust:latest AS builder
WORKDIR /app
COPY . .
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
```

Add a `.dockerignore` file to speed up builds:

```
target/
.git/
docs/
tmp/
```

## Dependencies
- Issue 012 (binary crate setup)
- Issue 125 (gateway command implementation)

## Acceptance Criteria
- [ ] `docker build -t aisopod .` completes successfully
- [ ] `docker run aisopod` starts the gateway on port 18789
- [ ] Container runs as non-root user (UID 1000)
- [ ] Health check passes when gateway is running
- [ ] `/data` volume is available for persistent storage
- [ ] Runtime image size is under 150 MB

---
*Created: 2026-02-15*
