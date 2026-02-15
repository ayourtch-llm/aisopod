# 0016 — Deployment & Packaging

**Master Plan Reference:** Section 3.16 — Deployment & Packaging  
**Phase:** 7 (Production)  
**Dependencies:** 0003 (Gateway), 0013 (CLI), 0014 (Web UI)

---

## Objective

Create deployment configurations and packaging for aisopod across multiple
platforms: Docker, Fly.io, Render, systemd/launchctl services, and
platform-specific packages.

---

## Deliverables

### 1. Docker Image

**Multi-stage Dockerfile:**
```dockerfile
# Build stage
FROM rust:latest AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/aisopod /usr/local/bin/
USER 1000:1000
EXPOSE 18789
CMD ["aisopod", "gateway", "--allow-unconfigured"]
```

**Features:**
- Small runtime image (no build tools)
- Non-root execution
- Configurable bind address
- Volume mount for persistent data (`/data`)
- Health check endpoint

### 2. Docker Compose

```yaml
services:
  aisopod:
    build: .
    ports:
      - "18789:18789"
    volumes:
      - aisopod-data:/data
      - ./config:/config
    environment:
      - AISOPOD_CONFIG=/config/aisopod.json
    restart: unless-stopped
```

### 3. Fly.io Deployment

- `fly.toml` with aisopod-specific configuration
- Persistent volume for state
- Health check configuration
- Auto-scaling settings
- Region configuration

### 4. Render Deployment

- `render.yaml` service definition
- Persistent disk configuration
- Environment variable setup
- Health check path

### 5. Systemd Service

```ini
[Unit]
Description=Aisopod AI Gateway
After=network.target

[Service]
Type=simple
User=aisopod
ExecStart=/usr/local/bin/aisopod gateway
Restart=on-failure
RestartSec=5

[Install]
WantedBy=multi-user.target
```

- Service file generation via `aisopod daemon install`
- Log rotation configuration
- Socket activation (optional)

### 6. launchctl Service (macOS)

- Property list generation
- Launch agent vs launch daemon
- Log directory setup
- Auto-start configuration

### 7. Platform Packages

- **Homebrew formula** for macOS
- **Cargo install** from crates.io
- **GitHub Releases** with pre-built binaries
- Cross-compilation matrix:
  - `x86_64-unknown-linux-gnu`
  - `x86_64-unknown-linux-musl` (static)
  - `aarch64-unknown-linux-gnu`
  - `x86_64-apple-darwin`
  - `aarch64-apple-darwin`
  - `x86_64-pc-windows-msvc`

### 8. CI/CD Release Pipeline

- GitHub Actions workflow for:
  - Build and test on all platforms
  - Cross-compilation for release targets
  - Docker image build and push
  - GitHub Release creation with binaries
  - Homebrew formula update
  - Crates.io publish

### 9. Configuration Templates

- Default config file generation
- Environment-specific templates (development, production, docker)
- Migration guide from OpenClaw configuration

---

## Acceptance Criteria

- [ ] Docker image builds and runs correctly
- [ ] Docker Compose setup starts a working instance
- [ ] Fly.io deployment works with persistent storage
- [ ] Render deployment works with health checks
- [ ] Systemd service installs and manages lifecycle
- [ ] launchctl service works on macOS
- [ ] Pre-built binaries available for major platforms
- [ ] CI/CD pipeline produces release artifacts
- [ ] Configuration migration from OpenClaw format works
- [ ] Documentation covers all deployment methods
