# Issue 154: Create Docker Compose Configuration

## Summary
Create a `docker-compose.yml` file that defines the aisopod service with port mapping, named volumes for persistent data, config directory mounting, and environment variable configuration.

## Location
- Crate: `aisopod` (workspace root)
- File: `docker-compose.yml`

## Current Behavior
No Docker Compose configuration exists. Users must manually pass all Docker run flags to start aisopod in a container.

## Expected Behavior
A `docker-compose.yml` at the repo root that allows `docker compose up` to start a fully configured aisopod instance with persistent storage and external configuration.

## Impact
Simplifies local development and self-hosted deployments by providing a single-command setup for running aisopod in Docker.

## Suggested Implementation

Create `docker-compose.yml` at the repository root:

```yaml
services:
  aisopod:
    build: .
    ports:
      - "18789:18789"
    volumes:
      - aisopod-data:/data
      - ./config:/config:ro
    environment:
      - AISOPOD_CONFIG=/config/aisopod.json
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "aisopod", "health"]
      interval: 30s
      timeout: 5s
      retries: 3
      start_period: 10s

volumes:
  aisopod-data:
    driver: local
```

Also create a `config/` directory with an example configuration file:

```bash
mkdir -p config
```

Add `config/aisopod.example.json` with a minimal starter config that users can copy to `config/aisopod.json`.

## Dependencies
- Issue 153 (Dockerfile)

## Acceptance Criteria
- [ ] `docker compose up` builds and starts aisopod successfully
- [ ] Port 18789 is accessible from the host
- [ ] Named volume `aisopod-data` persists data across restarts
- [ ] Configuration files in `./config/` are mounted read-only into the container
- [ ] `AISOPOD_CONFIG` environment variable is set correctly
- [ ] `docker compose down && docker compose up` preserves data in the named volume

---
*Created: 2026-02-15*
